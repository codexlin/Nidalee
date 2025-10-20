use crate::http_client;
use crate::shared::types::{PlayerMatchStats, ChampSelectSession};
use std::collections::HashMap;
use super::service;

#[tauri::command]
pub async fn get_champselect_team_players_info() -> Result<HashMap<String, PlayerMatchStats>, String> {
    let client = http_client::get_lcu_client();
    service::get_champselect_team_players_info(client).await
}

#[tauri::command]
pub async fn get_champ_select_session() -> Result<serde_json::Value, String> {
    let client = http_client::get_lcu_client();
    service::get_champ_select_session_raw(client).await
}

#[tauri::command]
pub async fn get_champ_select_session_typed() -> Result<ChampSelectSession, String> {
    let client = http_client::get_lcu_client();
    service::get_champ_select_session(client).await
}

#[tauri::command]
pub async fn pick_champion(action_id: u64, champion_id: u64, completed: bool) -> Result<String, String> {
    let client = http_client::get_lcu_client();
    match service::pick_champion(client, action_id, champion_id, completed).await {
        Ok(()) => {
            let action_type = if completed { "锁定" } else { "预选" };
            let message = format!(
                "{}英雄成功 (ActionID: {}, ChampionID: {})",
                action_type, action_id, champion_id
            );
            log::info!("[Commands] {}", message);
            Ok(message)
        }
        Err(e) => {
            let action_type = if completed { "锁定" } else { "预选" };
            let error_msg = format!("{}英雄失败: {}", action_type, e);
            log::error!("[Commands] {}", error_msg);
            Err(error_msg)
        }
    }
}

#[tauri::command]
pub async fn ban_champion(action_id: u64, champion_id: u64) -> Result<String, String> {
    let client = http_client::get_lcu_client();
    match service::ban_champion(client, action_id, champion_id).await {
        Ok(()) => {
            let message = format!("禁用英雄成功 (ActionID: {}, ChampionID: {})", action_id, champion_id);
            log::info!("[Commands] {}", message);
            Ok(message)
        }
        Err(e) => {
            let error_msg = format!("禁用英雄失败: {}", e);
            log::error!("[Commands] {}", error_msg);
            Err(error_msg)
        }
    }
}
