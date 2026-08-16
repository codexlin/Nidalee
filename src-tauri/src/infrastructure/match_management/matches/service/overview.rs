use reqwest::Client;
#[cfg(debug_assertions)]
use serde_json::Value;

#[cfg(debug_assertions)]
use crate::domains::analysis::analyzers::core::strategy::AnalysisMode;
use crate::shared::types::PlayerMatchStats;

/// 单次请求指定数量的原生 LCU 战绩。
///
/// 实现已统一到 [`super::super::fetcher`]，这里只做转发，避免出现第二套列表获取逻辑。
#[cfg(debug_assertions)]
pub async fn fetch_match_list(client: &Client, puuid: &str, count: usize) -> Result<Value, String> {
    super::super::fetcher::fetch_match_list(client, puuid, count).await
}

/// 获取当前玩家历史战绩统计（自动认证、统一请求、日志耗时）
/// 内部兼容入口（数据采集等）；前端请使用 `analyze_matches`。
///
/// 改为调用统一应用服务，不再自行请求战绩列表。
#[cfg(debug_assertions)]
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
