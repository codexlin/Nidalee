use super::types::*;
use serde_json::Value;
use std::collections::HashMap;

const MAX_AUGMENTS: usize = 40;
const MAX_COMBOS: usize = 5;

fn f64_field(v: &Value, key: &str) -> f64 {
    v.get(key).and_then(|x| x.as_f64()).unwrap_or(0.0)
}

fn i32_field(v: &Value, key: &str) -> Option<i32> {
    v.get(key)
        .and_then(|x| x.as_i64().map(|n| n as i32).or_else(|| x.as_f64().map(|n| n as i32)))
}

fn i32_field_or(v: &Value, key: &str, default: i32) -> i32 {
    i32_field(v, key).unwrap_or(default)
}

fn str_field(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}

fn string_array(v: &Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

fn i32_array(v: &Value, key: &str) -> Vec<i32> {
    v.get(key)
        .and_then(|x| x.as_array())
        .map(|arr| arr.iter().filter_map(|x| x.as_i64().map(|n| n as i32)).collect())
        .unwrap_or_default()
}

fn parse_item_combo(v: &Value) -> Option<HextechItemCombo> {
    let item_ids = i32_array(v, "itemIds");
    if item_ids.is_empty() {
        return None;
    }
    Some(HextechItemCombo {
        item_ids,
        win_rate: f64_field(v, "winRate"),
        pick_rate: f64_field(v, "pickRate"),
        games: i32_field_or(v, "games", 0),
        wins: i32_field_or(v, "wins", 0),
    })
}

fn parse_spell_combo(v: &Value) -> Option<HextechSpellCombo> {
    let spell_ids = i32_array(v, "summonerSpellIds");
    if spell_ids.is_empty() {
        return None;
    }
    Some(HextechSpellCombo {
        spell_ids,
        win_rate: f64_field(v, "winRate"),
        pick_rate: f64_field(v, "pickRate"),
        games: i32_field_or(v, "games", 0),
        wins: i32_field_or(v, "wins", 0),
    })
}

fn parse_skill_order(v: &Value) -> Option<HextechSkillOrder> {
    let skill_keys = string_array(v, "skillKeys");
    if skill_keys.is_empty() {
        return None;
    }
    Some(HextechSkillOrder {
        skill_keys,
        skill_order: i32_array(v, "skillOrder"),
        win_rate: f64_field(v, "winRate"),
        pick_rate: f64_field(v, "pickRate"),
        games: i32_field_or(v, "games", 0),
        wins: i32_field_or(v, "wins", 0),
    })
}

fn parse_augment(v: &Value, catalog: &HashMap<i32, CatalogAugment>) -> Option<HextechAugmentStat> {
    let id = i32_field(v, "id")?;
    let stats = v.get("stats").cloned().unwrap_or(Value::Null);
    let catalog_hit = catalog.get(&id);
    let name = {
        let from_row = str_field(v, "name");
        if !from_row.is_empty() {
            from_row
        } else {
            catalog_hit.map(|c| c.name.clone()).unwrap_or_default()
        }
    };
    let icon_url = {
        let from_row = str_field(v, "iconUrl");
        if !from_row.is_empty() {
            from_row
        } else {
            catalog_hit.map(|c| c.icon_url.clone()).unwrap_or_default()
        }
    };
    Some(HextechAugmentStat {
        id,
        name,
        rarity: i32_field_or(v, "rarity", catalog_hit.map(|c| c.rarity).unwrap_or(0)),
        rarity_name: {
            let s = str_field(v, "rarityName");
            if s.is_empty() {
                catalog_hit.map(|c| c.rarity_name.clone()).unwrap_or_default()
            } else {
                s
            }
        },
        rarity_display_name: {
            let s = str_field(v, "rarityDisplayName");
            if s.is_empty() {
                catalog_hit.map(|c| c.rarity_display_name.clone()).unwrap_or_default()
            } else {
                s
            }
        },
        icon_url,
        win_rate: f64_field(&stats, "winRate"),
        pick_rate: f64_field(&stats, "pickRate"),
        games: i32_field(&stats, "games"),
        wins: i32_field(&stats, "wins"),
        tier: i32_field(&stats, "tier"),
        rank: i32_field(&stats, "rank"),
    })
}

#[derive(Debug, Clone, Default)]
struct CatalogAugment {
    name: String,
    icon_url: String,
    rarity: i32,
    rarity_name: String,
    rarity_display_name: String,
}

/// 从 augments.json 建 id → 名称/图标索引（补全三连元数据）
fn parse_augment_catalog(raw: &Value) -> HashMap<i32, CatalogAugment> {
    let mut map = HashMap::new();
    let Some(rows) = raw.get("data").and_then(|v| v.as_array()) else {
        return map;
    };
    for row in rows {
        let Some(id) = i32_field(row, "id") else {
            continue;
        };
        map.insert(
            id,
            CatalogAugment {
                name: str_field(row, "name"),
                icon_url: str_field(row, "iconUrl"),
                rarity: i32_field_or(row, "rarity", 0),
                rarity_name: str_field(row, "rarityName"),
                rarity_display_name: str_field(row, "rarityDisplayName"),
            },
        );
    }
    map
}

fn parse_trio(v: &Value) -> Option<HextechAugmentTrio> {
    let augment_ids = i32_array(v, "augmentIds");
    if augment_ids.is_empty() {
        return None;
    }
    let stats = v.get("stats").cloned().unwrap_or(Value::Null);
    Some(HextechAugmentTrio {
        augment_ids,
        win_rate: f64_field(&stats, "winRate").max(f64_field(v, "winRate")),
        pick_rate: f64_field(&stats, "pickRate").max(f64_field(v, "pickRate")),
        games: i32_field(&stats, "games").or_else(|| i32_field(v, "games")),
        wins: i32_field(&stats, "wins"),
    })
}

/// 解析英雄强度榜 JSON
pub fn parse_tier_list(raw: Value) -> Result<HextechTierList, String> {
    let data_version = str_field(&raw, "dataVersion");
    let region = raw
        .get("locale")
        .and_then(|v| v.as_str())
        .unwrap_or("zh-CN")
        .to_string();
    let rows = raw
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or("champions.json 缺少 data")?;

    let mut items: Vec<HextechTierListItem> = rows
        .iter()
        .filter_map(|row| {
            let champion_id = i32_field(row, "id")?;
            let stats = row.get("stats").cloned().unwrap_or(Value::Null);
            Some(HextechTierListItem {
                champion_id,
                name: str_field(row, "name"),
                alias: str_field(row, "alias"),
                icon_url: str_field(row, "iconUrl"),
                roles: string_array(row, "roles"),
                win_rate: f64_field(&stats, "winRate"),
                pick_rate: f64_field(&stats, "pickRate"),
                tier: i32_field(&stats, "tier"),
                rank: 0,
            })
        })
        .collect();

    items.sort_by(|a, b| {
        b.win_rate
            .partial_cmp(&a.win_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| {
                b.pick_rate
                    .partial_cmp(&a.pick_rate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    });
    for (idx, item) in items.iter_mut().enumerate() {
        item.rank = (idx + 1) as i32;
    }

    let game_patch = items
        .first()
        .and_then(|_| rows.first())
        .and_then(|row| row.get("stats"))
        .and_then(|s| s.get("gamePatch"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(HextechTierList {
        data_version,
        game_patch,
        region,
        data: items,
    })
}

/// 解析英雄详情 JSON（可附带 augments 目录补全描述）
pub fn parse_champion_detail(raw: Value) -> Result<HextechChampionDetail, String> {
    parse_champion_detail_with_catalog(raw, &HashMap::new())
}

/// 详情 + augments.json 合并（补全三连名称/图标）
pub fn parse_champion_detail_merging_catalog(
    detail: Value,
    catalog_raw: &Value,
) -> Result<HextechChampionDetail, String> {
    let catalog = parse_augment_catalog(catalog_raw);
    parse_champion_detail_with_catalog(detail, &catalog)
}

fn parse_champion_detail_with_catalog(
    raw: Value,
    catalog: &HashMap<i32, CatalogAugment>,
) -> Result<HextechChampionDetail, String> {
    let data_version = str_field(&raw, "dataVersion");
    let champion = raw.get("champion").ok_or("详情缺少 champion")?;
    let champion_id = i32_field(champion, "id")
        .or_else(|| i32_field(&raw, "championId"))
        .ok_or("详情缺少 championId")?;
    let stats = champion.get("stats").cloned().unwrap_or(Value::Null);

    let summary = HextechChampionSummary {
        champion_id,
        name: str_field(champion, "name"),
        alias: str_field(champion, "alias"),
        title: str_field(champion, "title"),
        icon_url: str_field(champion, "iconUrl"),
        roles: string_array(champion, "roles"),
        win_rate: f64_field(&stats, "winRate"),
        pick_rate: f64_field(&stats, "pickRate"),
        tier: i32_field(&stats, "tier"),
        data_version: data_version.clone(),
        game_patch: stats.get("gamePatch").and_then(|v| v.as_str()).map(|s| s.to_string()),
    };

    let mut augments: Vec<HextechAugmentStat> = raw
        .get("augments")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|row| parse_augment(row, catalog)).collect())
        .unwrap_or_default();
    augments.sort_by(|a, b| b.win_rate.partial_cmp(&a.win_rate).unwrap_or(std::cmp::Ordering::Equal));

    let mut augment_trios: Vec<HextechAugmentTrio> = raw
        .get("augmentTrios")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_trio).collect())
        .unwrap_or_default();
    augment_trios.truncate(MAX_COMBOS);

    // 保证三连里的增强仍保留元数据（图标/名称），即便未进 Top N
    let keep_ids: std::collections::HashSet<i32> = augment_trios
        .iter()
        .flat_map(|t| t.augment_ids.iter().copied())
        .collect();
    let mut kept: Vec<HextechAugmentStat> = Vec::with_capacity(MAX_AUGMENTS + keep_ids.len());
    let mut kept_ids = std::collections::HashSet::new();
    for aug in augments.iter().take(MAX_AUGMENTS) {
        kept_ids.insert(aug.id);
        kept.push(aug.clone());
    }
    for aug in augments {
        if keep_ids.contains(&aug.id) && !kept_ids.contains(&aug.id) {
            kept_ids.insert(aug.id);
            kept.push(aug);
        }
    }
    for id in keep_ids {
        if kept_ids.contains(&id) {
            continue;
        }
        if let Some(meta) = catalog.get(&id) {
            kept_ids.insert(id);
            kept.push(HextechAugmentStat {
                id,
                name: meta.name.clone(),
                rarity: meta.rarity,
                rarity_name: meta.rarity_name.clone(),
                rarity_display_name: meta.rarity_display_name.clone(),
                icon_url: meta.icon_url.clone(),
                win_rate: 0.0,
                pick_rate: 0.0,
                games: None,
                wins: None,
                tier: None,
                rank: None,
            });
        }
    }
    let augments = kept;

    let build = raw.get("build").cloned().unwrap_or(Value::Null);
    let summoner_spells: Vec<HextechSpellCombo> = build
        .get("summonerSpells")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_spell_combo).take(MAX_COMBOS).collect())
        .unwrap_or_default();
    let skill_orders: Vec<HextechSkillOrder> = build
        .get("skillOrders")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_skill_order).take(MAX_COMBOS).collect())
        .unwrap_or_default();
    let starting_items: Vec<HextechItemCombo> = build
        .get("startingItems")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_item_combo).take(MAX_COMBOS).collect())
        .unwrap_or_default();
    let core_items: Vec<HextechItemCombo> = build
        .get("coreItems")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(parse_item_combo).take(MAX_COMBOS).collect())
        .unwrap_or_default();

    Ok(HextechChampionDetail {
        summary,
        augments,
        augment_trios,
        summoner_spells,
        skill_orders,
        starting_items,
        core_items,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_tier_list_sorted_by_win_rate() {
        let raw = json!({
            "dataVersion": "16.15.6",
            "locale": "zh-CN",
            "data": [
                {
                    "id": 1,
                    "name": "A",
                    "alias": "A",
                    "iconUrl": "a.png",
                    "roles": ["mage"],
                    "stats": { "winRate": 0.4, "pickRate": 0.1, "tier": 3, "gamePatch": "16.15" }
                },
                {
                    "id": 2,
                    "name": "B",
                    "alias": "B",
                    "iconUrl": "b.png",
                    "roles": ["fighter"],
                    "stats": { "winRate": 0.6, "pickRate": 0.2, "tier": 1, "gamePatch": "16.15" }
                }
            ]
        });
        let list = parse_tier_list(raw).unwrap();
        assert_eq!(list.data.len(), 2);
        assert_eq!(list.data[0].champion_id, 2);
        assert_eq!(list.data[0].rank, 1);
        assert_eq!(list.data[1].rank, 2);
        assert_eq!(list.game_patch.as_deref(), Some("16.15"));
    }

    #[test]
    fn parses_champion_detail_core_fields() {
        let raw = json!({
            "dataVersion": "16.15.6",
            "championId": 157,
            "champion": {
                "id": 157,
                "alias": "Yasuo",
                "name": "亚索",
                "title": "疾风剑豪",
                "roles": ["fighter"],
                "iconUrl": "https://cdn.example/157.png",
                "stats": { "tier": 2, "winRate": 0.55, "pickRate": 0.1, "gamePatch": "16.15" }
            },
            "augments": [{
                "id": 1001,
                "name": "泰坦的坚决",
                "rarity": 2,
                "rarityName": "prismatic",
                "rarityDisplayName": "棱彩",
                "iconUrl": "https://cdn.example/a.png",
                "stats": { "winRate": 0.53, "pickRate": 0.01, "games": 100, "wins": 53, "tier": 2, "rank": 1 }
            }],
            "augmentTrios": [{
                "augmentIds": [1001, 1002, 1003],
                "stats": { "winRate": 0.7, "pickRate": 0.01, "games": 50, "wins": 35 }
            }],
            "build": {
                "summonerSpells": [{
                    "summonerSpellIds": [4, 32],
                    "games": 10, "wins": 6, "pickRate": 0.8, "winRate": 0.6
                }],
                "skillOrders": [{
                    "skillOrder": [1, 2, 3],
                    "skillKeys": ["Q", "W", "E"],
                    "games": 10, "wins": 6, "pickRate": 0.5, "winRate": 0.6
                }],
                "startingItems": [{
                    "itemIds": [1055],
                    "games": 10, "wins": 6, "pickRate": 0.5, "winRate": 0.6
                }],
                "coreItems": [{
                    "itemIds": [3006, 3153],
                    "games": 10, "wins": 6, "pickRate": 0.4, "winRate": 0.6
                }]
            }
        });
        let catalog = json!({
            "data": [
                { "id": 1001, "name": "泰坦的坚决", "iconUrl": "a.png", "rarity": 2, "rarityName": "prismatic", "rarityDisplayName": "棱彩" },
                { "id": 1002, "name": "增强二", "iconUrl": "b.png", "rarity": 1, "rarityName": "gold", "rarityDisplayName": "黄金" },
                { "id": 1003, "name": "增强三", "iconUrl": "c.png", "rarity": 0, "rarityName": "silver", "rarityDisplayName": "白银" }
            ]
        });
        let detail = parse_champion_detail_merging_catalog(raw, &catalog).unwrap();
        assert_eq!(detail.summary.champion_id, 157);
        assert!(detail.augments.len() >= 3);
        assert_eq!(detail.augments[0].name, "泰坦的坚决");
        assert!(detail.augments.iter().any(|a| a.id == 1002 && a.name == "增强二"));
        assert_eq!(detail.augment_trios[0].augment_ids.len(), 3);
        assert_eq!(detail.summoner_spells[0].spell_ids, vec![4, 32]);
        assert_eq!(detail.skill_orders[0].skill_keys[0], "Q");
        assert_eq!(detail.core_items[0].item_ids.len(), 2);
    }
}
