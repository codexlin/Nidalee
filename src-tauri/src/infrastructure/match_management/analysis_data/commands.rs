#[tauri::command]
pub async fn retry_player_analysis(puuid: String) -> Result<crate::shared::types::PlayerAnalysisStatus, String> {
    let event_handler = crate::infrastructure::real_time::websocket::service::get_event_handler()
        .ok_or_else(|| "LCU WebSocket has not been started".to_string())?;
    event_handler
        .retry_player_analysis(&puuid)
        .await
        .map_err(|error| error.to_string())
}
