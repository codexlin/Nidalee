use crate::shared::types::TeamAnalysisData;

#[tauri::command]
pub async fn get_cached_analysis_data() -> Result<Option<TeamAnalysisData>, String> {
    log::info!(
        target: "analysis_data::commands",
        "Frontend requested cached analysis data"
    );

    // 从全局获取事件处理器
    let event_handler = match crate::infrastructure::real_time::websocket::service::get_event_handler() {
        Some(handler) => handler,
        None => {
            log::warn!(
                target: "analysis_data::commands",
                "WebSocket event handler not initialized"
            );
            return Ok(None);
        }
    };

    // 1. 首先尝试从内存缓存获取
    if let Some(data) = event_handler.get_cached_team_analysis_data().await {
        log::info!(
            target: "analysis_data::commands",
            "Returning cached data: my_team={}, enemy_team={}",
            data.my_team.len(),
            data.enemy_team.len()
        );
        return Ok(Some(data));
    }

    // 2. 没有缓存，直接返回 None
    // 注意：不在这里主动构建，由 WebSocket 的 phase-change 事件自动触发
    log::debug!(
        target: "analysis_data::commands",
        "No cached data available, returning None (will be built by phase-change event)"
    );

    Ok(None)
}

#[tauri::command]
pub async fn retry_player_analysis(puuid: String) -> Result<crate::shared::types::PlayerAnalysisStatus, String> {
    let event_handler = crate::infrastructure::real_time::websocket::service::get_event_handler()
        .ok_or_else(|| "LCU WebSocket has not been started".to_string())?;
    event_handler
        .retry_player_analysis(&puuid)
        .await
        .map_err(|error| error.to_string())
}
