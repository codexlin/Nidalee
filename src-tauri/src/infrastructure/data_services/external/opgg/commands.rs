use super::{
    service,
    types::{OpggChampionBuild, OpggTierList},
};

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
