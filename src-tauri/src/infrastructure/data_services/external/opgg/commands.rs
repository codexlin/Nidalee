use super::{
    service,
    types::{OpggChampionBuild, OpggTierList},
};
use crate::http_client;
use crate::infrastructure::champion_selection::perks::service as perks_service;

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
pub async fn apply_opgg_runes(
    region: String,
    mode: String,
    champion_id: i32,
    champion_name: String,
    position: Option<String>,
    tier: String,
    build_index: Option<usize>,
) -> Result<String, String> {
    let build = service::get_champion_build(&region, &mode, champion_id, position, &tier).await?;
    let index = build_index.unwrap_or_default();
    let selected = build
        .perks
        .get(index)
        .ok_or_else(|| format!("符文方案索引 {index} 超出范围，共 {} 个方案", build.perks.len()))?;

    perks_service::apply_rune_build(
        http_client::get_lcu_client(),
        &champion_name,
        selected.primary_id,
        selected.secondary_id,
        selected.perks.clone(),
    )
    .await
}
