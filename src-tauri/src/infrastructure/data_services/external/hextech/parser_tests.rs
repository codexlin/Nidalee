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
