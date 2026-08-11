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

/// 强度榜对抗样本
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggTierCounter.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggTierCounter {
    pub champion_id: i32,
    pub play: i32,
    pub win: i32,
}

/// 强度榜统计（全英雄或某分路）
#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggTierStats.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggTierStats {
    pub play: i32,
    /// 排位/ARAM/URF: 胜率；Arena: 吃鸡率（first_place / play）
    pub win_rate: f64,
    pub pick_rate: f64,
    /// ARAM/URF 常为 0（现网 null）
    pub ban_rate: f64,
    pub kda: f64,
    /// OP.GG tier 数字（1=OP …）
    pub tier: i32,
    pub rank: i32,
    /// Arena: 吃鸡场次
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_place: Option<i32>,
    /// Arena: 排名总和
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_place: Option<i32>,
}

/// ARAM 等模式的英雄定位占比
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggTierRole.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggTierRole {
    pub name: String,
    pub win_rate: f64,
    pub role_rate: f64,
    pub play: i32,
}

/// 强度榜分路行
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggTierPosition.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggTierPosition {
    pub name: String,
    pub stats: OpggTierStats,
    pub counters: Vec<OpggTierCounter>,
}

/// 层级列表项（对齐现网 lol-api-champion 结构）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpggTierListItem.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpggTierListItem {
    pub champion_id: i32,
    pub average_stats: OpggTierStats,
    /// 排位有分路；ARAM/URF/Arena 为空
    pub positions: Vec<OpggTierPosition>,
    /// ARAM 常见；其它模式多为空
    pub roles: Vec<OpggTierRole>,
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
