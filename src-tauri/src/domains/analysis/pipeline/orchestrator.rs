//! 唯一分析编排器（纯领域）
//!
//! 这里是「一次对局分析」的唯一执行路径：队列过滤 → 基础统计 → 位置分组 →
//! Evidence → 能力收敛 → 诊断汇总 → 确定性特征与建议。
//!
//! DDD 约束：本模块**不**依赖 `infrastructure`、`reqwest`、LCU。
//! 输入全部是纯视图（原始对局 JSON + 可选时间线 JSON + 目标 PUUID），
//! `MatchBundle → OrchestratorInput` 的映射由命令层/应用服务负责。
//!
//! 两个数量刻意分开，不要混用：
//! - `count`（展示场数）决定 `display_games` / `matches` / 基础统计的覆盖范围
//! - `max_analysis_games`（深度上限）只约束 Evidence 与时间线请求
//!
//! 特征与建议**只能**来自 Evidence。原始 JSON 不允许再走第二条特征计算链路，
//! 否则同一个结论会出现两个互相矛盾的版本。
//!
//! `result.diagnostics` 契约：策略级 + 运行期聚合后的**唯一**列表，
//! 同一 `(code, feature)` 只保留信息量最大的一条，禁止每局刷屏。

use std::collections::HashMap;

use serde_json::Value;

use crate::domains::analysis::analyzers::core::parser::{parse_games, ParsedGame};
use crate::domains::analysis::analyzers::core::stats::{analyze_player_stats_with_resolver, AnalysisContext};
use crate::domains::analysis::evidence::{
    build_evidence_bundle, position_from_role_lane, EvidenceBundle, EvidenceIssue, EvidencePosition, EvidenceQuality,
    MatchEvidence, MatchEvidenceInput,
};
use crate::shared::types::{ChampionStat, MatchPerformance, PositionStats, SummonerTrait, TrendPoint};

use super::insights::build_deterministic_advice;
use super::process_insights::build_process_insight;
use super::trait_strategies::{analyze_traits, TraitAnalysisContext};
use super::types::{
    is_ranked_queue, AnalysisCapabilities, AnalysisDegradationCode, AnalysisDepth, AnalysisDiagnostic, AnalysisFeature,
    AnalysisPolicy, DeterministicAdvice, DeterministicTrait, MatchAnalysisRequest, MatchAnalysisResult,
    UNKNOWN_POSITION,
};

/// 英雄池展示条数
const CHAMPION_POOL_SIZE: usize = 5;

/// 胜率趋势的移动平均窗口
const WIN_RATE_TREND_WINDOW: usize = 5;

/// 一场「可做深度证据」的对局素材（纯视图）
#[derive(Debug, Clone, Copy)]
pub struct DeepMatchInput<'a> {
    pub game: &'a Value,
    pub timeline: Option<&'a Value>,
    pub queue_id: i64,
}

/// 编排器输入（全部为纯视图）
pub struct OrchestratorInput<'a> {
    pub target_puuid: &'a str,
    pub display_games: &'a [Value],
    pub deep_matches: &'a [DeepMatchInput<'a>],
    pub champion_name: Option<&'a dyn Fn(i32) -> Option<String>>,
}

impl<'a> OrchestratorInput<'a> {
    pub fn new(target_puuid: &'a str, display_games: &'a [Value], deep_matches: &'a [DeepMatchInput<'a>]) -> Self {
        Self {
            target_puuid,
            display_games,
            deep_matches,
            champion_name: None,
        }
    }

    pub fn with_champion_name(mut self, resolver: &'a dyn Fn(i32) -> Option<String>) -> Self {
        self.champion_name = Some(resolver);
        self
    }

    fn champion_name_of(&self, champion_id: i32) -> String {
        self.champion_name
            .and_then(|resolve| resolve(champion_id))
            .unwrap_or_else(|| format!("未知英雄({})", champion_id))
    }
}

