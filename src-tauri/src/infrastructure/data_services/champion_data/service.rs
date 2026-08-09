/// 英雄数据服务层 - 核心业务逻辑
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::http_client;

// 🔥 全局静态变量：英雄别名（英文名） -> ID 映射
static CHAMPION_ALIAS_TO_ID: OnceCell<HashMap<String, i32>> = OnceCell::new();

// 🔥 全局静态变量：英雄名称（中文名） -> ID 映射
static CHAMPION_NAME_TO_ID: OnceCell<HashMap<String, i32>> = OnceCell::new();

// 🔥 全局静态变量：英雄 ID -> 完整信息映射
static CHAMPION_DATA: OnceCell<HashMap<i32, ChampionInfo>> = OnceCell::new();

/// 英雄信息结构（根据实际 API 返回结构定义）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChampionInfo {
    pub id: i32,
    pub name: String,                 // 中文名称，如 "黑暗之女"
    pub description: String,          // 英雄称号，如 "安妮"
    pub alias: String,                // 英文别名，如 "Annie"
    pub content_id: String,           // 内容ID
    pub square_portrait_path: String, // 头像路径
    pub roles: Vec<String>,           // 英雄定位，如 ["mage", "support"]
}

/// 写入中文名 -> ID 映射，冲突时保留 ID 最小的本体英雄
///
/// Community Dragon 的 champion-summary 里混有特殊模式变体（如 `Jade_Annie`，id 60001），
/// 它们的中文名和称号与本体完全相同。直接 insert 会让"谁在 JSON 里排最后谁赢"，
/// 于是按名字查"安妮"可能拿到只在 JADE 模式存在的 60001。变体 ID 恒大于本体，
/// 取最小值即可稳定落在本体上，且不依赖上游数组顺序。
fn keep_canonical_champion_id(map: &mut HashMap<String, i32>, name: String, id: i32) {
    map.entry(name)
        .and_modify(|existing| {
            if id < *existing {
                *existing = id;
            }
        })
        .or_insert(id);
}

type ChampionMaps = (HashMap<String, i32>, HashMap<String, i32>, HashMap<i32, ChampionInfo>);

fn build_champion_maps(champions: Vec<ChampionInfo>) -> ChampionMaps {
    let mut alias_map = HashMap::new();
    let mut name_map = HashMap::new();
    let mut data_map = HashMap::new();

    for champ in champions {
        if champ.id < 0 {
            continue;
        }

        alias_map.insert(champ.alias.to_lowercase(), champ.id);
        keep_canonical_champion_id(&mut name_map, champ.name.clone(), champ.id);
        if !champ.description.is_empty() {
            keep_canonical_champion_id(&mut name_map, champ.description.clone(), champ.id);
        }
        data_map.insert(champ.id, champ);
    }

    (alias_map, name_map, data_map)
}

/// 从 Community Dragon 获取英雄摘要数据并构建映射
pub async fn load_champion_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 检查是否已加载
    if CHAMPION_ALIAS_TO_ID.get().is_some() && CHAMPION_NAME_TO_ID.get().is_some() && CHAMPION_DATA.get().is_some() {
        log::info!("[ChampionData] ✅ 英雄数据已加载，跳过重复加载");
        return Ok(());
    }

    log::info!("[ChampionData] 🌐 正在从 Community Dragon 加载英雄摘要数据...");

    let url =
        "https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/champion-summary.json";

    let response = http_client::get_public_client()
        .get(url)
        .send()
        .await?
        .error_for_status()?;
    let champions: Vec<ChampionInfo> = response.json().await?;

    let (alias_map, name_map, data_map) = build_champion_maps(champions);

    log::info!("[ChampionData] ✅ 英雄数据加载完成，共 {} 个英雄", alias_map.len());

    // 设置全局缓存
    CHAMPION_ALIAS_TO_ID
        .set(alias_map)
        .map_err(|_| "无法设置 CHAMPION_ALIAS_TO_ID")?;
    CHAMPION_NAME_TO_ID
        .set(name_map)
        .map_err(|_| "无法设置 CHAMPION_NAME_TO_ID")?;
    CHAMPION_DATA.set(data_map).map_err(|_| "无法设置 CHAMPION_DATA")?;

    Ok(())
}

/// 根据别名获取英雄 ID（英文名，不区分大小写）
pub fn get_champion_id_by_alias(alias: &str) -> Option<i32> {
    CHAMPION_ALIAS_TO_ID.get()?.get(&alias.to_lowercase()).copied()
}

/// 根据中文名称获取英雄 ID（支持完整名称或称号）
pub fn get_champion_id_by_name(name: &str) -> Option<i32> {
    CHAMPION_NAME_TO_ID.get()?.get(name).copied()
}

/// 根据 ID 获取英雄信息
pub fn get_champion_info(id: i32) -> Option<ChampionInfo> {
    CHAMPION_DATA.get()?.get(&id).cloned()
}

