//! LiveClient 数据相关的 Tauri 命令
//!
//! 这层只做参数转发和 JSON 序列化，业务实现在同模块的 `service` 中。

use crate::infrastructure::real_time::liveclient;

#[tauri::command]
pub async fn get_live_player_list() -> Result<String, String> {
    let players = liveclient::get_live_player_list().await?;
    let json = serde_json::to_string(&players).map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
pub async fn get_live_events() -> Result<String, String> {
    let events = liveclient::get_live_events().await?;
    let json = serde_json::to_string(&events).map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
pub async fn get_game_stats() -> Result<String, String> {
    let stats = liveclient::get_game_stats().await?;
    let json = serde_json::to_string(&stats).map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
pub async fn is_liveclient_available() -> bool {
    liveclient::is_liveclient_available().await
}