/// 执行一次完整分析
pub fn orchestrate_analysis(
    request: &MatchAnalysisRequest,
    policy: &AnalysisPolicy,
    input: OrchestratorInput<'_>,
) -> MatchAnalysisResult {
    let parsed: Vec<ParsedGame> = parse_games(input.display_games, input.target_puuid)
        .into_iter()
        .filter(|game| policy.includes_queue(game.queue_id))
        .collect();

    if parsed.is_empty() {
        return MatchAnalysisResult::empty(policy.clone());
    }

    let mut overall_stats =
        analyze_player_stats_with_resolver(&parsed, input.target_puuid, AnalysisContext::new(), input.champion_name);
    let matches: Vec<MatchPerformance> = overall_stats.recent_performance.clone();
    let display_games = overall_stats.total_games;

    let ranked_games: Vec<ParsedGame> = parsed
        .iter()
        .filter(|game| is_ranked_queue(game.queue_id))
        .cloned()
        .collect();
    let other_games: Vec<ParsedGame> = parsed
        .iter()
        .filter(|game| !is_ranked_queue(game.queue_id))
        .cloned()
        .collect();

    let ranked_stats = if ranked_games.is_empty() {
        None
    } else {
        Some(analyze_player_stats_with_resolver(
            &ranked_games,
            input.target_puuid,
            AnalysisContext::new(),
            input.champion_name,
        ))
    };
    let mut other_stats = if other_games.is_empty() {
        None
    } else {
        Some(analyze_player_stats_with_resolver(
            &other_games,
            input.target_puuid,
            AnalysisContext::new(),
            input.champion_name,
        ))
    };

    // 非排位桶单独算模式亲和/娱乐特征，避免「全部」样本被排位占比压成「排位为主」
    if let Some(stats) = other_stats.as_mut() {
        let other_traits = analyze_traits(&TraitAnalysisContext {
            display_games: &other_games,
            evidence_matches: &[],
            position: None,
        });
        stats.traits = legacy_traits(&other_traits);
    }

    let mut diagnostics = policy.diagnostics.clone();
    let evidence = build_evidence(policy, &input, &mut diagnostics);

    let evidence_matches: &[MatchEvidence] = evidence.as_ref().map(|bundle| bundle.matches.as_slice()).unwrap_or(&[]);
    let traits = analyze_traits(&TraitAnalysisContext {
        display_games: &parsed,
        evidence_matches,
        position: None,
    });
    let advice = build_deterministic_advice(
        &traits,
        request.effective_perspective(),
        request.target_player.as_deref(),
    );

    push_sample_diagnostics(&traits, &mut diagnostics);

    overall_stats.traits = legacy_traits(&traits);
    overall_stats.advice = advice.iter().map(DeterministicAdvice::to_legacy_advice).collect();

    let position_stats = build_position_stats(&parsed, &input, evidence_matches, request);
    let main_position = position_stats
        .first()
        .map(|stats| stats.position.clone())
        .unwrap_or_else(|| UNKNOWN_POSITION.to_string());

    let capabilities = refine_capabilities(policy, evidence.as_ref(), &position_stats);
    dedupe_diagnostics(&mut diagnostics);

    MatchAnalysisResult {
        overall_stats,
        ranked_stats,
        other_stats,
        position_stats,
        main_position,
        analyzed_games: evidence.as_ref().map(|bundle| bundle.match_count).unwrap_or(0),
        display_games,
        matches,
        traits,
        advice,
        policy: policy.clone(),
        capabilities,
        diagnostics,
        evidence,
        ai_insight: None,
    }
}

/// 能力声明必须与实际输出一致：开关为真不等于结果里真有该维度。
fn refine_capabilities(
    policy: &AnalysisPolicy,
    evidence: Option<&EvidenceBundle>,
    position_stats: &[PositionStats],
) -> AnalysisCapabilities {
    let observed_queues: Vec<i64> = evidence
        .map(|bundle| bundle.matches.iter().map(|m| m.queue_id).collect())
        .unwrap_or_default();

    let mut caps = AnalysisCapabilities::from_policy(policy).refined_with_observed_queues(&observed_queues);

    // 位置拆分：有分组结果即可（Simple 也产出英雄池/趋势）
    caps.position_breakdown = !position_stats.is_empty();

    let has_full_timeline = evidence
        .map(|bundle| bundle.matches.iter().any(|m| m.quality == EvidenceQuality::Full))
        .unwrap_or(false);
    let has_opponent = evidence
        .map(|bundle| bundle.matches.iter().any(|m| m.opponent.is_some()))
        .unwrap_or(false);

    caps.timeline = caps.timeline && caps.deep_analysis && has_full_timeline;
    caps.opponent = caps.opponent && caps.deep_analysis && has_opponent;
    // 队友 / 自我提升尚未从 Evidence 独立产出；在接入前不得声明可用（避免设置页开关误导）
    caps.teammate = false;
    caps.self_improvement = false;

    caps
}