/// 根据别名获取英雄信息（英文名，不区分大小写）
pub fn get_champion_info_by_alias(alias: &str) -> Option<ChampionInfo> {
    let id = get_champion_id_by_alias(alias)?;
    get_champion_info(id)
}

/// 根据中文名称获取英雄信息（支持完整名称或称号）
pub fn get_champion_info_by_name(name: &str) -> Option<ChampionInfo> {
    let id = get_champion_id_by_name(name)?;
    get_champion_info(id)
}

/// 获取所有英雄数据（按 ID 排序）
pub fn get_all_champions() -> Option<Vec<ChampionInfo>> {
    let data = CHAMPION_DATA.get()?;
    let mut champions: Vec<ChampionInfo> = data.values().cloned().collect();
    champions.sort_by_key(|c| c.id);
    Some(champions)
}

/// 检查数据是否已加载
pub fn is_loaded() -> bool {
    CHAMPION_ALIAS_TO_ID.get().is_some() && CHAMPION_NAME_TO_ID.get().is_some() && CHAMPION_DATA.get().is_some()
}

/// 获取英雄总数
pub fn get_champion_count() -> usize {
    CHAMPION_DATA.get().map(|m| m.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---------------------------------------------------------------------
    // `keep_canonical_champion_id` 的离线用例（不联网）
    //
    // 「同名冲突时取最小 ID」是**当前上游契约假设**：Community Dragon 的
    // champion-summary 里特殊模式变体（如 `Jade_Annie`）的 ID 恒大于本体
    // （60001 > 1）。这个假设一旦被上游打破（比如给变体分配更小的 ID），
    // 下面的用例会先红——那时应该改成按 alias 前缀或模式白名单来判定本体，
    // 而不是放宽断言。
    //
    // 两个方向都要覆盖：上游数组顺序不受我们控制，实现必须与插入顺序无关。
    // ---------------------------------------------------------------------

    /// 变体先、本体后：后写入的本体（更小 ID）必须覆盖掉变体
    #[test]
    fn canonical_champion_id_keeps_base_when_variant_comes_first() {
        let mut map: HashMap<String, i32> = HashMap::new();

        // Jade_Annie（JADE 模式变体，中文名与称号和本体完全相同）
        keep_canonical_champion_id(&mut map, "黑暗之女".to_string(), 60001);
        keep_canonical_champion_id(&mut map, "安妮".to_string(), 60001);
        // 本体安妮
        keep_canonical_champion_id(&mut map, "黑暗之女".to_string(), 1);
        keep_canonical_champion_id(&mut map, "安妮".to_string(), 1);

        assert_eq!(map.get("黑暗之女").copied(), Some(1));
        assert_eq!(map.get("安妮").copied(), Some(1));
    }

    /// 本体先、变体后：已经落地的本体不能被更大 ID 的变体挤掉
    #[test]
    fn canonical_champion_id_keeps_base_when_variant_comes_last() {
        let mut map: HashMap<String, i32> = HashMap::new();

        keep_canonical_champion_id(&mut map, "黑暗之女".to_string(), 1);
        keep_canonical_champion_id(&mut map, "安妮".to_string(), 1);
        keep_canonical_champion_id(&mut map, "黑暗之女".to_string(), 60001);
        keep_canonical_champion_id(&mut map, "安妮".to_string(), 60001);

        assert_eq!(map.get("黑暗之女").copied(), Some(1));
        assert_eq!(map.get("安妮").copied(), Some(1));
    }

    /// 去重只发生在同名条目之间，不同名字互不干扰
    #[test]
    fn canonical_champion_id_keeps_distinct_names_independent() {
        let mut map: HashMap<String, i32> = HashMap::new();

        keep_canonical_champion_id(&mut map, "黑暗之女".to_string(), 1);
        keep_canonical_champion_id(&mut map, "疾风剑豪".to_string(), 157);

        assert_eq!(map.get("黑暗之女").copied(), Some(1));
        assert_eq!(map.get("疾风剑豪").copied(), Some(157));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn builds_champion_maps_without_network() {
        let champions = serde_json::from_value(serde_json::json!([
            { "id": 60001, "name": "黑暗之女", "description": "安妮", "alias": "Jade_Annie", "contentId": "variant", "squarePortraitPath": "variant.png", "roles": ["mage"] },
            { "id": 1, "name": "黑暗之女", "description": "安妮", "alias": "Annie", "contentId": "base", "squarePortraitPath": "annie.png", "roles": ["mage"] },
            { "id": -1, "name": "无", "description": "", "alias": "None", "contentId": "", "squarePortraitPath": "", "roles": [] }
        ]))
        .expect("离线英雄 fixture 应可解析");

        let (aliases, names, data) = build_champion_maps(champions);
        assert_eq!(aliases.get("annie").copied(), Some(1));
        assert_eq!(aliases.get("jade_annie").copied(), Some(60001));
        assert_eq!(names.get("黑暗之女").copied(), Some(1));
        assert_eq!(names.get("安妮").copied(), Some(1));
        assert!(!data.contains_key(&-1));
    }
}
