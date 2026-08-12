use super::service::{get_all_champions, ChampionInfo};
use crate::infrastructure::data_services::static_catalog::ensure_static_catalogs;

#[tauri::command]
pub async fn get_all_champion_data() -> Result<Vec<ChampionInfo>, String> {
    ensure_static_catalogs().await?;
    get_all_champions().ok_or_else(|| "英雄目录尚未就绪".to_owned())
}
