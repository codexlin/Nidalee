use serde::{Deserialize, Serialize};
use ts_rs::TS;
/// 分析用英雄统计数据（包含英雄名称）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisChampionStats.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisChampionStats {
    pub champion_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub champion_name: Option<String>,
    pub games: u32,
    pub wins: u32,
    pub win_rate: f64,
}

/// 召唤师特征标签
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/SummonerTrait.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct SummonerTrait {
    pub name: String,
    pub description: String,
    pub score: i32,
    #[serde(rename = "type")]
    pub trait_type: String, // "good" or "bad"
}

/// 建议分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AdviceCategory.ts",
    rename_all = "PascalCase"
)]
#[serde(rename_all = "PascalCase")]
pub enum AdviceCategory {
    Laning,      // 对线
    Farming,     // 发育/补刀
    Teamfight,   // 团战
    Vision,      // 视野
    Positioning, // 站位
    Decision,    // 决策
    Champion,    // 英雄池
}

/// 建议视角
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AdvicePerspective.ts",
    rename_all = "PascalCase"
)]
#[serde(rename_all = "PascalCase")]
pub enum AdvicePerspective {
    SelfImprovement, // 对自己的改进建议
    Targeting,       // 针对敌人的战术建议
    Collaboration,   // 协同队友的建议
}

/// 游戏建议（v3.0 新增）⭐
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/GameAdvice.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct GameAdvice {
    pub title: String,
    pub problem: String,
    pub evidence: String,
    pub suggestions: Vec<String>,
    pub priority: i32,
    pub category: AdviceCategory,
    pub perspective: AdvicePerspective,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub affected_role: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_player: Option<String>,
}

/// 玩家战绩统计（完整版 - 包含所有分析数据）
#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[ts(
    export,
    export_to = "../../src/types/generated/PlayerMatchStats.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct PlayerMatchStats {
    // === 基础统计 ===
    pub total_games: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,

    // === KDA 统计 ===
    pub avg_kills: f64,
    pub avg_deaths: f64,
    pub avg_assists: f64,
    pub avg_kda: f64,

    // === 今日统计 ===
    pub today_games: u32,
    pub today_wins: u32,

    // === 衍生量化指标 ===
    pub dpm: f64,  // 每分钟伤害
    pub cspm: f64, // 每分钟补刀
    pub vspm: f64, // 每分钟视野得分

    // === 定性特征标签 ===
    pub traits: Vec<SummonerTrait>,

    // === 常用英雄 ===
    pub favorite_champions: Vec<AnalysisChampionStats>,

    // === 当前分析样本中的最近表现 ===
    pub recent_performance: Vec<MatchPerformance>,

    // === v3.0: 智能建议 ⭐ ===
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub advice: Vec<GameAdvice>,
}

/// 英雄统计数据
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ChampionStat.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ChampionStat {
    /// 英雄ID
    pub champion_id: i32,

    /// 英雄名称（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub champion_name: Option<String>,

    /// 使用场次
    pub games: u32,

    /// 胜场
    pub wins: u32,

    /// 胜率
    pub win_rate: f64,

    /// 平均KDA
    pub avg_kda: f64,
}

/// 趋势点数据
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/TrendPoint.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct TrendPoint {
    /// 对局索引（第几场）
    pub index: u32,

    /// 是否胜利
    pub win: bool,

    /// 累计胜率（截止到这场）
    pub cumulative_win_rate: f64,

    /// 移动平均胜率（最近5场）
    pub moving_avg_win_rate: f64,
}

/// 单场比赛表现
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/MatchPerformance.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct MatchPerformance {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | undefined")]
    pub game_id: Option<u64>,
    pub win: bool,
    pub champion_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub champion_name: Option<String>,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub kda: f64,
    /// 自研单场评级：S+ / S / A / B / C / D（由 KDA 阈值映射，非官方）
    pub grade: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_duration: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | undefined")]
    pub game_creation: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(type = "number | undefined")]
    pub queue_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_mode: Option<String>,
    // ⭐ v3.1: 位置信息（用于前端展示和分析）
    pub role: String, // 原始 role：DUO_CARRY, DUO_SUPPORT, SOLO, JUNGLE
    pub lane: String, // 原始 lane：TOP, MIDDLE, BOTTOM, JUNGLE
    /// ASCII 位置码：TOP / JUNGLE / MID / ADC / SUPPORT / ARAM / FLEX / UNKNOWN
    pub position: String,
}
