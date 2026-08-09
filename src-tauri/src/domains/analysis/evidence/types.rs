//! Evidence 证据包的具体类型
//!
//! 这一层只描述「事实」：真实时间、真实增量、阶段末绝对差、归一化优势、事件参与、
//! 样本量与置信度。任何阈值判断、结论生成、AI 解释都不属于这里。

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::position::EvidencePosition;

/// 对线期结束（毫秒）
pub const PHASE_EARLY_END_MS: i64 = 600_000;
/// 中期结束（毫秒）
pub const PHASE_MID_END_MS: i64 = 1_200_000;

/// 支撑结论所需的最小样本量（低于该值只允许描述，不允许下结论）
pub const MIN_SAMPLE_FOR_CONCLUSION: u32 = 3;

/// 归一化优势的组合权重（和为 1，作用于无量纲份额差而非 raw 值）
pub const ADVANTAGE_WEIGHT_CS: f64 = 0.35;
pub const ADVANTAGE_WEIGHT_GOLD: f64 = 0.35;
pub const ADVANTAGE_WEIGHT_XP: f64 = 0.30;

/// 归一化优势 → 百分比的换算系数
///
/// 证据层的事实值一律是无量纲份额差 `[-1, 1]`；只有需要喂给「按百分比写死阈值」的
/// 旧消费者时才乘这个系数。两种量纲不可混用，换算必须走 [`advantage_to_percent`]。
pub const ADVANTAGE_PERCENT_SCALE: f64 = 100.0;

/// 对局阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/GamePhase.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum GamePhase {
    /// [0, 10min]
    Early,
    /// [10min, 20min]
    Mid,
    /// [20min, 结束]
    Late,
}

impl GamePhase {
    /// 固定顺序（聚合与输出都依赖它）
    pub const ALL: [GamePhase; 3] = [GamePhase::Early, GamePhase::Mid, GamePhase::Late];

    /// 阶段边界（闭区间；边界帧同时作为前一阶段的终点和后一阶段的锚点）
    pub fn bounds(&self) -> (i64, Option<i64>) {
        match self {
            GamePhase::Early => (0, Some(PHASE_EARLY_END_MS)),
            GamePhase::Mid => (PHASE_EARLY_END_MS, Some(PHASE_MID_END_MS)),
            GamePhase::Late => (PHASE_MID_END_MS, None),
        }
    }

    pub fn contains(&self, timestamp_ms: i64) -> bool {
        match self.bounds() {
            (start, Some(end)) => timestamp_ms >= start && timestamp_ms <= end,
            (start, None) => timestamp_ms >= start,
        }
    }
}

/// 单局证据的完整度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EvidenceQuality.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceQuality {
    /// 详情 + 可用时间线齐全
    Full,
    /// 没有可用时间线，只有对局详情
    TimelineMissing,
    /// 有时间线但存在缺帧 / 单帧，部分速率不可计算
    TimelinePartial,
}

/// 证据缺陷码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EvidenceIssue.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceIssue {
    /// 调用方没有提供时间线
    TimelineMissing,
    /// 时间线响应里没有 `frames` 字段
    TimelineFramesMissing,
    /// `frames` 为空数组
    TimelineFramesEmpty,
    /// 部分（或全部）帧里没有目标玩家的 participantFrame
    TargetParticipantFramesMissing,
    /// 阶段内只有一个锚点帧，无法计算速率（禁止除零）
    SingleFrameNoRate,
    /// 无法确定对线对手
    OpponentUnidentified,
    /// 对手来自空间邻近回退，置信度低于 lane-role 匹配
    OpponentFromSpatialProximity,
    /// 该局无法提取证据，已跳过
    MatchSkipped,
}

/// 证据诊断（向用户解释「为什么这条结论不可用」）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EvidenceDiagnostic.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceDiagnostic {
    pub code: EvidenceIssue,
    pub message: String,
}

