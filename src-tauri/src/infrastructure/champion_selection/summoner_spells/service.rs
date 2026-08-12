/// 召唤师技能数据服务层 - 核心业务逻辑
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;
use ts_rs::TS;

use crate::http_client;

struct SpellStore {
    data: HashMap<i64, SummonerSpellInfo>,
    name_to_id: HashMap<String, i64>,
}

static SPELL_STORE: Lazy<RwLock<Option<SpellStore>>> = Lazy::new(|| RwLock::new(None));

/// 召唤师技能信息结构
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/SummonerSpellInfo.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct SummonerSpellInfo {
    #[ts(type = "number")]
    pub id: i64, // 改为 i64 以支持大数值（API 可能返回 4294967295）
    pub name: String,
    pub description: String,
    #[ts(type = "number")]
    pub summoner_level: i64, // 改为 i64
    #[ts(type = "number")]
    pub cooldown: i64, // 改为 i64
    pub game_modes: Vec<String>,
    pub icon_path: String,
}

/// 写入技能名 -> ID 映射，冲突时保留 ID 最小的本体技能
///
/// 上游数据里同一个技能名会出现多份：除了通用的本体（如"引燃" id 14，覆盖
/// CLASSIC/ARAM 等模式），还有只服务单一模式的变体（如 JADE 专用的 id 714）。
/// 直接 insert 等于让上游数组顺序决定结果，取最小 ID 才能稳定命中本体。
fn keep_canonical_spell_id(map: &mut HashMap<String, i64>, name: String, id: i64) {
    map.entry(name)
        .and_modify(|existing| {
            if id < *existing {
                *existing = id;
            }
        })
        .or_insert(id);
}

fn build_summoner_spell_maps(
    spells: Vec<SummonerSpellInfo>,
) -> (HashMap<i64, SummonerSpellInfo>, HashMap<String, i64>) {
    let mut data_map = HashMap::new();
    let mut name_map = HashMap::new();

    for spell in spells {
        if spell.id == -1 || spell.id == 4294967295 || spell.name.is_empty() {
            continue;
        }

        data_map.insert(spell.id, spell.clone());
        keep_canonical_spell_id(&mut name_map, spell.name.clone(), spell.id);
    }

    (data_map, name_map)
}

/// 安装/替换内存中的召唤师技能目录（供 static_catalog 编排）
pub fn install_summoner_spell_maps(spells: Vec<SummonerSpellInfo>) -> Result<(), String> {
    let (data, name_to_id) = build_summoner_spell_maps(spells);
    let count = data.len();
    let mut guard = SPELL_STORE.write().map_err(|e| format!("召唤师技能目录锁中毒: {e}"))?;
    *guard = Some(SpellStore { data, name_to_id });
    log::info!("[SummonerSpells] ✅ 召唤师技能目录已安装，共 {count} 个");
    Ok(())
}

/// 从 Community Dragon 拉取原始技能列表（不写内存）
pub async fn fetch_summoner_spell_data_from_network(
) -> Result<Vec<SummonerSpellInfo>, Box<dyn std::error::Error + Send + Sync>> {
    log::info!("[SummonerSpells] 🌐 正在从 Community Dragon 拉取召唤师技能...");
    let url =
        "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/summoner-spells.json";

    let response = http_client::get_public_client()
        .get(url)
        .send()
        .await?
        .error_for_status()?;
    let spells: Vec<SummonerSpellInfo> = response.json().await?;
    Ok(spells)
}

/// 获取所有召唤师技能数据（按 ID 排序）
pub fn get_all_summoner_spells() -> Option<Vec<SummonerSpellInfo>> {
    let guard = SPELL_STORE.read().ok()?;
    let data = &guard.as_ref()?.data;
    let mut spells: Vec<SummonerSpellInfo> = data.values().cloned().collect();
    spells.sort_by_key(|s| s.id);
    Some(spells)
}

/// 检查数据是否已加载
pub fn is_loaded() -> bool {
    SPELL_STORE.read().map(|g| g.is_some()).unwrap_or(false)
}

/// 获取召唤师技能总数
pub fn get_spell_count() -> usize {
    SPELL_STORE
        .read()
        .ok()
        .and_then(|g| g.as_ref().map(|s| s.data.len()))
        .unwrap_or(0)
}

/// 根据名称查找召唤师技能 ID（支持中文）
pub fn get_spell_id_by_name(name: &str) -> Option<i64> {
    let guard = SPELL_STORE.read().ok()?;
    guard.as_ref()?.name_to_id.get(name).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 变体先、本体后：后写入的本体（更小 ID）必须覆盖掉变体
    #[test]
    fn canonical_spell_id_keeps_base_when_variant_comes_first() {
        let mut map: HashMap<String, i64> = HashMap::new();

        keep_canonical_spell_id(&mut map, "引燃".to_string(), 714);
        keep_canonical_spell_id(&mut map, "引燃".to_string(), 14);

        assert_eq!(map.get("引燃").copied(), Some(14));
    }

    /// 本体先、变体后：已经落地的本体不能被更大 ID 的变体挤掉
    #[test]
    fn canonical_spell_id_keeps_base_when_variant_comes_last() {
        let mut map: HashMap<String, i64> = HashMap::new();

        keep_canonical_spell_id(&mut map, "引燃".to_string(), 14);
        keep_canonical_spell_id(&mut map, "引燃".to_string(), 714);

        assert_eq!(map.get("引燃").copied(), Some(14));
    }

    /// 去重只发生在同名条目之间，不同名字互不干扰
    #[test]
    fn canonical_spell_id_keeps_distinct_names_independent() {
        let mut map: HashMap<String, i64> = HashMap::new();

        keep_canonical_spell_id(&mut map, "闪现".to_string(), 4);
        keep_canonical_spell_id(&mut map, "惩戒".to_string(), 11);

        assert_eq!(map.get("闪现").copied(), Some(4));
        assert_eq!(map.get("惩戒").copied(), Some(11));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn builds_spell_maps_without_network() {
        let spells = serde_json::from_value(serde_json::json!([
            { "id": 4, "name": "闪现", "description": "", "summonerLevel": 1, "cooldown": 300, "gameModes": ["CLASSIC"], "iconPath": "flash.png" },
            { "id": 714, "name": "引燃", "description": "", "summonerLevel": 1, "cooldown": 1, "gameModes": ["JADE"], "iconPath": "jade-ignite.png" },
            { "id": 14, "name": "引燃", "description": "", "summonerLevel": 1, "cooldown": 180, "gameModes": ["CLASSIC"], "iconPath": "ignite.png" },
            { "id": -1, "name": "无效", "description": "", "summonerLevel": 0, "cooldown": 0, "gameModes": [], "iconPath": "" }
        ]))
        .expect("离线召唤师技能 fixture 应可解析");

        let (data, names) = build_summoner_spell_maps(spells);
        assert_eq!(data.len(), 3);
        assert_eq!(data.get(&4).map(|spell| spell.name.as_str()), Some("闪现"));
        assert_eq!(names.get("引燃").copied(), Some(14));
        assert!(!data.contains_key(&-1));
    }
}
