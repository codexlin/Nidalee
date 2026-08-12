use super::service::{get_all_summoner_spells, SummonerSpellInfo};
use crate::infrastructure::data_services::static_catalog::ensure_static_catalogs;

#[tauri::command]
pub async fn get_all_summoner_spell_data() -> Result<Vec<SummonerSpellInfo>, String> {
    ensure_static_catalogs().await?;
    get_all_summoner_spells().ok_or_else(|| "召唤师技能目录尚未就绪".to_owned())
}
