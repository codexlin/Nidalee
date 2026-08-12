use super::{ChampSelectAction, ChampSelectBans, ChampSelectTimer, SummonerInfo};
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
    pub champion_name: String,
    pub games: u32,
    pub wins: u32,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/SummonerWithMatches.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct SummonerWithMatches {
    pub display_name: String,
    pub summoner_info: SummonerInfo,
    pub matches: PlayerMatchStats,
    /// 与 `matches` 同源的一次 analyze 结果投影，避免搜索页再打第二轮 LCU
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub position_analysis: Option<MultiPositionAnalysis>,
}

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

/// ⭐ v3.4: 多位置分组分析结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/MultiPositionAnalysis.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct MultiPositionAnalysis {
    /// 所有位置的统计（按场次从多到少排序）
    pub position_stats: Vec<PositionStats>,

    /// 主要位置（场次最多的）
    pub main_position: String,

    /// 总览数据（所有位置合计；全部模式下为混合合计）
    pub overall_stats: PlayerMatchStats,

    /// 排位桶统计（420/440）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ranked_stats: Option<PlayerMatchStats>,

    /// 非排位桶统计
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub other_stats: Option<PlayerMatchStats>,
}

/// ⭐ v3.4: 单个位置的统计数据
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PositionStats.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct PositionStats {
    /// ASCII 位置码：TOP / JUNGLE / MID / ADC / SUPPORT / ARAM / FLEX / UNKNOWN
    pub position: String,

    /// 该位置的场次
    pub games: u32,

    /// 该位置的胜场
    pub wins: u32,

    /// 该位置的胜率
    pub win_rate: f64,

    /// 该位置的统计数据
    pub stats: PlayerMatchStats,

    /// 英雄池数据（Top 5）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub champion_pool: Option<Vec<ChampionStat>>,

    /// 胜率趋势（最近的对局）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_rate_trend: Option<Vec<TrendPoint>>,

    /// 过程复盘洞察（深度时间线）；无时间线时带降级说明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_insight: Option<ProcessInsight>,
}

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
    pub champion_name: String,
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
