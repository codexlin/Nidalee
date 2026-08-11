use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 海克斯英雄摘要
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/HextechChampionSummary.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct HextechChampionSummary {
    pub champion_id: i32,
    pub name: String,
    pub alias: String,
    pub title: String,
    pub icon_url: String,
    pub roles: Vec<String>,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub tier: Option<i32>,
    pub data_version: String,
    pub game_patch: Option<String>,
}

/// 增强条目（英雄侧统计）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/HextechAugmentStat.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct HextechAugmentStat {
    pub id: i32,
    pub name: String,
    pub rarity: i32,
    pub rarity_name: String,
    pub rarity_display_name: String,
    pub icon_url: String,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub games: Option<i32>,
    pub wins: Option<i32>,
    pub tier: Option<i32>,
    pub rank: Option<i32>,
}

/// 推荐增强三连
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/HextechAugmentTrio.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct HextechAugmentTrio {
    pub augment_ids: Vec<i32>,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub games: Option<i32>,
    pub wins: Option<i32>,
}

/// 召唤师技能组合
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/HextechSpellCombo.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct HextechSpellCombo {
    pub spell_ids: Vec<i32>,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub games: i32,
    pub wins: i32,
}

/// 技能加点
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/HextechSkillOrder.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct HextechSkillOrder {
    pub skill_keys: Vec<String>,
    pub skill_order: Vec<i32>,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub games: i32,
    pub wins: i32,
}

/// 装备组合
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/HextechItemCombo.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct HextechItemCombo {
    pub item_ids: Vec<i32>,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub games: i32,
    pub wins: i32,
}

/// 海克斯英雄详情（增强 + 出装）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/HextechChampionDetail.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct HextechChampionDetail {
    pub summary: HextechChampionSummary,
    pub augments: Vec<HextechAugmentStat>,
    pub augment_trios: Vec<HextechAugmentTrio>,
    pub summoner_spells: Vec<HextechSpellCombo>,
    pub skill_orders: Vec<HextechSkillOrder>,
    pub starting_items: Vec<HextechItemCombo>,
    pub core_items: Vec<HextechItemCombo>,
}

/// 强度榜行
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/HextechTierListItem.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct HextechTierListItem {
    pub champion_id: i32,
    pub name: String,
    pub alias: String,
    pub icon_url: String,
    pub roles: Vec<String>,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub tier: Option<i32>,
    pub rank: i32,
}

/// 强度榜
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/HextechTierList.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct HextechTierList {
    pub data_version: String,
    pub game_patch: Option<String>,
    pub region: String,
    pub data: Vec<HextechTierListItem>,
}
