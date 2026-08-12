use super::service;
use crate::http_client;

#[tauri::command]
pub async fn pick_champion(action_id: u64, champion_id: u64, completed: bool) -> Result<String, String> {
    service::pick_champion(http_client::get_lcu_client(), action_id, champion_id, completed)
        .await
        .map(|()| {
            let action = if completed { "锁定" } else { "预选" };
            format!("{action}英雄成功 (ActionID: {action_id}, ChampionID: {champion_id})")
        })
}

#[tauri::command]
pub async fn ban_champion(action_id: u64, champion_id: u64) -> Result<String, String> {
    service::ban_champion(http_client::get_lcu_client(), action_id, champion_id)
        .await
        .map(|()| format!("禁用英雄成功 (ActionID: {action_id}, ChampionID: {champion_id})"))
}
