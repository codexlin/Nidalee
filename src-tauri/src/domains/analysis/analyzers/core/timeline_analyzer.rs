//! 时间线分析器（兼容映射层）
//!
//! ⚠️ 这里**不再**实现任何速率 / 差值 / 优势公式。
//! 所有计算都委托给 `domains::analysis::evidence`，本文件只负责把
//! `PhaseEvidence` / `EventEvidence` 映射回旧的 `TimelineAnalysis` 形状，
//! 让 crate 内既有消费者（traits / advice / service）不受影响。
//!
//! 相对旧实现的修正：
//! - 速率分母是真实经过时间 `(last.timestamp - first.timestamp)/60000`，不再是帧数量
//! - CS 含线上 + 野怪，打野数据不再丢失
//! - 短局 / remake 不再整体返回 `None`，缺失阶段通过 `has_phase()` 显式表达
//! - 事件统计包含助攻，`killerId == 0` 不归给任何玩家
//! - `overall_advantage` 基于归一化份额差，不再直接加权 raw XP/Gold
//! - 「对线比较」固定取 Early 阶段，不再退化成最后一个有差值的阶段

use serde_json::Value;
use std::collections::HashMap;

use crate::domains::analysis::evidence::{
    advantage_to_percent, compute_phase_evidence, extract_event_evidence, laning_opponent_diff, timeline_frames,
    EventEvidence, EvidenceEventKind, GamePhase, PhaseEvidence,
};

/// 时间线帧数据结构（旧分析器仍在使用）
#[derive(Debug, Clone)]
pub struct TimelineFrame {
    pub timestamp: i64, // 时间戳（毫秒）
    pub events: Vec<GameEvent>,
    pub participant_frames: HashMap<String, ParticipantFrame>,
}

/// 游戏事件
#[derive(Debug, Clone)]
pub struct GameEvent {
    pub event_type: String,
    pub timestamp: i64,
    pub killer_id: Option<i32>,
    pub victim_id: Option<i32>,
    pub assisting_participant_ids: Vec<i32>,
    pub position: Option<Position>,
    pub monster_type: Option<String>,
    pub monster_sub_type: Option<String>,
}

/// 参与者帧数据
#[derive(Debug, Clone)]
pub struct ParticipantFrame {
    pub participant_id: i32,
    pub current_gold: i32,
    pub total_gold: i32,
    pub level: i32,
    pub xp: i32,
    pub minions_killed: i32,
    pub jungle_minions_killed: i32,
    pub position: Position,
}

/// 位置信息（地图坐标）
#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// 阶段分析结果
///
/// `cs_per_minute` 是 **total CS**（线上 + 野怪）/ 真实分钟；
/// `*_difference` 是阶段末的累计绝对差，不是 per-minute delta。
#[derive(Debug, Clone, Default)]
pub struct PhaseAnalysis {
    pub cs_per_minute: f64,
    pub gold_per_minute: f64,
    pub xp_per_minute: f64,
    pub cs_difference: f64,
    pub xp_difference: f64,
    pub gold_difference: f64,
    pub level_difference: f64,
}

/// 关键事件
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub event_type: String,
    pub timestamp: i64,
    pub participant_id: i32,
    pub importance_score: f64,
    pub description: String,
}

/// 对线期对手比较
///
/// 只描述**对线期**（0-10 分钟）的对位差距：中后期差值反映的是团队节奏与带线收益，
/// 拿它当对线结论会把「对线被打崩但中期滚起来」说成对线优势。
///
/// `overall_advantage` 是归一化份额差组合后的**百分比**（-100 ~ 100），
/// 由 `evidence::advantage_to_percent` 统一换算；
/// `*_advantage` 则仍是各自单位的阶段末绝对差，两者量纲不同，不可互相比较。
#[derive(Debug, Clone, Default)]
pub struct OpponentComparison {
    pub opponent_id: i32,
    pub cs_advantage: f64,
    pub xp_advantage: f64,
    pub gold_advantage: f64,
    pub level_advantage: f64,
    pub overall_advantage: f64,
}

/// 时间线分析结果
#[derive(Debug, Clone)]
pub struct TimelineAnalysis {
    /// 对线期 (0-10 分钟)；阶段不存在时为零值，请配合 `has_phase` 判断
    pub early_game: PhaseAnalysis,

    /// 中期 (10-20 分钟)
    pub mid_game: PhaseAnalysis,

    /// 后期 (20 分钟+)
    pub late_game: PhaseAnalysis,

    /// 关键事件（按时间升序）
    pub key_events: Vec<KeyEvent>,

    /// 对手比较
    pub opponent_comparison: OpponentComparison,

    /// 权威证据：只包含真实存在的阶段（短局 / remake 会缺 mid/late）
    pub phases: Vec<PhaseEvidence>,
}

impl TimelineAnalysis {
    /// 该阶段是否真实存在（而不是被补出来的零值）
    pub fn has_phase(&self, phase: GamePhase) -> bool {
        self.phases.iter().any(|p| p.phase == phase)
    }

    pub fn phase_evidence(&self, phase: GamePhase) -> Option<&PhaseEvidence> {
        self.phases.iter().find(|p| p.phase == phase)
    }
}