impl EvidenceDiagnostic {
    pub fn new(code: EvidenceIssue, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// 对手识别方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpponentMatchMethod.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum OpponentMatchMethod {
    /// 敌方存在唯一 role+lane 完全一致的玩家
    LaneRole,
    /// 敌方存在唯一同「统一位置」的玩家（role/lane 写法不同但语义一致）
    PositionMatch,
    /// 前 10 分钟帧的空间邻近（只在 lane-role 不可用时回退）
    SpatialProximity,
}

/// 对线对手证据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OpponentEvidence.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OpponentEvidence {
    pub participant_id: i32,
    pub team_id: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub champion_id: Option<i32>,
    pub position: EvidencePosition,
    pub method: OpponentMatchMethod,
    /// 0.0 ~ 1.0
    pub confidence: f64,
}

/// 阶段末与对手的绝对差 + 归一化优势
///
/// `*_diff` 是阶段最后一帧的累计值绝对差，**不是** per-minute delta，两者不可混用。
/// `normalized_*` 是无量纲份额差 `(self - opponent) / (self + opponent)`，取值 [-1, 1]，
/// 只有归一化之后才允许按权重组合，禁止直接加权 raw XP/Gold。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PhaseOpponentDiff.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct PhaseOpponentDiff {
    pub opponent_participant_id: i32,
    /// total CS（线上 + 野怪）绝对差
    #[ts(type = "number")]
    pub cs_diff: i64,
    #[ts(type = "number")]
    pub gold_diff: i64,
    #[ts(type = "number")]
    pub xp_diff: i64,
    pub level_diff: i32,
    pub normalized_cs_advantage: f64,
    pub normalized_gold_advantage: f64,
    pub normalized_xp_advantage: f64,
    /// 归一化后按权重组合的总优势，[-1, 1]
    pub overall_advantage: f64,
}

/// 单阶段证据
///
/// 速率的分母固定是 `(last.timestamp - first.timestamp) / 60000.0`（真实经过时间），
/// 不使用帧数量。阶段内只有一个锚点帧时速率为 `None`，绝不除零。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PhaseEvidence.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct PhaseEvidence {
    pub phase: GamePhase,
    /// 阶段内第一帧的真实时间戳
    #[ts(type = "number")]
    pub start_ms: i64,
    /// 阶段内最后一帧的真实时间戳
    #[ts(type = "number")]
    pub end_ms: i64,
    /// 真实经过分钟数（速率分母）
    pub duration_minutes: f64,
    /// 落在本阶段闭区间 `[start, end]` 内的帧数，含两端边界锚点。
    ///
    /// 相邻阶段共享边界帧，所以边界帧会同时计入两个阶段，
    /// 各阶段 `frame_count` 之和会比总帧数多（每个边界 +1）。这是刻意的：
    /// 每个阶段都需要自己的起点锚点才能算出该阶段内的增量，
    /// 少了它首帧的 CS/金币增量就会被算成从 0 开始。
    ///
    /// 因此 `frame_count` 只表示采样密度，不要拿它当速率分母 —— 速率一律用
    /// `duration_minutes`，而边界帧不会被重复计入任何增量。
    pub frame_count: u32,

    /// 阶段内线上兵线增量
    #[ts(type = "number")]
    pub lane_cs: i64,
    /// 阶段内野怪增量（打野单列）
    #[ts(type = "number")]
    pub jungle_cs: i64,
    /// lane + jungle
    #[ts(type = "number")]
    pub total_cs: i64,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub lane_cs_per_min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub jungle_cs_per_min: Option<f64>,
    /// total CS / 真实分钟
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub cs_per_min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub gold_per_min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub xp_per_min: Option<f64>,

    #[ts(type = "number")]
    pub end_gold: i64,
    #[ts(type = "number")]
    pub end_xp: i64,
    pub end_level: i32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub opponent_diff: Option<PhaseOpponentDiff>,
}

