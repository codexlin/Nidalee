//! Pure sampling policy for the live team-analysis screen.
//!
//! The LCU adapter fetches one recent list. This module decides which games are the
//! primary evidence for the current match without issuing network requests.

use serde_json::Value;

use std::collections::BTreeMap;

use crate::domains::analysis::analyzers::core::parser::{parse_games, ParsedGame};
use crate::domains::analysis::analyzers::core::stats::{
    analyze_player_stats_with_resolver, match_performance_from_game, AnalysisContext, ChampionNameResolver,
};
use crate::domains::analysis::evidence::{position_from_lane, position_from_role_lane, EvidencePosition};
use crate::domains::analysis::queue_config::{AnalysisProfile, QueueType};
use crate::shared::types::{
    ChampionPerformance, CurrentPositionPerformance, PlayerAnalysisBasis, PlayerAnalysisConfidence,
    PlayerAnalysisDepth, PlayerAnalysisResult, PlayerAnalysisScope, PlayerMatchStats, PlayerPerformanceSample,
    PlayerRankSummary, PositionPerformance, RankedAnalysis, RankedPlayerRating, RankedPositionProfile,
    RankedRatingGrade, RankedRatingQueue,
};

const MIN_PRIMARY_SAMPLE: usize = 5;
const MIN_GAME_DURATION_SECONDS: i32 = 300;
const MAX_ANALYSIS_SAMPLE: usize = 20;
const VISIBLE_RECENT_MATCHES: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RealtimeMatchContext {
    queue_id: i64,
    is_custom_game: bool,
}

impl RealtimeMatchContext {
    pub fn from_lcu(queue_id: i64, is_custom_game: bool) -> Self {
        Self {
            queue_id,
            is_custom_game,
        }
    }

    pub fn analysis_profile(self) -> AnalysisProfile {
        if self.is_custom_game {
            AnalysisProfile::Simple
        } else {
            QueueType::from_queue_id(self.queue_id as i32).analysis_profile()
        }
    }
}

pub fn analyze_realtime_history(
    games: &[Value],
    target_puuid: &str,
    context: RealtimeMatchContext,
    current_position: Option<&str>,
    current_champion_id: Option<i32>,
    champion_name: Option<ChampionNameResolver<'_>>,
) -> Option<PlayerAnalysisResult> {
    let mut eligible: Vec<ParsedGame> = parse_games(games, target_puuid)
        .into_iter()
        .filter(|game| game.game_duration >= MIN_GAME_DURATION_SECONDS)
        .collect();

    if eligible.is_empty() {
        return None;
    }

    eligible.sort_unstable_by_key(|game| std::cmp::Reverse(game.game_creation));
    let recent_matches = eligible
        .iter()
        .take(VISIBLE_RECENT_MATCHES)
        .map(|game| match_performance_from_game(game, champion_name))
        .collect();

    let (depth, scope, selected, ranked_queue) = match context.analysis_profile() {
        AnalysisProfile::Ranked(queue) => {
            let queue_id = context.queue_id;
            let selected = eligible
                .iter()
                .filter(|game| game.queue_id == queue_id)
                .take(MAX_ANALYSIS_SAMPLE)
                .cloned()
                .collect::<Vec<_>>();
            let scope = match queue {
                RankedRatingQueue::SoloRanked => PlayerAnalysisScope::SoloRanked,
                RankedRatingQueue::FlexRanked => PlayerAnalysisScope::FlexRanked,
            };
            (PlayerAnalysisDepth::Ranked, scope, selected, Some(queue))
        }
        AnalysisProfile::Simple => (
            PlayerAnalysisDepth::Simple,
            PlayerAnalysisScope::RecentOverall,
            eligible.iter().take(MAX_ANALYSIS_SAMPLE).cloned().collect(),
            None,
        ),
    };

    let primary_games = selected.len() as u32;
    let mut stats = analyze_player_stats_with_resolver(&selected, target_puuid, AnalysisContext::new(), champion_name);
    stats.traits.clear();
    let basis = PlayerAnalysisBasis {
        primary_scope: scope,
        scanned_games: eligible.len() as u32,
        primary_games,
        supporting_games: 0,
        confidence: confidence_for(primary_games),
        fallback_used: false,
    };
    let ranked = ranked_queue.map(|queue| {
        build_ranked_analysis(
            queue,
            context.queue_id,
            &selected,
            target_puuid,
            current_position,
            current_champion_id,
            champion_name,
        )
    });

    Some(PlayerAnalysisResult {
        depth,
        stats,
        basis,
        recent_matches,
        ranked,
    })
}

