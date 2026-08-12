use serde::{Deserialize, Serialize};
use ts_rs::TS;
/// 过程复盘洞察（不以胜率为主结论）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ProcessInsight.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInsight {
    pub sample_size: u32,
    pub timeline_games: u32,
    pub has_timeline: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degradation_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub death_breakdown: Option<DeathBreakdownCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub laning_process: Option<LaningProcessCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objective_process: Option<ObjectiveProcessCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision_process: Option<VisionProcessCard>,
    pub actions: Vec<ProcessAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/DeathBreakdownCard.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct DeathBreakdownCard {
    pub total_deaths: u32,
    pub solo: u32,
    pub gank_or_multi: u32,
    pub tower_or_minion: u32,
    pub solo_rate: f64,
    pub gank_rate: f64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/LaningProcessCard.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct LaningProcessCard {
    pub sample_size: u32,
    pub avg_cs_diff: f64,
    pub avg_gold_diff: f64,
    pub avg_overall_advantage_pct: f64,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ObjectiveProcessCard.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ObjectiveProcessCard {
    pub dragons_seen: u32,
    pub dragons_taken: u32,
    pub dragons_missed: u32,
    pub heralds_seen: u32,
    pub heralds_taken: u32,
    pub heralds_missed: u32,
    pub barons_seen: u32,
    pub barons_taken: u32,
    pub barons_missed: u32,
    pub missed_activity: Vec<ActivityBucketCount>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ActivityBucketCount.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ActivityBucketCount {
    /// dead / base / ownJungle / enemyJungle / riverOrObjective / lane / unknown
    pub activity: String,
    pub label: String,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/VisionProcessCard.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct VisionProcessCard {
    pub wards_placed: u32,
    pub wards_killed: u32,
    pub games_with_wards: u32,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ProcessAction.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ProcessAction {
    pub key: String,
    pub title: String,
    pub detail: String,
    pub priority: u8,
}

/// 单局过程复盘的关键时刻（短列表，不是完整时间轴）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ProcessKeyMoment.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ProcessKeyMoment {
    #[ts(type = "number")]
    pub timestamp_ms: i64,
    pub label: String,
    /// 我在干嘛（邻近帧粗分类）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    /// 对位在干嘛
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opponent_detail: Option<String>,
}

/// 对位阶段差（相对对手，正数=领先）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpponentPhaseCompare.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpponentPhaseCompare {
    /// early / mid / late
    pub phase: String,
    /// 展示标签：前 10 分 / 10–20 分 / 20 分后
    pub label: String,
    #[ts(type = "number")]
    pub cs_diff: i64,
    #[ts(type = "number")]
    pub gold_diff: i64,
    #[ts(type = "number")]
    pub xp_diff: i64,
    pub level_diff: i32,
    /// 综合优势百分比（约 [-100, 100]）
    pub overall_advantage_pct: f64,
    #[ts(type = "number")]
    pub my_gold: i64,
    #[ts(type = "number")]
    pub opponent_gold: i64,
}

/// 单局对位对比
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpponentCompareCard.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpponentCompareCard {
    pub my_champion_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opponent_champion_id: Option<i32>,
    pub opponent_participant_id: i32,
    /// TOP / JUNGLE / ...
    pub position: String,
    pub confidence: f64,
    pub phases: Vec<OpponentPhaseCompare>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// 单局过程复盘（详情弹窗）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/GameProcessReview.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct GameProcessReview {
    pub insight: ProcessInsight,
    pub key_moments: Vec<ProcessKeyMoment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opponent_compare: Option<OpponentCompareCard>,
    /// full / timelineMissing / timelinePartial（与 EvidenceQuality 同形字符串）
    pub quality: String,
    /// 是否命中前端传入的已分析证据（未再打 LCU）
    pub from_cache: bool,
}
