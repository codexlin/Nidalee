/// 召唤师技能数据命令层 - Tauri 命令接口
use super::service::{
    get_all_summoner_spells, get_spell_by_name, get_spell_count, get_summoner_spell_info, is_loaded, SummonerSpellInfo,
};
use crate::infrastructure::data_services::static_catalog::ensure_static_catalogs;

/// 📋 获取所有召唤师技能数据
#[tauri::command]
pub async fn get_all_summoner_spell_data() -> Result<Vec<SummonerSpellInfo>, String> {
    ensure_static_catalogs().await?;
    get_all_summoner_spells().ok_or_else(|| "获取召唤师技能数据失败".to_string())
}

/// 🔍 根据 ID 获取召唤师技能信息
#[tauri::command]
pub fn get_summoner_spell_by_id(id: i64) -> Result<Option<SummonerSpellInfo>, String> {
    if !is_loaded() {
        return Err("召唤师技能数据尚未加载".to_string());
    }

    Ok(get_summoner_spell_info(id))
}

/// 🔍 根据名称获取召唤师技能信息
#[tauri::command]
pub fn get_summoner_spell_by_name(name: String) -> Result<Option<SummonerSpellInfo>, String> {
    if !is_loaded() {
        return Err("召唤师技能数据尚未加载".to_string());
    }

    Ok(get_spell_by_name(&name))
}

/// ✅ 检查召唤师技能数据是否已加载
#[tauri::command]
pub fn is_summoner_spell_data_loaded() -> bool {
    is_loaded()
}

/// 📊 获取召唤师技能数量
#[tauri::command]
pub fn get_summoner_spell_count() -> usize {
    get_spell_count()
}
