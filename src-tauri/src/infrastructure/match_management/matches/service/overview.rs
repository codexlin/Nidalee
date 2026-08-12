use reqwest::Client;
use serde_json::Value;

use crate::domains::analysis::analyzers::core::strategy::AnalysisMode;
use crate::shared::types::{AdvicePerspective, PlayerMatchStats};

/// 单次请求指定数量的原生 LCU 战绩。
///
/// 实现已统一到 [`super::super::fetcher`]，这里只做转发，避免出现第二套列表获取逻辑。
pub async fn fetch_match_list(client: &Client, puuid: &str, count: usize) -> Result<Value, String> {
    super::super::fetcher::fetch_match_list(client, puuid, count).await
}

/// 获取当前玩家历史战绩统计（自动认证、统一请求、日志耗时）
/// 内部兼容入口（数据采集等）；前端请使用 `analyze_matches`。
///
/// 改为调用统一应用服务，不再自行请求战绩列表。
pub(crate) async fn get_match_history(
    client: &Client,
    end_count: usize,
    queue_id: Option<i32>,
    analysis_mode: Option<AnalysisMode>,
) -> Result<PlayerMatchStats, String> {
    let request =
        super::super::analysis_service::legacy_analysis_request(end_count as u32, queue_id, analysis_mode, None, None);
    let result = super::super::analysis_service::analyze_current_summoner(client, &request).await?;

    log::info!(
        "[战绩] 展示 {} 场，胜率 {:.1}%，深度证据 {} 场",
        result.overall_stats.total_games,
        result.overall_stats.win_rate,
        result.analyzed_games
    );

    Ok(super::super::analysis_service::to_player_match_stats(&result))
}

/// 获取指定召唤师最近几场简单战绩
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `puuid`: 玩家 PUUID
/// - `count`: 获取对局数量
/// - `queue_id`: 可选的队列ID，用于过滤（如 420=单排, 440=灵活排）
/// - `perspective`: 建议视角（默认SelfImprovement）
/// - `target_name`: 目标玩家名称（用于建议措辞）
pub async fn get_recent_matches_by_puuid(
    client: &Client,
    puuid: &str,
    count: usize,
    queue_id: Option<i32>,
) -> Result<PlayerMatchStats, String> {
    get_recent_matches_by_puuid_with_perspective(
        client,
        puuid,
        count,
        queue_id,
        AdvicePerspective::SelfImprovement,
        None,
    )
    .await
}

/// 获取指定召唤师最近几场战绩（支持指定建议视角）
///
/// 兼容适配器：团队/选人概览路径，强制 Simple（零时间线），
/// 避免 10 玩家 × N 时间线把客户端拖垮。个人深度分析走 `analyze_matches`。
pub async fn get_recent_matches_by_puuid_with_perspective(
    client: &Client,
    puuid: &str,
    count: usize,
    queue_id: Option<i32>,
    perspective: AdvicePerspective,
    target_name: Option<String>,
) -> Result<PlayerMatchStats, String> {
    let mut request = super::super::analysis_service::legacy_overview_request(count as u32, queue_id, None);
    request.perspective = Some(perspective);
    request.target_player = target_name;
    let result = super::super::analysis_service::analyze_matches_for_puuid(client, puuid, &request).await?;
    Ok(super::super::analysis_service::to_player_match_stats(&result))
}
