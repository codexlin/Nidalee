use super::client::{resolve_build_position, OpggClient};
use super::parser::parse_champion_build;
use super::types::*;
use serde_json::Value;

pub async fn get_champion_build(
    region: &str,
    mode: &str,
    champion_id: i32,
    position: Option<String>,
    tier: &str,
) -> Result<OpggChampionBuild, String> {
    let client = OpggClient::new();
    let pos = resolve_build_position(mode, position.as_deref());
    let raw_data = client.get_champion_build(region, mode, champion_id, &pos, tier).await?;
    let label = if pos.is_empty() { mode } else { pos.as_str() };
    parse_champion_build(raw_data, label).map_err(|e| e.to_string())
}

pub async fn get_champion_build_raw(
    region: &str,
    mode: &str,
    champion_id: i32,
    position: Option<String>,
    tier: &str,
) -> Result<Value, String> {
    let client = OpggClient::new();
    let pos = resolve_build_position(mode, position.as_deref());
    client.get_champion_build(region, mode, champion_id, &pos, tier).await
}

pub async fn get_tier_list(region: &str, mode: &str, tier: &str) -> Result<OpggTierList, String> {
    let client = OpggClient::new();
    let raw_data = client.get_tier_list(region, mode, tier).await?;
    let meta = raw_data.get("meta").ok_or("无法获取元数据")?;
    let data = raw_data
        .get("data")
        .and_then(|v| v.as_array())
        .ok_or("无法获取层级数据")?;

    let tier_list_data: Vec<OpggTierListItem> = data.iter().filter_map(parse_tier_list_item).collect();

    let meta_data = OpggTierListMeta {
        version: meta.get("version").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        region: region.to_string(),
        mode: mode.to_string(),
        tier: tier.to_string(),
    };
    Ok(OpggTierList {
        meta: meta_data,
        data: tier_list_data,
    })
}

fn parse_tier_stats(stats: &Value) -> OpggTierStats {
    let tier_data = stats.get("tier_data");
    let tier = tier_data
        .and_then(|t| t.get("tier"))
        .and_then(|v| v.as_i64())
        .or_else(|| stats.get("tier").and_then(|v| v.as_i64()))
        .unwrap_or(0) as i32;
    let rank = tier_data
        .and_then(|t| t.get("rank"))
        .and_then(|v| v.as_i64())
        .or_else(|| stats.get("rank").and_then(|v| v.as_i64()))
        .unwrap_or(0) as i32;

    let play = stats.get("play").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let first_place = stats.get("first_place").and_then(|v| v.as_i64()).map(|v| v as i32);
    let total_place = stats.get("total_place").and_then(|v| v.as_i64()).map(|v| v as i32);

    // Arena 无 win_rate，用吃鸡率顶上便于统一排序/展示
    let win_rate = stats
        .get("win_rate")
        .and_then(|v| v.as_f64())
        .or_else(|| first_place.and_then(|fp| if play > 0 { Some(fp as f64 / play as f64) } else { None }))
        .unwrap_or(0.0);

    OpggTierStats {
        play,
        win_rate: normalize_rate(win_rate),
        pick_rate: normalize_rate(stats.get("pick_rate").and_then(|v| v.as_f64()).unwrap_or(0.0)),
        ban_rate: normalize_rate(stats.get("ban_rate").and_then(|v| v.as_f64()).unwrap_or(0.0)),
        kda: stats.get("kda").and_then(|v| v.as_f64()).unwrap_or(0.0),
        tier,
        rank,
        first_place,
        total_place,
    }
}

fn normalize_rate(v: f64) -> f64 {
    if v > 1.0 {
        v / 100.0
    } else {
        v
    }
}