/// 关键事件类型（抽取层覆盖时间线已知枚举；未知进 Unknown）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EvidenceEventKind.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceEventKind {
    ChampionKill,
    ChampionDeath,
    ChampionAssist,
    ChampionSpecialKill,
    Dragon,
    Herald,
    Baron,
    /// 虚空幼虫
    Horde,
    /// 其余史诗野怪
    EliteMonster,
    Building,
    TurretPlate,
    WardPlaced,
    WardKill,
    ItemPurchased,
    ItemSold,
    ItemDestroyed,
    ItemUndo,
    SkillLevelUp,
    LevelUp,
    DragonSoul,
    GameEnd,
    PauseEnd,
    /// 未识别的 type，raw 名在 detail
    Unknown,
}

/// 目标玩家在事件中的角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EventInvolvement.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum EventInvolvement {
    Killer,
    Victim,
    Assist,
    /// 事件发生但目标未参与（仍记录，供资源复盘）
    Uninvolved,
    /// 目标是事件主体（放眼 / 买装 / 加点等）
    Actor,
}

/// 阵亡原因（过程复盘）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/DeathCause.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum DeathCause {
    /// 无助攻的英雄击杀
    Solo,
    /// 有助攻或合计多人参与
    GankOrMulti,
    /// killerId == 0（塔/小兵/处决）
    TowerOrMinion,
}

/// 未知事件样例（禁止静默丢弃）
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/UnknownTimelineEvent.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct UnknownTimelineEvent {
    pub raw_type: String,
    #[ts(type = "number")]
    pub timestamp_ms: i64,
    pub count: u32,
}

/// 单条关键事件证据（保留时间戳，供上层引用为证据）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/KeyEventEvidence.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct KeyEventEvidence {
    pub kind: EvidenceEventKind,
    #[ts(type = "number")]
    pub timestamp_ms: i64,
    pub involvement: EventInvolvement,
    /// 事件补充信息（龙的子类型 / 建筑类型 / 未知 raw type 等）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub death_cause: Option<DeathCause>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub killer_participant_id: Option<i32>,
    #[serde(default)]
    pub assistant_count: u32,
    /// 击杀者是否为对线对手（仅死亡事件）
    #[serde(default)]
    pub lane_opponent_kill: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub activity_context: Option<super::activity::ActivityContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub item_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub skill_slot: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub actor_participant_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub position_x: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub position_y: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub bounty: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub kill_streak_length: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub killer_team_id: Option<i32>,
    /// 资源/建筑是否己方阵营击杀
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub allied_side: Option<bool>,
    /// victimDamageReceived 条目数（有则保留，供后续伤害复盘）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub victim_damage_received_count: Option<u32>,
}

impl KeyEventEvidence {
    pub fn basic(
        kind: EvidenceEventKind,
        timestamp_ms: i64,
        involvement: EventInvolvement,
        detail: Option<String>,
    ) -> Self {
        Self {
            kind,
            timestamp_ms,
            involvement,
            detail,
            death_cause: None,
            killer_participant_id: None,
            assistant_count: 0,
            lane_opponent_kill: false,
            activity_context: None,
            item_id: None,
            skill_slot: None,
            actor_participant_id: None,
            position_x: None,
            position_y: None,
            bounty: None,
            kill_streak_length: None,
            killer_team_id: None,
            allied_side: None,
            victim_damage_received_count: None,
        }
    }
}

/// 事件证据（击杀 / 死亡 / 助攻 / 资源 / 建筑 / 视野 / 出装 / 加点…）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EventEvidence.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct EventEvidence {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub deaths_solo: u32,
    pub deaths_gank_or_multi: u32,
    pub deaths_tower_or_minion: u32,
    /// 参与（击杀或助攻）的小龙数
    pub dragon_takedowns: u32,
    pub herald_takedowns: u32,
    pub baron_takedowns: u32,
    pub horde_takedowns: u32,
    /// 地图上出现的小龙次数（含未参与）
    pub dragons_seen: u32,
    pub heralds_seen: u32,
    pub barons_seen: u32,
    /// 敌方拿龙且自己未参与
    pub dragons_missed: u32,
    pub heralds_missed: u32,
    pub barons_missed: u32,
    /// 建筑击杀 + 建筑助攻
    pub building_takedowns: u32,
    pub buildings_seen: u32,
    pub turret_plates: u32,
    pub wards_placed: u32,
    pub wards_killed: u32,
    pub items_purchased: u32,
    pub items_sold: u32,
    pub items_destroyed: u32,
    pub items_undo: u32,
    pub skill_level_ups: u32,
    pub level_ups: u32,
    pub special_kills: u32,
    pub dragon_souls: u32,
    /// 按时间升序的关键事件（抽全）
    pub key_events: Vec<KeyEventEvidence>,
    /// 未知 type 聚合（不丢弃）
    pub unknown_events: Vec<UnknownTimelineEvent>,
}

