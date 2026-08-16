use super::service;
use crate::http_client;
use crate::shared::types::SummonerInfo;

#[tauri::command]
pub async fn get_current_summoner() -> Result<SummonerInfo, String> {
    let client = http_client::get_lcu_client();
    service::get_current_summoner(client).await
}

#[tauri::command]
pub async fn get_summoners_by_names(names: Vec<String>) -> Result<Vec<SummonerInfo>, String> {
    let client = http_client::get_lcu_client();
    service::get_summoners_by_names_with_rank(client, names)
        .await
        .map_err(|error| format!("批量获取召唤师信息失败: {error}"))
}

#[tauri::command]
pub async fn set_summoner_background_skin(skin_id: u64) -> Result<(), String> {
    let client = http_client::get_lcu_client();
    service::set_summoner_background(client, skin_id).await
}

#[tauri::command]
pub async fn set_summoner_chat_profile(
    status_message: Option<String>,
    queue: Option<String>,
    tier: Option<String>,
    division: Option<String>,
) -> Result<(), String> {
    let client = http_client::get_lcu_client();
    service::set_summoner_chat_profile(client, status_message, queue, tier, division).await
}
