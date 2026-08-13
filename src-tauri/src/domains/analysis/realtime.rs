//! Pure sampling policy for the live team-analysis screen.
//!
//! The LCU adapter fetches one recent list. This module decides which games are the
//! primary evidence for the current match without issuing network requests.

use serde_json::Value;

use crate::domains::analysis::analyzers::core::parser::{parse_games, ParsedGame};
use crate::domains::analysis::analyzers::core::stats::{
    analyze_player_stats_with_resolver, match_performance_from_game, AnalysisContext, ChampionNameResolver,
};
use crate::shared::types::{
    MatchPerformance, PlayerAnalysisBasis, PlayerAnalysisConfidence, PlayerAnalysisScope, PlayerMatchStats,
    PlayerRankSummary, RankedPlayerRating, RankedRatingGrade, RankedRatingQueue, SummonerTrait,
};

const MIN_PRIMARY_SAMPLE: usize = 5;
const MIN_GAME_DURATION_SECONDS: i32 = 300;
const MAX_ANALYSIS_SAMPLE: usize = 20;
const VISIBLE_RECENT_MATCHES: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RealtimeMatchContext {
    SoloRanked,
    FlexRanked,
    SummonersRift,
    Aram,
    Custom,
    CurrentMode(i64),
}

impl RealtimeMatchContext {
    pub fn from_lcu(queue_id: i64, is_custom_game: bool) -> Self {
        if is_custom_game {
            return Self::Custom;
        }

        match queue_id {
            420 => Self::SoloRanked,
            440 => Self::FlexRanked,
            400 | 430 | 490 => Self::SummonersRift,
            450 => Self::Aram,
            other => Self::CurrentMode(other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RealtimePlayerAnalysis {
    pub stats: PlayerMatchStats,
    pub basis: PlayerAnalysisBasis,
    pub recent_matches: Vec<MatchPerformance>,
}

pub fn analyze_realtime_history(
    games: &[Value],
    target_puuid: &str,
    context: RealtimeMatchContext,
    champion_name: Option<ChampionNameResolver<'_>>,
) -> Option<RealtimePlayerAnalysis> {
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

    let ranked: Vec<ParsedGame> = eligible
        .iter()
        .filter(|game| is_ranked(game.queue_id))
        .cloned()
        .collect();
    let solo: Vec<ParsedGame> = ranked.iter().filter(|game| game.queue_id == 420).cloned().collect();
    let flex: Vec<ParsedGame> = ranked.iter().filter(|game| game.queue_id == 440).cloned().collect();
    let normal_rift: Vec<ParsedGame> = eligible
        .iter()
        .filter(|game| is_normal_rift(game.queue_id))
        .cloned()
        .collect();
    let summoners_rift: Vec<ParsedGame> = eligible
        .iter()
        .filter(|game| is_ranked(game.queue_id) || is_normal_rift(game.queue_id))
        .cloned()
        .collect();
    let aram: Vec<ParsedGame> = eligible.iter().filter(|game| game.queue_id == 450).cloned().collect();

    let selected = match context {
        RealtimeMatchContext::SoloRanked if solo.len() >= MIN_PRIMARY_SAMPLE => {
            Selection::new(solo, PlayerAnalysisScope::SoloRanked, 0, false)
        }
        RealtimeMatchContext::FlexRanked if flex.len() >= MIN_PRIMARY_SAMPLE => {
            Selection::new(flex, PlayerAnalysisScope::FlexRanked, 0, false)
        }
        RealtimeMatchContext::SoloRanked if ranked.len() >= MIN_PRIMARY_SAMPLE => {
            let supporting_games = ranked.iter().filter(|game| game.queue_id != 420).count();
            Selection::new(ranked, PlayerAnalysisScope::Ranked, supporting_games, true)
        }
        RealtimeMatchContext::FlexRanked if ranked.len() >= MIN_PRIMARY_SAMPLE => {
            let supporting_games = ranked.iter().filter(|game| game.queue_id != 440).count();
            Selection::new(ranked, PlayerAnalysisScope::Ranked, supporting_games, true)
        }
        RealtimeMatchContext::SummonersRift if ranked.len() >= MIN_PRIMARY_SAMPLE => {
            Selection::new(ranked, PlayerAnalysisScope::Ranked, 0, false)
        }
        RealtimeMatchContext::SoloRanked if !summoners_rift.is_empty() => {
            let supporting_games = summoners_rift.iter().filter(|game| game.queue_id != 420).count();
            Selection::new(
                summoners_rift,
                PlayerAnalysisScope::SummonersRift,
                supporting_games,
                true,
            )
        }
        RealtimeMatchContext::FlexRanked if !summoners_rift.is_empty() => {
            let supporting_games = summoners_rift.iter().filter(|game| game.queue_id != 440).count();
            Selection::new(
                summoners_rift,
                PlayerAnalysisScope::SummonersRift,
                supporting_games,
                true,
            )
        }
        RealtimeMatchContext::SummonersRift if !summoners_rift.is_empty() => Selection::new(
            summoners_rift,
            PlayerAnalysisScope::SummonersRift,
            normal_rift.len(),
            true,
        ),
        RealtimeMatchContext::Custom if ranked.len() >= MIN_PRIMARY_SAMPLE => {
            Selection::new(ranked, PlayerAnalysisScope::Ranked, 0, false)
        }
        RealtimeMatchContext::Aram if !aram.is_empty() => Selection::new(aram, PlayerAnalysisScope::Aram, 0, false),
        RealtimeMatchContext::CurrentMode(queue_id) => {
            let current_mode: Vec<ParsedGame> = eligible
                .iter()
                .filter(|game| game.queue_id == queue_id)
                .cloned()
                .collect();
            if current_mode.is_empty() {
                Selection::new(
                    eligible.clone(),
                    PlayerAnalysisScope::RecentOverall,
                    eligible.len(),
                    true,
                )
            } else {
                Selection::new(current_mode, PlayerAnalysisScope::CurrentMode, 0, false)
            }
        }
        _ => Selection::new(
            eligible.clone(),
            PlayerAnalysisScope::RecentOverall,
            eligible.len(),
            true,
        ),
    }
    .cap_for_context(context);

    let primary_games = selected.games.len() as u32;
    let mut stats =
        analyze_player_stats_with_resolver(&selected.games, target_puuid, AnalysisContext::new(), champion_name);
    stats.traits = build_realtime_traits(&stats);

    Some(RealtimePlayerAnalysis {
        stats,
        recent_matches,
        basis: PlayerAnalysisBasis {
            primary_scope: selected.scope,
            scanned_games: eligible.len() as u32,
            primary_games,
            supporting_games: selected.supporting_games as u32,
            confidence: confidence_for(primary_games),
            fallback_used: selected.fallback_used,
        },
    })
}

/// Real-time cards need concise conclusions that can be computed from the already fetched match
/// list. Every label includes its sample and measured value; no timeline-only claim is invented.
fn build_realtime_traits(stats: &PlayerMatchStats) -> Vec<SummonerTrait> {
    if stats.total_games < MIN_PRIMARY_SAMPLE as u32 {
        return Vec::new();
    }

    let (form_name, form_type, form_score) = if stats.win_rate >= 60.0 {
        ("近期状态强势", "good", 80)
    } else if stats.win_rate <= 40.0 {
        ("近期状态低迷", "bad", -80)
    } else {
        ("近期状态平稳", "neutral", 0)
    };
    let mut traits = vec![SummonerTrait {
        name: form_name.to_string(),
        description: format!(
            "最近 {} 场取得 {} 胜 {} 负，胜率 {:.0}%",
            stats.total_games, stats.wins, stats.losses, stats.win_rate
        ),
        score: form_score,
        trait_type: form_type.to_string(),
    }];

    if stats.avg_deaths <= 4.0 {
        traits.push(SummonerTrait {
            name: "生存稳定".to_string(),
            description: format!("最近样本场均死亡 {:.1} 次", stats.avg_deaths),
            score: 70,
            trait_type: "good".to_string(),
        });
    } else if stats.avg_deaths >= 7.0 {
        traits.push(SummonerTrait {
            name: "阵亡偏多".to_string(),
            description: format!("最近样本场均死亡 {:.1} 次", stats.avg_deaths),
            score: -70,
            trait_type: "bad".to_string(),
        });
    }

    if stats.avg_kda >= 4.0 {
        traits.push(SummonerTrait {
            name: "贡献效率突出".to_string(),
            description: format!("最近样本平均 KDA {:.2}", stats.avg_kda),
            score: 70,
            trait_type: "good".to_string(),
        });
    } else if stats.avg_kda <= 2.0 {
        traits.push(SummonerTrait {
            name: "贡献效率偏低".to_string(),
            description: format!("最近样本平均 KDA {:.2}", stats.avg_kda),
            score: -60,
            trait_type: "bad".to_string(),
        });
    }

    if traits.len() < 3 {
        if let Some(champion) = stats
            .favorite_champions
            .first()
            .filter(|champion| champion.champion_name.is_some())
        {
            let share = champion.games as f64 / stats.total_games as f64;
            if champion.games >= 3 && share >= 0.45 {
                let champion_name = champion.champion_name.as_deref().unwrap_or_default();
                traits.push(SummonerTrait {
                    name: "英雄选择集中".to_string(),
                    description: format!(
                        "{} 在最近样本中使用 {} 场，占 {:.0}%",
                        champion_name,
                        champion.games,
                        share * 100.0
                    ),
                    score: 0,
                    trait_type: "neutral".to_string(),
                });
            }
        }
    }

    traits.truncate(3);
    traits
}

struct Selection {
    games: Vec<ParsedGame>,
    scope: PlayerAnalysisScope,
    supporting_games: usize,
    fallback_used: bool,
}

impl Selection {
    fn new(games: Vec<ParsedGame>, scope: PlayerAnalysisScope, supporting_games: usize, fallback_used: bool) -> Self {
        Self {
            games,
            scope,
            supporting_games,
            fallback_used,
        }
    }

    fn cap_for_context(mut self, context: RealtimeMatchContext) -> Self {
        self.games.truncate(MAX_ANALYSIS_SAMPLE);
        self.supporting_games = match (context, self.scope) {
            (RealtimeMatchContext::SoloRanked, PlayerAnalysisScope::Ranked) => {
                self.games.iter().filter(|game| game.queue_id != 420).count()
            }
            (RealtimeMatchContext::FlexRanked, PlayerAnalysisScope::Ranked) => {
                self.games.iter().filter(|game| game.queue_id != 440).count()
            }
            (RealtimeMatchContext::SoloRanked, PlayerAnalysisScope::SummonersRift) => {
                self.games.iter().filter(|game| game.queue_id != 420).count()
            }
            (RealtimeMatchContext::FlexRanked, PlayerAnalysisScope::SummonersRift) => {
                self.games.iter().filter(|game| game.queue_id != 440).count()
            }
            (RealtimeMatchContext::SummonersRift, PlayerAnalysisScope::SummonersRift) => {
                self.games.iter().filter(|game| is_normal_rift(game.queue_id)).count()
            }
            (_, PlayerAnalysisScope::RecentOverall) => self.games.len(),
            _ => self.supporting_games.min(self.games.len()),
        };
        self
    }
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
    let (queue, rank) = select_rating_rank(queue_id, basis.primary_scope, solo_rank, flex_rank)?;
    let rank_score = rank_score(rank)?;
    let has_ranked_sample = matches!(
        basis.primary_scope,
        PlayerAnalysisScope::SoloRanked | PlayerAnalysisScope::FlexRanked | PlayerAnalysisScope::Ranked
    );
    let sample_factor = if has_ranked_sample {
        match stats.total_games {
            0..=4 => 0.0,
            5..=7 => 0.4,
            8..=14 => 0.7,
            _ => 1.0,
        }
    } else {
        0.0
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
    let form_summary = if !has_ranked_sample {
        "当前样本并非排位，仅展示段位基础评级"
    } else if stats.total_games < MIN_PRIMARY_SAMPLE as u32 {
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
    scope: PlayerAnalysisScope,
    solo_rank: Option<&'a PlayerRankSummary>,
    flex_rank: Option<&'a PlayerRankSummary>,
) -> Option<(RankedRatingQueue, &'a PlayerRankSummary)> {
    match queue_id {
        420 => solo_rank.map(|rank| (RankedRatingQueue::SoloRanked, rank)),
        440 => flex_rank.map(|rank| (RankedRatingQueue::FlexRanked, rank)),
        _ => match scope {
            PlayerAnalysisScope::SoloRanked => solo_rank.map(|rank| (RankedRatingQueue::SoloRanked, rank)),
            PlayerAnalysisScope::FlexRanked => flex_rank.map(|rank| (RankedRatingQueue::FlexRanked, rank)),
            _ => strongest_rating_rank(solo_rank, flex_rank),
        },
    }
}

fn strongest_rating_rank<'a>(
    solo_rank: Option<&'a PlayerRankSummary>,
    flex_rank: Option<&'a PlayerRankSummary>,
) -> Option<(RankedRatingQueue, &'a PlayerRankSummary)> {
    [
        solo_rank.map(|rank| (RankedRatingQueue::SoloRanked, rank)),
        flex_rank.map(|rank| (RankedRatingQueue::FlexRanked, rank)),
    ]
    .into_iter()
    .flatten()
    .filter_map(|(queue, rank)| rank_score(rank).map(|score| (score, queue, rank)))
    .max_by(|left, right| left.0.total_cmp(&right.0))
    .map(|(_, queue, rank)| (queue, rank))
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

fn is_ranked(queue_id: i64) -> bool {
    matches!(queue_id, 420 | 440)
}

fn is_normal_rift(queue_id: i64) -> bool {
    matches!(queue_id, 400 | 430 | 490)
}

#[cfg(test)]
#[path = "realtime_tests.rs"]
mod tests;
