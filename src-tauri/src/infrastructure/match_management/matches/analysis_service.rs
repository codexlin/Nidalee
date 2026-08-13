//! 对局分析应用服务（新旧命令的唯一执行路径）
//!
//! 这里是**唯一**把「请求 → 策略 → 数据获取 → 领域编排」串起来的地方：
//!
//! ```text
//! MatchAnalysisRequest
//!   → resolve_analysis_policy            (领域：策略解析)
//!   → MatchFetcher::fetch_bundles        (基础设施：一次列表 + 按需时间线)
//!   → MatchBundle → OrchestratorInput    (本层：把基础设施类型映射成纯视图)
//!   → orchestrate_analysis               (领域：唯一编排器)
//! ```
//!
//! 约束：
//! - 本层是普通函数，**不是** Tauri 命令。命令只做参数解析与结果投影，
//!   彼此之间不允许再 `invoke`，否则会绕出第二条抓取链路（并可能形成环）。
//! - 任何调用方都不得自己再请求战绩列表 / 时间线，一次分析只有一次列表请求。
//!
//! `result.diagnostics` 契约：编排器产出策略+运行期聚合后的唯一列表。
//! 本层不再把每局 FetchStage 明细原样刷进顶层；仅在编排器未覆盖的列表级失败时补充。

use reqwest::Client;

use crate::domains::analysis::pipeline::{
    dedupe_diagnostics, orchestrate_analysis, resolve_analysis_policy, AnalysisDegradationCode, AnalysisDepth,
    AnalysisDiagnostic, AnalysisFeature, AnalysisMode, DeepMatchInput, MatchAnalysisRequest, MatchAnalysisResult,
    OrchestratorInput,
};
use crate::infrastructure::data_services::champion_data::service::get_champion_info;
use crate::infrastructure::data_services::static_catalog::ensure_static_catalogs;
use crate::shared::types::{AdvicePerspective, GameAdvice, MultiPositionAnalysis, PlayerMatchStats};

use super::fetch_types::FetchStage;
use super::fetcher::{lcu_fetcher, MatchDataSource, MatchFetcher};

/// 旧入口的深度证据上限（仅 Deep 路径使用；与前端默认 maxAnalysisGames 对齐）
pub const LEGACY_MAX_ANALYSIS_GAMES: u32 = 20;

/// 战术建议分析的对局数量
pub const TACTICAL_ADVICE_GAME_COUNT: u32 = 20;

/// 执行一次完整分析（可注入数据源，便于测试）
pub async fn analyze_matches_with_fetcher<S: MatchDataSource>(
    fetcher: &MatchFetcher<S>,
    puuid: &str,
    request: &MatchAnalysisRequest,
) -> Result<MatchAnalysisResult, String> {
    if puuid.trim().is_empty() {
        return Err("分析目标 PUUID 不能为空".to_string());
    }

    let policy = resolve_analysis_policy(request);
    let outcome = fetcher.fetch_bundles(puuid, request.count as usize, &policy).await?;

    let deep_matches: Vec<DeepMatchInput<'_>> = outcome
        .bundles
        .iter()
        .map(|bundle| DeepMatchInput {
            game: bundle.game(),
            timeline: bundle.timeline.as_ref(),
            queue_id: bundle.queue_id,
        })
        .collect();

    let champion_name = |champion_id: i32| get_champion_info(champion_id).map(|info| info.name);
    let input = OrchestratorInput::new(puuid, &outcome.display_games, &deep_matches).with_champion_name(&champion_name);

    let mut result = orchestrate_analysis(request, &policy, input);

    // 仅补充列表级问题（编排器看不到）；单局 timeline/detail 由编排器聚合成一条
    if outcome.insufficient_matches_in_scope {
        let message = outcome
            .diagnostics
            .iter()
            .find(|d| d.stage == FetchStage::List && d.game_id == 0)
            .map(|d| d.message.clone())
            .unwrap_or_else(|| "队列过滤后符合条件的对局不足请求数量".to_string());
        result.diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::InsufficientMatchesInScope,
            AnalysisFeature::GameCount,
            message,
        ));
    }
    for diagnostic in &outcome.diagnostics {
        if diagnostic.stage != FetchStage::List {
            continue;
        }
        // 场数不足已单独映射，避免重复刷成 EvidenceExtractionFailed
        if outcome.insufficient_matches_in_scope && diagnostic.game_id == 0 {
            continue;
        }
        result.diagnostics.push(AnalysisDiagnostic::with_feature(
            AnalysisDegradationCode::EvidenceExtractionFailed,
            AnalysisFeature::DeepAnalysis,
            diagnostic.message.clone(),
        ));
    }
    dedupe_diagnostics(&mut result.diagnostics);

    // 避免完整 PUUID 进入日志；证据包也不再回传目标标识给前端
    if let Some(evidence) = result.evidence.as_mut() {
        evidence.target_puuid.clear();
    }

    log::info!(
        "[分析] puuid={} 展示 {} 场 / 深度 {} 场；列表请求 {} 次、时间线请求 {} 次（缓存命中 {} 次）",
        redact_puuid(puuid),
        result.display_games,
        result.analyzed_games,
        outcome.stats.list_requests,
        outcome.stats.timeline_requests,
        outcome.stats.timeline_cache_hits,
    );

    Ok(result)
}

fn redact_puuid(puuid: &str) -> String {
    let trimmed = puuid.trim();
    if trimmed.len() <= 8 {
        return "***".to_string();
    }
    format!("{}…{}", &trimmed[..4], &trimmed[trimmed.len() - 4..])
}

