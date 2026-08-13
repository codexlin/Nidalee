use super::client::{resolve_build_position, OpggClient};
use super::parser::parse_champion_build;
use super::types::*;
use once_cell::sync::Lazy;
use serde_json::Value;

static OPGG_CLIENT: Lazy<OpggClient> =
    Lazy::new(|| OpggClient::with_client(crate::http_client::get_public_client().clone()));

const REGIONS: &[&str] = &["global", "kr", "na"];
const MODES: &[&str] = &["ranked", "aram", "urf", "arena"];
const TIERS: &[&str] = &[
    "emerald_plus",
    "platinum_plus",
    "diamond_plus",
    "all",
    "iron",
    "bronze",
    "silver",
    "gold",
    "platinum",
    "emerald",
    "diamond",
    "master",
    "grandmaster",
    "challenger",
];

fn validate_request(region: &str, mode: &str, champion_id: Option<i32>, tier: &str) -> Result<(), String> {
    if !REGIONS.contains(&region) {
        return Err(format!("不支持的 OP.GG 区域: {region}"));
    }
    if !MODES.contains(&mode) {
        return Err(format!("不支持的 OP.GG 模式: {mode}"));
    }
    if champion_id.is_some_and(|id| id <= 0) {
        return Err("英雄 ID 必须大于 0".to_string());
    }
    if mode == "ranked" && !TIERS.contains(&tier) {
        return Err(format!("不支持的 OP.GG 段位筛选: {tier}"));
    }
    Ok(())
}

pub async fn get_champion_build(
    region: &str,
    mode: &str,
    champion_id: i32,
    position: Option<String>,
    tier: &str,
) -> Result<OpggChampionBuild, String> {
    validate_request(region, mode, Some(champion_id), tier)?;
    let pos = resolve_build_position(mode, position.as_deref())?;
    let raw_data = OPGG_CLIENT
        .get_champion_build(region, mode, champion_id, &pos, tier)
        .await?;
    let label = if pos.is_empty() { mode } else { pos.as_str() };
    parse_champion_build(raw_data, label).map_err(|e| e.to_string())
}

pub async fn get_tier_list(region: &str, mode: &str, tier: &str) -> Result<OpggTierList, String> {
    validate_request(region, mode, None, tier)?;
    let raw_data = OPGG_CLIENT.get_tier_list(region, mode, tier).await?;
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

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