/// 时间线覆盖范围
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/TimelineSpan.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct TimelineSpan {
    #[ts(type = "number")]
    pub start_ms: i64,
    #[ts(type = "number")]
    pub end_ms: i64,
    pub frame_count: u32,
    /// 其中包含目标玩家 participantFrame 的帧数
    pub target_frame_count: u32,
}

/// 单局证据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/MatchEvidence.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct MatchEvidence {
    #[ts(type = "number")]
    pub game_id: u64,
    #[ts(type = "number")]
    pub queue_id: i64,
    #[ts(type = "number")]
    pub game_duration_seconds: i64,
    #[ts(type = "number")]
    pub game_creation: i64,

    pub participant_id: i32,
    pub team_id: i32,
    pub champion_id: i32,
    pub win: bool,
    pub position: EvidencePosition,
    pub quality: EvidenceQuality,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub opponent: Option<OpponentEvidence>,

    /// 只包含真实存在的阶段（短局 / remake 可能只有 early）
    pub phases: Vec<PhaseEvidence>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub events: Option<EventEvidence>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub timeline_span: Option<TimelineSpan>,

    pub diagnostics: Vec<EvidenceDiagnostic>,
}

impl MatchEvidence {
    pub fn phase(&self, phase: GamePhase) -> Option<&PhaseEvidence> {
        self.phases.iter().find(|p| p.phase == phase)
    }

    pub fn has_issue(&self, issue: EvidenceIssue) -> bool {
        self.diagnostics.iter().any(|d| d.code == issue)
    }
}

/// 样本量置信度（禁止一场触发全局强结论）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EvidenceConfidence.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum EvidenceConfidence {
    /// < 3 场：只能描述，不能下结论
    Insufficient,
    Low,
    Medium,
    High,
}

impl EvidenceConfidence {
    pub fn from_sample_size(sample_size: u32) -> Self {
        match sample_size {
            0..=2 => EvidenceConfidence::Insufficient,
            3..=4 => EvidenceConfidence::Low,
            5..=9 => EvidenceConfidence::Medium,
            _ => EvidenceConfidence::High,
        }
    }

    pub fn supports_conclusion(&self) -> bool {
        !matches!(self, EvidenceConfidence::Insufficient)
    }
}

/// 某阶段在一组对局上的均值
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PhaseAverages.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct PhaseAverages {
    pub phase: GamePhase,
    /// 有可用速率的对局数（零时长阶段不计入）
    pub sample_size: u32,
    /// 识别到对手且有阶段末差值的对局数
    pub opponent_sample_size: u32,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_lane_cs_per_min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_jungle_cs_per_min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_cs_per_min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_gold_per_min: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_xp_per_min: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_cs_diff: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_gold_diff: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_xp_diff: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub avg_overall_advantage: Option<f64>,
}

/// 事件频率（场均）
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EvidenceEventRates.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceEventRates {
    /// 有事件证据（即有时间线）的对局数
    pub sample_size: u32,
    pub kills_per_game: f64,
    pub deaths_per_game: f64,
    pub assists_per_game: f64,
    pub dragon_takedowns_per_game: f64,
    pub herald_takedowns_per_game: f64,
    pub baron_takedowns_per_game: f64,
    pub horde_takedowns_per_game: f64,
    pub building_takedowns_per_game: f64,
    /// 至少参与一条小龙的对局占比
    pub games_with_dragon_rate: f64,
    /// 至少参与一次大龙的对局占比
    pub games_with_baron_rate: f64,
}

