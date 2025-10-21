use crate::http_client;
use crate::infrastructure::champion_selection::perks::service as perks_service;
use crate::infrastructure::data_services::external::opgg::service;
use crate::infrastructure::data_services::external::opgg::types::*;
use serde_json::Value;

#[tauri::command]
pub async fn get_opgg_champion_build_raw(
    region: String,
    mode: String,
    champion_id: i32,
    position: Option<String>,
    tier: String,
) -> Result<Value, String> {
    service::get_champion_build_raw(&region, &mode, champion_id, position, &tier).await
}

#[tauri::command]
pub async fn get_opgg_champion_build(
    region: String,
    mode: String,
    champion_id: i32,
    position: Option<String>,
    tier: String,
) -> Result<OpggChampionBuild, String> {
    service::get_champion_build(&region, &mode, champion_id, position, &tier).await
}

#[tauri::command]
pub async fn get_opgg_tier_list(region: String, mode: String, tier: String) -> Result<OpggTierList, String> {
    service::get_tier_list(&region, &mode, &tier).await
}

#[tauri::command]
pub async fn get_opgg_champion_positions(
    region: String,
    champion_id: i32,
    tier: String,
) -> Result<Vec<String>, String> {
    service::get_champion_positions(&region, champion_id, &tier).await
}

#[tauri::command]
pub async fn apply_opgg_runes(
    region: String,
    mode: String,
    champion_id: i32,
    champion_name: String,
    position: Option<String>,
    tier: String,
    build_index: Option<usize>,
) -> Result<String, String> {
    let build = service::get_champion_build(&region, &mode, champion_id, position.clone(), &tier).await?;
    let build_idx = build_index.unwrap_or(0);
    if build_idx >= build.perks.len() {
        return Err(format!(
            "详细索引 {} 超出范围，总共有 {} 个详细",
            build_idx,
            build.perks.len()
        ));
    }
    let selected_perk = &build.perks[build_idx];
    let client = http_client::get_lcu_client();
    match perks_service::apply_rune_build(
        client,
        &champion_name,
        selected_perk.primary_id,
        selected_perk.secondary_id,
        selected_perk.perks.clone(),
    )
    .await
    {
        Ok(message) => Ok(format!("OP.GG符文配置应用成功！{}", message)),
        Err(e) => Err(format!("OP.GG符文配置应用失败: {}", e)),
    }
}
