use crate::infrastructure::data_services::external::hextech::service;
use crate::infrastructure::data_services::external::hextech::types::{HextechChampionDetail, HextechTierList};

#[tauri::command]
pub async fn get_hextech_tier_list() -> Result<HextechTierList, String> {
    service::get_tier_list().await
}

#[tauri::command]
pub async fn get_hextech_champion_detail(champion_id: i32) -> Result<HextechChampionDetail, String> {
    service::get_champion_detail(champion_id).await
}