fn parse_roles(item: &Value) -> Vec<OpggTierRole> {
    item.get("roles")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|role| {
                    let name = role.get("name")?.as_str()?.to_string();
                    let stats = role.get("stats")?;
                    Some(OpggTierRole {
                        name,
                        win_rate: normalize_rate(stats.get("win_rate").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                        role_rate: normalize_rate(stats.get("role_rate").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                        play: stats.get("play").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_tier_list_item(item: &Value) -> Option<OpggTierListItem> {
    // Arena 预留位：is_rip 或 average_stats 为空
    if item.get("is_rip").and_then(|v| v.as_bool()).unwrap_or(false) {
        return None;
    }
    if item.get("average_stats").map(|v| v.is_null()).unwrap_or(true) {
        return None;
    }

    let champion_id = item
        .get("id")
        .or_else(|| item.get("champion_id"))
        .and_then(|v| v.as_i64())? as i32;

    // 过滤异常占位 id（Arena 常见 600xx）
    if champion_id >= 60000 {
        return None;
    }

    let average_stats = item.get("average_stats").map(parse_tier_stats).unwrap_or_default();

    if average_stats.rank <= 0 && average_stats.play <= 0 {
        return None;
    }

    let positions = item
        .get("positions")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|pos| {
                    let name = pos.get("name")?.as_str()?.to_string();
                    let stats = pos.get("stats").map(parse_tier_stats).unwrap_or_default();
                    let counters = pos
                        .get("counters")
                        .and_then(|v| v.as_array())
                        .map(|cs| {
                            cs.iter()
                                .filter_map(|c| {
                                    Some(OpggTierCounter {
                                        champion_id: c.get("champion_id")?.as_i64()? as i32,
                                        play: c.get("play").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                        win: c.get("win").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                    })
                                })
                                .collect()
                        })
                        .unwrap_or_default();
                    Some(OpggTierPosition { name, stats, counters })
                })
                .collect()
        })
        .unwrap_or_default();

    Some(OpggTierListItem {
        champion_id,
        average_stats,
        positions,
        roles: parse_roles(item),
    })
}

pub async fn get_champion_positions(region: &str, champion_id: i32, tier: &str) -> Result<Vec<String>, String> {
    let client = OpggClient::new();
    client.get_champion_positions(region, champion_id, tier).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_ranked_item() {
        let item = json!({
            "id": 86,
            "average_stats": {
                "play": 100,
                "win_rate": 0.52,
                "pick_rate": 0.1,
                "ban_rate": 0.02,
                "kda": 2.5,
                "tier_data": { "tier": 2, "rank": 6 }
            },
            "positions": [{
                "name": "MID",
                "stats": {
                    "play": 80,
                    "win_rate": 0.53,
                    "pick_rate": 0.08,
                    "ban_rate": 0.02,
                    "kda": 2.6,
                    "tier_data": { "tier": 2, "rank": 4 }
                },
                "counters": [{ "champion_id": 103, "play": 10, "win": 6 }]
            }],
            "roles": []
        });
        let parsed = parse_tier_list_item(&item).expect("ranked");
        assert_eq!(parsed.champion_id, 86);
        assert_eq!(parsed.positions.len(), 1);
        assert!((parsed.average_stats.win_rate - 0.52).abs() < 1e-6);
    }

    #[test]
    fn parse_aram_roles_and_null_ban() {
        let item = json!({
            "id": 86,
            "average_stats": {
                "play": 100,
                "win_rate": 0.49,
                "pick_rate": 0.2,
                "ban_rate": null,
                "kda": 3.0,
                "tier_data": { "tier": 1, "rank": 3 }
            },
            "positions": null,
            "roles": [{
                "name": "TANK",
                "stats": { "win_rate": 0.5, "role_rate": 0.6, "play": 60, "win": 30 }
            }]
        });
        let parsed = parse_tier_list_item(&item).expect("aram");
        assert!(parsed.positions.is_empty());
        assert_eq!(parsed.roles.len(), 1);
        assert_eq!(parsed.roles[0].name, "TANK");
        assert_eq!(parsed.average_stats.ban_rate, 0.0);
    }

    #[test]
    fn parse_arena_chicken_rate_and_skip_rip() {
        let rip = json!({ "id": 60001, "is_rip": true, "average_stats": null });
        assert!(parse_tier_list_item(&rip).is_none());

        let item = json!({
            "id": 50,
            "average_stats": {
                "win": 2000,
                "play": 10000,
                "total_place": 40000,
                "first_place": 1580,
                "pick_rate": 0.1,
                "ban_rate": 0.05,
                "kills": 1.0,
                "assists": 2.0,
                "deaths": 1.0,
                "tier_data": { "tier": 1, "rank": 2 }
            },
            "positions": null,
            "roles": null
        });
        let parsed = parse_tier_list_item(&item).expect("arena");
        assert_eq!(parsed.average_stats.first_place, Some(1580));
        assert!((parsed.average_stats.win_rate - 0.158).abs() < 1e-6);
    }
}