/// 同一 `(code, feature)` 只保留第一条（通常信息量最大）
pub fn dedupe_diagnostics(diagnostics: &mut Vec<AnalysisDiagnostic>) {
    let mut seen = std::collections::HashSet::new();
    diagnostics.retain(|d| seen.insert((d.code, d.feature)));
}

fn build_evidence(
    policy: &AnalysisPolicy,
    input: &OrchestratorInput<'_>,
    diagnostics: &mut Vec<AnalysisDiagnostic>,
) -> Option<EvidenceBundle> {
    if policy.basic_only {
        return None;
    }

    let candidates: Vec<&DeepMatchInput<'_>> = input
        .deep_matches
        .iter()
        .filter(|candidate| {
            policy.includes_queue(candidate.queue_id)
                && policy.depth_for_queue(candidate.queue_id) == AnalysisDepth::Deep
        })
        .collect();

    if candidates.is_empty() {
        return None;
    }

    let evidence_inputs: Vec<MatchEvidenceInput<'_>> = candidates
        .iter()
        .map(|candidate| MatchEvidenceInput::new(candidate.game, candidate.timeline))
        .collect();

    let bundle = build_evidence_bundle(input.target_puuid, &evidence_inputs);

    let skipped = bundle
        .diagnostics
        .iter()
        .filter(|d| d.code == EvidenceIssue::MatchSkipped)
        .count();
    if skipped > 0 {
        diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::EvidenceExtractionFailed,
            AnalysisFeature::DeepAnalysis,
            format!("{} 场对局的证据提取失败，已跳过这些对局", skipped),
        ));
    }

    let missing_timeline = bundle
        .matches
        .iter()
        .filter(|evidence| evidence.quality != EvidenceQuality::Full)
        .count();
    if missing_timeline > 0 {
        diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::TimelineDataUnavailable,
            AnalysisFeature::Timeline,
            format!(
                "{} 场对局的时间线不可用或不完整，分阶段结论已相应收敛",
                missing_timeline
            ),
        ));
    }

    Some(bundle)
}

fn push_sample_diagnostics(traits: &[DeterministicTrait], diagnostics: &mut Vec<AnalysisDiagnostic>) {
    if traits.is_empty() || traits.iter().any(|item| item.supports_conclusion) {
        return;
    }

    let sample = traits.iter().map(|item| item.sample_count).max().unwrap_or(0);
    diagnostics.push(AnalysisDiagnostic::with_feature(
        AnalysisDegradationCode::InsufficientEvidenceSample,
        AnalysisFeature::DeepAnalysis,
        format!("有效样本仅 {} 场，只给出事实描述，不下结论", sample),
    ));
}

fn legacy_traits(traits: &[DeterministicTrait]) -> Vec<SummonerTrait> {
    traits
        .iter()
        .filter(|item| item.supports_conclusion)
        .map(DeterministicTrait::to_legacy_trait)
        .collect()
}