/// 执行一次完整分析（真实 LCU 客户端）
pub async fn analyze_matches_for_puuid(
    client: &Client,
    puuid: &str,
    request: &MatchAnalysisRequest,
) -> Result<MatchAnalysisResult, String> {
    ensure_static_catalogs().await?;
    analyze_matches_with_fetcher(&lcu_fetcher(client), puuid, request).await
}

/// 分析当前登录的召唤师
#[cfg(debug_assertions)]
pub async fn analyze_current_summoner(
    client: &Client,
    request: &MatchAnalysisRequest,
) -> Result<MatchAnalysisResult, String> {
    let puuid = current_summoner_puuid(client).await?;
    analyze_matches_for_puuid(client, &puuid, request).await
}

/// 读取当前召唤师 PUUID
#[cfg(debug_assertions)]
pub async fn current_summoner_puuid(client: &Client) -> Result<String, String> {
    let summoner: serde_json::Value =
        crate::shared::utils::lcu_get(client, "/lol-summoner/v1/current-summoner").await?;
    summoner
        .get("puuid")
        .and_then(|puuid| puuid.as_str())
        .filter(|puuid| !puuid.is_empty())
        .map(str::to_string)
        .ok_or_else(|| "未找到当前召唤师 PUUID".to_string())
}

// === 旧入口的请求构造与结果投影 ===

/// 由旧参数构造统一请求（默认 Deep，供个人战绩 / 位置分析）
pub fn legacy_analysis_request(
    count: u32,
    queue_id: Option<i32>,
    analysis_mode: Option<AnalysisMode>,
    perspective: Option<AdvicePerspective>,
    target_player: Option<String>,
) -> MatchAnalysisRequest {
    legacy_analysis_request_with_depth(
        count,
        queue_id,
        analysis_mode,
        perspective,
        target_player,
        AnalysisDepth::Deep,
    )
}

/// 团队/选人概览请求：强制 Simple，零时间线（避免 10 玩家 × N 时间线）
pub fn legacy_overview_request(
    count: u32,
    queue_id: Option<i32>,
    analysis_mode: Option<AnalysisMode>,
) -> MatchAnalysisRequest {
    let mut request =
        legacy_analysis_request_with_depth(count, queue_id, analysis_mode, None, None, AnalysisDepth::Simple);
    request.features.timeline = false;
    request.features.opponent = false;
    request.features.teammate = false;
    request.features.self_improvement = false;
    request.max_analysis_games = None;
    request
}

fn legacy_analysis_request_with_depth(
    count: u32,
    queue_id: Option<i32>,
    analysis_mode: Option<AnalysisMode>,
    perspective: Option<AdvicePerspective>,
    target_player: Option<String>,
    depth: AnalysisDepth,
) -> MatchAnalysisRequest {
    let (mode, queue_ids) = resolve_legacy_queue_filter(queue_id, analysis_mode);

    MatchAnalysisRequest {
        count,
        mode,
        depth,
        queue_ids,
        max_analysis_games: if depth == AnalysisDepth::Deep {
            Some(LEGACY_MAX_ANALYSIS_GAMES)
        } else {
            None
        },
        perspective,
        target_player,
        ..MatchAnalysisRequest::default()
    }
}

/// 旧的战术建议入口请求（Deep，不过滤队列）
pub fn tactical_advice_request(perspective: AdvicePerspective, target_player: Option<String>) -> MatchAnalysisRequest {
    MatchAnalysisRequest {
        count: TACTICAL_ADVICE_GAME_COUNT,
        mode: AnalysisMode::AllModes,
        depth: AnalysisDepth::Deep,
        max_analysis_games: Some(LEGACY_MAX_ANALYSIS_GAMES),
        perspective: Some(perspective),
        target_player,
        ..MatchAnalysisRequest::default()
    }
}

fn resolve_legacy_queue_filter(
    queue_id: Option<i32>,
    analysis_mode: Option<AnalysisMode>,
) -> (AnalysisMode, Option<Vec<i64>>) {
    match (analysis_mode, queue_id) {
        (Some(AnalysisMode::AllModes), Some(queue_id)) => (AnalysisMode::AllModes, Some(vec![queue_id as i64])),
        (Some(mode), _) => (mode, None),
        (None, None) => (AnalysisMode::AllModes, None),
        (None, Some(420)) => (AnalysisMode::SoloRanked, None),
        (None, Some(440)) => (AnalysisMode::FlexRanked, None),
        (None, Some(450)) => (AnalysisMode::Aram, None),
        (None, Some(queue_id)) => (AnalysisMode::AllModes, Some(vec![queue_id as i64])),
    }
}

pub fn to_player_match_stats(result: &MatchAnalysisResult) -> PlayerMatchStats {
    result.overall_stats.clone()
}

pub fn to_multi_position_analysis(result: &MatchAnalysisResult) -> MultiPositionAnalysis {
    MultiPositionAnalysis {
        position_stats: result.position_stats.clone(),
        main_position: result.main_position.clone(),
        overall_stats: result.overall_stats.clone(),
        ranked_stats: result.ranked_stats.clone(),
        other_stats: result.other_stats.clone(),
    }
}

pub fn to_legacy_advice(result: &MatchAnalysisResult, fallback_role: Option<&str>) -> Vec<GameAdvice> {
    result
        .advice
        .iter()
        .map(|advice| {
            let mut legacy = advice.to_legacy_advice();
            if legacy.affected_role.is_none() {
                legacy.affected_role = fallback_role.map(str::to_string);
            }
            legacy
        })
        .collect()
}
