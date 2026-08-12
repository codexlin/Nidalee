use super::super::{ChampSelectAction, ChampSelectBans, ChampSelectTimer};
use super::statistics::{MatchPerformance, PlayerMatchStats};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/generated/ConnectionState.ts")]
pub enum ConnectionState {
    Connected,
    ProcessFound,
    Unstable,
    AuthExpired,
    Disconnected,
}

/// 单个排位队列的段位快照。单双排与灵活排位必须分别表达，不能复用一个模糊的 tier 字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PlayerRankSummary.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct PlayerRankSummary {
    pub tier: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub division: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub league_points: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/RankedRatingGrade.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum RankedRatingGrade {
    SPlus,
    S,
    A,
    B,
    C,
    D,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/RankedRatingQueue.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum RankedRatingQueue {
    SoloRanked,
    FlexRanked,
}

/// A rank-anchored, sample-adjusted summary for quick comparison on the live analysis screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/RankedPlayerRating.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct RankedPlayerRating {
    pub grade: RankedRatingGrade,
    pub label: String,
    pub queue: RankedRatingQueue,
    pub confidence: PlayerAnalysisConfidence,
    pub summary: String,
}

/// 玩家完整分析数据（包含战绩）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PlayerAnalysisData.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct PlayerAnalysisData {
    // 基础信息
    pub cell_id: i32,
    pub display_name: String,
    pub summoner_id: Option<String>,
    pub puuid: Option<String>,
    pub is_local: bool,
    pub is_bot: bool,
    pub analysis_status: PlayerAnalysisStatus,

    // 英雄信息
    pub champion_id: Option<i32>,
    pub champion_name: Option<String>,
    pub champion_pick_intent: Option<i32>,
    pub position: Option<String>,

    // 召唤师信息
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub solo_rank: Option<PlayerRankSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub flex_rank: Option<PlayerRankSummary>,
    pub profile_icon_id: Option<i32>,
    pub tag_line: Option<String>,
    pub spell1_id: Option<i64>, // 改为 i64 以支持大数值
    pub spell2_id: Option<i64>, // 改为 i64 以支持大数值

    // 战绩数据（只有真实玩家才有）
    pub match_stats: Option<PlayerMatchStats>,

    /// 未经分析范围筛选的真实最近对局，按时间倒序，供实时卡片切换展示。
    pub recent_matches: Vec<MatchPerformance>,

    /// 本次实时分析实际采用的样本范围。与战绩一起返回，避免前端猜测数据口径。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub analysis_basis: Option<PlayerAnalysisBasis>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ranked_rating: Option<RankedPlayerRating>,
}

/// 实时玩家分析生命周期。显式状态避免前端用空名字/空战绩猜测加载进度。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PlayerAnalysisStatus.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum PlayerAnalysisStatus {
    AwaitingIdentity,
    Loading,
    Ready,
    InsufficientData,
    Unavailable,
    Bot,
}

/// 实时对局分析采用的主要历史样本范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PlayerAnalysisScope.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum PlayerAnalysisScope {
    SoloRanked,
    FlexRanked,
    Ranked,
    SummonersRift,
    Aram,
    CurrentMode,
    RecentOverall,
}

/// 样本可信度只描述数据量，不代表对玩家实力的主观评级。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PlayerAnalysisConfidence.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum PlayerAnalysisConfidence {
    Low,
    Medium,
    High,
}

/// 玩家卡片展示用的分析口径与样本说明。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PlayerAnalysisBasis.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct PlayerAnalysisBasis {
    pub primary_scope: PlayerAnalysisScope,
    pub scanned_games: u32,
    pub primary_games: u32,
    /// 已选样本中，为弥补当前精确范围而纳入的更宽范围对局数。
    pub supporting_games: u32,
    pub confidence: PlayerAnalysisConfidence,
    pub fallback_used: bool,
}

/// 队伍分析数据
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/TeamAnalysisData.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct TeamAnalysisData {
    pub my_team: Vec<PlayerAnalysisData>,
    pub enemy_team: Vec<PlayerAnalysisData>,
    pub local_player_cell_id: i32,
    pub game_phase: String,
    #[ts(type = "number")]
    pub queue_id: i64, // 队列类型ID：420=单排, 440=灵活排位, 450=大乱斗等
    pub is_custom_game: bool, // 是否自定义游戏

    // 🔥 新增：选人流程相关字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<Vec<ChampSelectAction>>>, // 选人/ban 动作序列
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bans: Option<ChampSelectBans>, // ban 位信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timer: Option<ChampSelectTimer>, // 计时器信息
}