fn build_ranked_analysis(
    queue: RankedRatingQueue,
    queue_id: i64,
    games: &[ParsedGame],
    target_puuid: &str,
    current_position: Option<&str>,
    current_champion_id: Option<i32>,
    champion_name: Option<ChampionNameResolver<'_>>,
) -> RankedAnalysis {
    let mut position_groups: BTreeMap<EvidencePosition, Vec<ParsedGame>> = BTreeMap::new();
    let mut champion_groups: BTreeMap<i32, Vec<ParsedGame>> = BTreeMap::new();

    for game in games {
        let position = position_from_role_lane(&game.player_data.role, &game.player_data.lane, queue_id);
        if position.is_lane_position() {
            position_groups.entry(position).or_default().push(game.clone());
        }
        champion_groups
            .entry(game.player_data.champion_id)
            .or_default()
            .push(game.clone());
    }

    let mut positions = position_groups
        .into_iter()
        .map(|(position, games)| PositionPerformance {
            position: position.as_str().to_string(),
            sample: performance_sample(&games, target_puuid, champion_name),
        })
        .collect::<Vec<_>>();
    positions.sort_by(|left, right| {
        right
            .sample
            .games
            .cmp(&left.sample.games)
            .then_with(|| left.position.cmp(&right.position))
    });
    let primary_position = positions.first().map(|position| position.position.clone());

    let mut champions = champion_groups
        .into_iter()
        .map(|(champion_id, games)| ChampionPerformance {
            champion_id,
            champion_name: champion_name.and_then(|resolve| resolve(champion_id)),
            sample: performance_sample(&games, target_puuid, champion_name),
        })
        .collect::<Vec<_>>();
    champions.sort_by(|left, right| {
        right
            .sample
            .games
            .cmp(&left.sample.games)
            .then_with(|| left.champion_id.cmp(&right.champion_id))
    });

    let mut ranked = RankedAnalysis {
        queue,
        rating: None,
        position_profile: RankedPositionProfile {
            primary_position,
            positions,
            current_position: None,
        },
        champions,
        current_champion: None,
    };
    project_ranked_context(&mut ranked, current_position, current_champion_id, champion_name);
    ranked
}

fn performance_sample(
    games: &[ParsedGame],
    target_puuid: &str,
    champion_name: Option<ChampionNameResolver<'_>>,
) -> PlayerPerformanceSample {
    let stats = analyze_player_stats_with_resolver(games, target_puuid, AnalysisContext::new(), champion_name);
    PlayerPerformanceSample {
        games: stats.total_games,
        wins: stats.wins,
        win_rate: stats.win_rate,
        avg_kda: stats.avg_kda,
    }
}

/// Reprojects ChampSelect position and champion from the cached ranked breakdown.
/// This function is pure and never requires another history request.
pub fn project_ranked_context(
    ranked: &mut RankedAnalysis,
    current_position: Option<&str>,
    current_champion_id: Option<i32>,
    champion_name: Option<ChampionNameResolver<'_>>,
) {
    ranked.position_profile.current_position = current_position.and_then(normalize_current_position).map(|position| {
        let sample = ranked
            .position_profile
            .positions
            .iter()
            .find(|candidate| candidate.position == position)
            .map(|candidate| candidate.sample.clone())
            .unwrap_or_default();
        CurrentPositionPerformance {
            is_primary: ranked.position_profile.primary_position.as_deref() == Some(position.as_str()),
            position,
            sample,
        }
    });

    ranked.current_champion = current_champion_id.filter(|id| *id > 0).map(|champion_id| {
        ranked
            .champions
            .iter()
            .find(|champion| champion.champion_id == champion_id)
            .cloned()
            .unwrap_or_else(|| ChampionPerformance {
                champion_id,
                champion_name: champion_name.and_then(|resolve| resolve(champion_id)),
                sample: PlayerPerformanceSample::default(),
            })
    });
}

fn normalize_current_position(position: &str) -> Option<String> {
    let normalized = position.trim().to_ascii_uppercase();
    let position = match normalized.as_str() {
        "TOP" | "JUNGLE" | "MID" | "ADC" | "SUPPORT" => normalized,
        other => position_from_lane(other, other)?.as_str().to_string(),
    };
    Some(position)
}

fn confidence_for(sample: u32) -> PlayerAnalysisConfidence {
    match sample {
        15.. => PlayerAnalysisConfidence::High,
        8..=14 => PlayerAnalysisConfidence::Medium,
        _ => PlayerAnalysisConfidence::Low,
    }
}