/// 按 (队列, 位置) 聚合的证据摘要
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EvidenceSummary.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceSummary {
    #[ts(type = "number")]
    pub queue_id: i64,
    pub position: EvidencePosition,
    pub sample_size: u32,
    pub wins: u32,
    /// 胜率**百分比**，取值 `0.0..=100.0`（55% 是 `55.0`，不是 `0.55`）。
    ///
    /// 定义为 `wins / sample_size * 100`；`sample_size == 0` 时为 `0.0`。
    /// 同结构里的 `event_rates` 用的是 0-1 占比，两者刻意不同，前端别混用。
    pub win_rate: f64,
    pub confidence: EvidenceConfidence,
    /// 样本量是否足以支撑结论
    pub supports_conclusion: bool,
    /// 识别到对线对手的对局数
    pub opponent_identified_games: u32,
    pub phase_averages: Vec<PhaseAverages>,
    pub event_rates: EvidenceEventRates,
    /// 支撑该摘要的对局 ID（升序去重）
    #[ts(type = "number[]")]
    pub evidence_game_ids: Vec<u64>,
}

impl EvidenceSummary {
    pub fn phase_average(&self, phase: GamePhase) -> Option<&PhaseAverages> {
        self.phase_averages.iter().find(|p| p.phase == phase)
    }
}

/// 多局证据包
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/EvidenceBundle.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceBundle {
    pub target_puuid: String,
    pub match_count: u32,
    pub matches: Vec<MatchEvidence>,
    /// 按 (queue_id 升序, position 升序) 排列
    pub summaries: Vec<EvidenceSummary>,
    pub diagnostics: Vec<EvidenceDiagnostic>,
}

impl EvidenceBundle {
    pub fn empty(target_puuid: impl Into<String>) -> Self {
        Self {
            target_puuid: target_puuid.into(),
            match_count: 0,
            matches: Vec::new(),
            summaries: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn summary(&self, queue_id: i64, position: EvidencePosition) -> Option<&EvidenceSummary> {
        self.summaries
            .iter()
            .find(|s| s.queue_id == queue_id && s.position == position)
    }
}

/// 证据提取失败原因（可解释的 None）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceExtractionError {
    /// 对局 JSON 缺少 `participantIdentities`
    MissingParticipantIdentities,
    /// 目标 PUUID 不在这一局里
    TargetNotFound,
    /// 对局 JSON 缺少 `participants`
    MissingParticipants,
    /// `participantIdentities` 里有目标，但 `participants` 里没有对应条目
    TargetParticipantMissing,
}

impl EvidenceExtractionError {
    pub fn message(&self) -> &'static str {
        match self {
            EvidenceExtractionError::MissingParticipantIdentities => {
                "对局数据缺少 participantIdentities，无法定位目标玩家"
            }
            EvidenceExtractionError::TargetNotFound => "目标 PUUID 不在该对局的参与者名单中",
            EvidenceExtractionError::MissingParticipants => "对局数据缺少 participants",
            EvidenceExtractionError::TargetParticipantMissing => "participants 中缺少目标玩家对应的 participantId",
        }
    }
}

impl std::fmt::Display for EvidenceExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}

/// 无量纲份额差 `(a - b) / (a + b)`，取值 [-1, 1]
///
/// 这是把不同单位（CS / Gold / XP）拉到同一尺度的唯一方式：
/// 直接加权 raw 值会让经验（数千量级）压死补刀（数十量级）。
/// 归一化优势（[-1, 1]）→ 百分比（[-100, 100]）
///
/// 唯一允许的量纲换算入口：越界输入先收敛再放大，保证旧阈值永远拿到有界的百分比。
pub fn advantage_to_percent(normalized_advantage: f64) -> f64 {
    normalized_advantage.clamp(-1.0, 1.0) * ADVANTAGE_PERCENT_SCALE
}

pub fn normalized_share_advantage(target: i64, opponent: i64) -> f64 {
    let total = target.saturating_add(opponent);
    if total == 0 {
        return 0.0;
    }
    let value = (target - opponent) as f64 / (total as f64).abs();
    value.clamp(-1.0, 1.0)
}
