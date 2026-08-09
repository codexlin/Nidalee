/// 召唤师技能数据服务层 - 核心业务逻辑
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::http_client;

// 🔥 全局静态变量：召唤师技能 ID -> 完整信息映射
static SUMMONER_SPELL_DATA: OnceCell<HashMap<i64, SummonerSpellInfo>> = OnceCell::new();

// 🔥 全局静态变量：召唤师技能名称 -> ID 映射
static SUMMONER_SPELL_NAME_TO_ID: OnceCell<HashMap<String, i64>> = OnceCell::new();

/// 召唤师技能信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummonerSpellInfo {
    pub id: i64, // 改为 i64 以支持大数值（API 可能返回 4294967295）
    pub name: String,
    pub description: String,
    pub summoner_level: i64, // 改为 i64
    pub cooldown: i64,       // 改为 i64
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

/// 从 Community Dragon 获取召唤师技能数据并构建映射
pub async fn load_summoner_spell_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 检查是否已加载
    if SUMMONER_SPELL_DATA.get().is_some() {
        log::info!("[SummonerSpells] ✅ 召唤师技能数据已加载，跳过重复加载");
        return Ok(());
    }

    log::info!("[SummonerSpells] 🌐 正在从 Community Dragon 加载召唤师技能数据...");

    let url =
        "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/summoner-spells.json";

    let response = http_client::get_public_client()
        .get(url)
        .send()
        .await?
        .error_for_status()?;
    let spells: Vec<SummonerSpellInfo> = response.json().await?;

    let (data_map, name_map) = build_summoner_spell_maps(spells);

    log::info!(
        "[SummonerSpells] ✅ 召唤师技能数据加载完成，共 {} 个技能",
        data_map.len()
    );

    // 设置全局缓存
    SUMMONER_SPELL_DATA
        .set(data_map)
        .map_err(|_| "无法设置 SUMMONER_SPELL_DATA")?;
    SUMMONER_SPELL_NAME_TO_ID
        .set(name_map)
        .map_err(|_| "无法设置 SUMMONER_SPELL_NAME_TO_ID")?;

    Ok(())
}

/// 根据 ID 获取召唤师技能信息
pub fn get_summoner_spell_info(id: i64) -> Option<SummonerSpellInfo> {
    SUMMONER_SPELL_DATA.get()?.get(&id).cloned()
}

/// 获取所有召唤师技能数据（按 ID 排序）
pub fn get_all_summoner_spells() -> Option<Vec<SummonerSpellInfo>> {
    let data = SUMMONER_SPELL_DATA.get()?;
    let mut spells: Vec<SummonerSpellInfo> = data.values().cloned().collect();
    spells.sort_by_key(|s| s.id);
    Some(spells)
}

/// 检查数据是否已加载
pub fn is_loaded() -> bool {
    SUMMONER_SPELL_DATA.get().is_some() && SUMMONER_SPELL_NAME_TO_ID.get().is_some()
}

/// 获取召唤师技能总数
pub fn get_spell_count() -> usize {
    SUMMONER_SPELL_DATA.get().map(|m| m.len()).unwrap_or(0)
}

/// 根据名称查找召唤师技能 ID（支持中文）
pub fn get_spell_id_by_name(name: &str) -> Option<i64> {
    SUMMONER_SPELL_NAME_TO_ID.get()?.get(name).copied()
}

/// 根据名称查找召唤师技能（支持中文）
pub fn get_spell_by_name(name: &str) -> Option<SummonerSpellInfo> {
    let id = get_spell_id_by_name(name)?;
    get_summoner_spell_info(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------
    // `keep_canonical_spell_id` 的离线用例（不联网）
    //
    // 「同名冲突时取最小 ID」是**当前上游契约假设**：通用本体技能（"引燃" id 14，
    // 覆盖 CLASSIC/ARAM 等模式）的 ID 恒小于只服务单一模式的变体（JADE 专用 id 714）。
    // 上游若不再遵守这个假设，下面的用例会先红——那时应该改成按 `game_modes`
    // 覆盖面来判定本体，而不是放宽断言。
    //
    // 两个方向都要覆盖：上游数组顺序不受我们控制，实现必须与插入顺序无关。
    // ---------------------------------------------------------------------

    /// 变体先、本体后：后写入的本体（更小 ID）必须覆盖掉变体
    #[test]
    fn canonical_spell_id_keeps_base_when_variant_comes_first() {
        let mut map: HashMap<String, i64> = HashMap::new();

        // JADE 模式专用的"引燃"
        keep_canonical_spell_id(&mut map, "引燃".to_string(), 714);
        // 通用本体"引燃"
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
