use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// OP.GG 英雄详细数据
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggChampionBuild.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggChampionBuild {
    pub summary: OpggChampionSummary,
    pub summoner_spells: Vec<OpggSummonerSpell>,
    pub champion_skills: OpggSkills,
    pub items: OpggItems,
    pub counters: OpggCounters,
    pub perks: Vec<OpggPerk>,
}

/// 英雄摘要信息
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggChampionSummary.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggChampionSummary {
    pub name: String,
    pub champion_id: i32,
    pub icon: String,
    pub position: String,
    pub win_rate: Option<f64>,
    pub pick_rate: Option<f64>,
    pub ban_rate: Option<f64>,
    pub kda: Option<f64>,
    pub tier: Option<String>,
    pub rank: Option<i32>,
}

/// 召唤师技能
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggSummonerSpell.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggSummonerSpell {
    pub spell_id: i32,
    pub ids: Vec<i32>,
    pub win: i32,
    pub play: i32,
    pub pick_rate: f64,
}

/// 英雄技能
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggSkills.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggSkills {
    pub masteries: Vec<String>,
    pub order: Vec<String>,
    pub play: i32,
    pub win: i32,
    pub pick_rate: f64,
}

/// 装备数据
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggItems.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggItems {
    pub start_items: Vec<OpggItem>,
    pub core_items: Vec<OpggItem>,
    pub boots: Vec<OpggItem>,
    pub last_items: Vec<OpggItem>,
}

/// 装备项目
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggItem.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggItem {
    pub id: i32,
    pub ids: Vec<i32>,
    pub icons: Vec<String>,
    pub win: i32,
    pub play: i32,
    pub pick_rate: f64,
}

/// 克制关系
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggCounters.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggCounters {
    pub strong_against: Vec<OpggCounter>,
    pub weak_against: Vec<OpggCounter>,
}

/// 克制英雄
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggCounter.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggCounter {
    pub champion_id: i32,
    pub win_rate: f64,
}

/// 符文配置
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggPerk.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggPerk {
    pub primary_id: i32,
    pub secondary_id: i32,
    pub perks: Vec<i32>,
    pub win: i32,
    pub play: i32,
    pub pick_rate: f64,
}

/// OP.GG API 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpggApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

/// 层级列表项
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggTierListItem.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggTierListItem {
    pub champion_id: i32,
    pub name: String,
    pub tier: String,
    pub rank: i32,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub ban_rate: f64,
}

/// 层级列表
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggTierList.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggTierList {
    pub meta: OpggTierListMeta,
    pub data: Vec<OpggTierListItem>,
}

/// 层级列表元数据
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggTierListMeta.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggTierListMeta {
    pub version: String,
    pub region: String,
    pub mode: String,
    pub tier: String,
}