/// 解析时间线数据
///
/// 只有在时间线里根本没有 `frames`（或为空）时才返回 `None`；
/// 短局、remake、缺帧都会返回部分阶段的分析结果。
pub fn parse_timeline_data(
    timeline_json: &Value,
    target_participant_id: i32,
    opponent_id: Option<i32>,
) -> Option<TimelineAnalysis> {
    let frames = timeline_frames(timeline_json)?;
    if frames.is_empty() {
        return None;
    }

    let (phases, _flags) = compute_phase_evidence(frames, target_participant_id, opponent_id);
    let events = extract_event_evidence(
        frames,
        crate::domains::analysis::evidence::events::EventExtractContext {
            target_participant_id,
            target_team_id: 100,
            opponent_participant_id: opponent_id,
        },
    );

    Some(TimelineAnalysis {
        early_game: to_phase_analysis(&phases, GamePhase::Early),
        mid_game: to_phase_analysis(&phases, GamePhase::Mid),
        late_game: to_phase_analysis(&phases, GamePhase::Late),
        key_events: to_key_events(&events, target_participant_id),
        opponent_comparison: to_opponent_comparison(&phases),
        phases,
    })
}

/// 阶段证据 → 旧的 `PhaseAnalysis`（阶段缺失时为零值）
fn to_phase_analysis(phases: &[PhaseEvidence], phase: GamePhase) -> PhaseAnalysis {
    let Some(evidence) = phases.iter().find(|p| p.phase == phase) else {
        return PhaseAnalysis::default();
    };

    let diff = evidence.opponent_diff.as_ref();

    PhaseAnalysis {
        cs_per_minute: evidence.cs_per_min.unwrap_or(0.0),
        gold_per_minute: evidence.gold_per_min.unwrap_or(0.0),
        xp_per_minute: evidence.xp_per_min.unwrap_or(0.0),
        cs_difference: diff.map(|d| d.cs_diff as f64).unwrap_or(0.0),
        xp_difference: diff.map(|d| d.xp_diff as f64).unwrap_or(0.0),
        gold_difference: diff.map(|d| d.gold_diff as f64).unwrap_or(0.0),
        level_difference: diff.map(|d| d.level_diff as f64).unwrap_or(0.0),
    }
}

/// 取**对线期**的对手差作为对位比较
///
/// 阶段选择委托给 `evidence::laning_opponent_diff`，与证据层共用同一条规则；
/// 没有对线期差值（缺帧 / 未识别到对手）时返回零值，由调用方结合
/// `has_phase(GamePhase::Early)` 判断是「真的持平」还是「根本没数据」。
fn to_opponent_comparison(phases: &[PhaseEvidence]) -> OpponentComparison {
    let Some(diff) = laning_opponent_diff(phases) else {
        return OpponentComparison::default();
    };

    OpponentComparison {
        opponent_id: diff.opponent_participant_id,
        cs_advantage: diff.cs_diff as f64,
        xp_advantage: diff.xp_diff as f64,
        gold_advantage: diff.gold_diff as f64,
        level_advantage: diff.level_diff as f64,
        // 归一化之后才组合，再统一换算成百分比供旧阈值使用
        overall_advantage: advantage_to_percent(diff.overall_advantage),
    }
}

/// 事件证据 → 旧的 `KeyEvent`
fn to_key_events(events: &EventEvidence, target_participant_id: i32) -> Vec<KeyEvent> {
    events
        .key_events
        .iter()
        .map(|event| {
            let (event_type, importance_score, description) = match event.kind {
                EvidenceEventKind::ChampionKill => ("CHAMPION_KILL", 10.0, "击杀敌方英雄"),
                EvidenceEventKind::ChampionDeath => ("CHAMPION_KILL", -8.0, "被敌方击杀"),
                EvidenceEventKind::ChampionAssist => ("CHAMPION_KILL", 6.0, "参与击杀"),
                EvidenceEventKind::ChampionSpecialKill => ("CHAMPION_SPECIAL_KILL", 8.0, "特殊击杀"),
                EvidenceEventKind::Dragon => ("ELITE_MONSTER_KILL", 7.0, "参与击杀小龙"),
                EvidenceEventKind::Herald => ("ELITE_MONSTER_KILL", 6.0, "参与击杀峡谷先锋"),
                EvidenceEventKind::Baron => ("ELITE_MONSTER_KILL", 10.0, "参与击杀纳什男爵"),
                EvidenceEventKind::Horde => ("ELITE_MONSTER_KILL", 4.0, "参与击杀虚空幼虫"),
                EvidenceEventKind::EliteMonster => ("ELITE_MONSTER_KILL", 5.0, "参与击杀史诗野怪"),
                EvidenceEventKind::Building => ("BUILDING_KILL", 5.0, "参与摧毁建筑"),
                EvidenceEventKind::TurretPlate => ("TURRET_PLATE_DESTROYED", 3.0, "镀层"),
                EvidenceEventKind::WardPlaced => ("WARD_PLACED", 2.0, "插眼"),
                EvidenceEventKind::WardKill => ("WARD_KILL", 2.0, "排眼"),
                EvidenceEventKind::ItemPurchased
                | EvidenceEventKind::ItemSold
                | EvidenceEventKind::ItemDestroyed
                | EvidenceEventKind::ItemUndo => ("ITEM", 1.0, "出装相关"),
                EvidenceEventKind::SkillLevelUp => ("SKILL_LEVEL_UP", 1.0, "技能加点"),
                EvidenceEventKind::LevelUp => ("LEVEL_UP", 1.0, "升级"),
                EvidenceEventKind::DragonSoul => ("DRAGON_SOUL_GIVEN", 6.0, "龙魂"),
                EvidenceEventKind::GameEnd => ("GAME_END", 0.0, "对局结束"),
                EvidenceEventKind::PauseEnd => ("PAUSE_END", 0.0, "暂停结束"),
                EvidenceEventKind::Unknown => ("UNKNOWN", 0.0, "未知时间线事件"),
            };

            KeyEvent {
                event_type: event_type.to_string(),
                timestamp: event.timestamp_ms,
                participant_id: target_participant_id,
                importance_score,
                description: description.to_string(),
            }
        })
        .collect()
}