/// Builds a compact ranked rating from the queue-specific rank and the selected recent sample.
/// Rank is the long-term anchor; recent form can only move it within a deliberately small range.
pub fn build_ranked_player_rating(
    queue_id: i64,
    solo_rank: Option<&PlayerRankSummary>,
    flex_rank: Option<&PlayerRankSummary>,
    stats: &PlayerMatchStats,
    basis: &PlayerAnalysisBasis,
) -> Option<RankedPlayerRating> {
    let (queue, rank) = select_rating_rank(queue_id, solo_rank, flex_rank)?;
    let rank_score = rank_score(rank)?;
    let sample_factor = match stats.total_games {
        0..=4 => 0.0,
        5..=7 => 0.4,
        8..=14 => 0.7,
        _ => 1.0,
    };
    let win_rate_adjustment = ((stats.win_rate - 50.0) * 0.35).clamp(-10.0, 10.0);
    let kda_adjustment = ((stats.avg_kda - 3.0) * 1.5).clamp(-4.0, 4.0);
    let form_adjustment = ((win_rate_adjustment + kda_adjustment) * sample_factor).clamp(-8.0, 8.0);
    let score = (rank_score + form_adjustment).clamp(0.0, 100.0);
    let (grade, label) = grade_for_score(score);
    let queue_label = match queue {
        RankedRatingQueue::SoloRanked => "单双排",
        RankedRatingQueue::FlexRanked => "灵活排位",
    };
    let confidence_label = match basis.confidence {
        PlayerAnalysisConfidence::Low => "低可信",
        PlayerAnalysisConfidence::Medium => "中可信",
        PlayerAnalysisConfidence::High => "高可信",
    };
    let division = rank
        .division
        .as_deref()
        .filter(|division| !division.is_empty())
        .unwrap_or("");
    let rank_label = format!("{} {}", tier_label(&rank.tier), division)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let form_summary = if stats.total_games < MIN_PRIMARY_SAMPLE as u32 {
        "近期样本不足，未进行状态修正"
    } else if form_adjustment >= 2.0 {
        "近期表现小幅上调"
    } else if form_adjustment <= -2.0 {
        "近期表现小幅下调"
    } else {
        "近期表现保持稳定"
    };

    Some(RankedPlayerRating {
        grade,
        label: label.to_string(),
        queue,
        confidence: basis.confidence,
        summary: format!(
            "以{queue_label} {rank_label}为基准，{form_summary}；{} 场样本，{confidence_label}",
            stats.total_games
        ),
    })
}

fn select_rating_rank<'a>(
    queue_id: i64,
    solo_rank: Option<&'a PlayerRankSummary>,
    flex_rank: Option<&'a PlayerRankSummary>,
) -> Option<(RankedRatingQueue, &'a PlayerRankSummary)> {
    match queue_id {
        420 => solo_rank.map(|rank| (RankedRatingQueue::SoloRanked, rank)),
        440 => flex_rank.map(|rank| (RankedRatingQueue::FlexRanked, rank)),
        _ => None,
    }
}

fn rank_score(rank: &PlayerRankSummary) -> Option<f64> {
    let tier = rank.tier.to_ascii_uppercase();
    let base = match tier.as_str() {
        "IRON" => 20.0,
        "BRONZE" => 28.0,
        "SILVER" => 36.0,
        "GOLD" => 45.0,
        "PLATINUM" => 54.0,
        "EMERALD" => 62.0,
        "DIAMOND" => 72.0,
        "MASTER" => 82.0,
        "GRANDMASTER" => 90.0,
        "CHALLENGER" => 96.0,
        _ => return None,
    };
    let division_bonus = match rank.division.as_deref().map(str::to_ascii_uppercase).as_deref() {
        Some("III") => 2.0,
        Some("II") => 4.0,
        Some("I") => 6.0,
        _ => 0.0,
    };
    let lp_bonus = if matches!(tier.as_str(), "MASTER" | "GRANDMASTER" | "CHALLENGER") {
        rank.league_points.unwrap_or_default().max(0) as f64 / 200.0
    } else {
        rank.league_points.unwrap_or_default().clamp(0, 100) as f64 / 50.0
    }
    .clamp(0.0, 4.0);
    Some((base + division_bonus + lp_bonus).min(100.0))
}

fn grade_for_score(score: f64) -> (RankedRatingGrade, &'static str) {
    if score >= 94.0 {
        (RankedRatingGrade::SPlus, "顶尖")
    } else if score >= 84.0 {
        (RankedRatingGrade::S, "强势")
    } else if score >= 70.0 {
        (RankedRatingGrade::A, "优秀")
    } else if score >= 57.0 {
        (RankedRatingGrade::B, "稳定")
    } else if score >= 42.0 {
        (RankedRatingGrade::C, "一般")
    } else {
        (RankedRatingGrade::D, "低迷")
    }
}

fn tier_label(tier: &str) -> &str {
    match tier.to_ascii_uppercase().as_str() {
        "IRON" => "坚韧黑铁",
        "BRONZE" => "英勇黄铜",
        "SILVER" => "不屈白银",
        "GOLD" => "荣耀黄金",
        "PLATINUM" => "华贵铂金",
        "EMERALD" => "流光翡翠",
        "DIAMOND" => "璀璨钻石",
        "MASTER" => "超凡大师",
        "GRANDMASTER" => "傲世宗师",
        "CHALLENGER" => "最强王者",
        _ => tier,
    }
}

#[cfg(test)]
#[path = "realtime_tests.rs"]
mod tests;