/// 按统一位置枚举分组——**仅排位（420/440）**
///
/// 无排位样本时返回空列表，不跑娱乐/匹配的假分路聚合。
fn build_position_stats(
    parsed: &[ParsedGame],
    input: &OrchestratorInput<'_>,
    evidence_matches: &[MatchEvidence],
    request: &MatchAnalysisRequest,
) -> Vec<PositionStats> {
    let ranked: Vec<&ParsedGame> = parsed.iter().filter(|game| is_ranked_queue(game.queue_id)).collect();
    if ranked.is_empty() {
        return Vec::new();
    }

    let mut groups: HashMap<EvidencePosition, Vec<ParsedGame>> = HashMap::new();
    for game in ranked {
        let position = position_from_role_lane(&game.player_data.role, &game.player_data.lane, game.queue_id);
        // 排位才进五分路；UNKNOWN 仍保留以便暴露数据缺口，不建 ARAM/FLEX 桶
        if !position.is_lane_position() && position != EvidencePosition::Unknown {
            continue;
        }
        groups.entry(position).or_default().push(game.clone());
    }

    if groups.is_empty() {
        return Vec::new();
    }

    let mut stats_list: Vec<PositionStats> = groups
        .into_iter()
        .map(|(position, games)| {
            let mut stats = analyze_player_stats_with_resolver(
                &games,
                input.target_puuid,
                AnalysisContext::new(),
                input.champion_name,
            );

            let position_evidence: Vec<MatchEvidence> = evidence_matches
                .iter()
                .filter(|evidence| evidence.position == position)
                .cloned()
                .collect();
            let position_key = position.as_str();
            let position_traits = analyze_traits(&TraitAnalysisContext {
                display_games: &games,
                evidence_matches: &position_evidence,
                position: Some(position_key),
            });
            let position_advice = build_deterministic_advice(
                &position_traits,
                request.effective_perspective(),
                request.target_player.as_deref(),
            );
            stats.traits = legacy_traits(&position_traits);
            stats.advice = position_advice
                .iter()
                .map(DeterministicAdvice::to_legacy_advice)
                .collect();

            let games_count = games.len() as u32;
            let wins = games.iter().filter(|game| game.player_data.win).count() as u32;

            PositionStats {
                position: position_key.to_string(),
                games: games_count,
                wins,
                win_rate: percentage(wins, games_count),
                stats,
                champion_pool: Some(build_champion_pool(&games, input)),
                win_rate_trend: Some(build_win_rate_trend(&games)),
                process_insight: Some(build_process_insight(&position_evidence)),
            }
        })
        .collect();

    stats_list.sort_by(|a, b| b.games.cmp(&a.games).then_with(|| a.position.cmp(&b.position)));
    stats_list
}

fn percentage(part: u32, total: u32) -> f64 {
    if total == 0 {
        return 0.0;
    }
    (part as f64 / total as f64) * 100.0
}

fn build_champion_pool(games: &[ParsedGame], input: &OrchestratorInput<'_>) -> Vec<ChampionStat> {
    let mut totals: HashMap<i32, (u32, u32, f64)> = HashMap::new();

    for game in games {
        let player = &game.player_data;
        let entry = totals.entry(player.champion_id).or_insert((0, 0, 0.0));
        entry.0 += 1;
        if player.win {
            entry.1 += 1;
        }
        entry.2 += player.kda;
    }

    let mut pool: Vec<ChampionStat> = totals
        .into_iter()
        .map(|(champion_id, (games, wins, total_kda))| ChampionStat {
            champion_id,
            champion_name: Some(input.champion_name_of(champion_id)),
            games,
            wins,
            win_rate: percentage(wins, games),
            avg_kda: if games > 0 { total_kda / games as f64 } else { 0.0 },
        })
        .collect();

    pool.sort_by(|a, b| b.games.cmp(&a.games).then_with(|| a.champion_id.cmp(&b.champion_id)));
    pool.truncate(CHAMPION_POOL_SIZE);
    pool
}

fn build_win_rate_trend(games: &[ParsedGame]) -> Vec<TrendPoint> {
    let mut ordered: Vec<&ParsedGame> = games.iter().collect();
    ordered.sort_by_key(|game| game.game_creation);

    let mut cumulative_wins = 0u32;
    let mut points = Vec::with_capacity(ordered.len());

    for (index, game) in ordered.iter().enumerate() {
        let played = index as u32 + 1;
        if game.player_data.win {
            cumulative_wins += 1;
        }

        let cumulative_win_rate = percentage(cumulative_wins, played);
        let moving_avg_win_rate = if played as usize >= WIN_RATE_TREND_WINDOW {
            let window = &ordered[index + 1 - WIN_RATE_TREND_WINDOW..=index];
            let wins = window.iter().filter(|game| game.player_data.win).count() as u32;
            percentage(wins, window.len() as u32)
        } else {
            cumulative_win_rate
        };

        points.push(TrendPoint {
            index: index as u32,
            win: game.player_data.win,
            cumulative_win_rate,
            moving_avg_win_rate,
        });
    }

    points
}
