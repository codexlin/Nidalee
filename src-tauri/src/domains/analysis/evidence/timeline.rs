//! 时间线帧的纯计算
//!
//! 唯一的速率公式来源：`(last.timestamp - first.timestamp) / 60000.0` 作分母。
//! 帧数量、帧间隔都不参与速率计算（帧间隔在缺帧/短局时并不可靠）。

use serde_json::Value;

use super::types::{
    normalized_share_advantage, GamePhase, PhaseEvidence, PhaseOpponentDiff, TimelineSpan, ADVANTAGE_WEIGHT_CS,
    ADVANTAGE_WEIGHT_GOLD, ADVANTAGE_WEIGHT_XP,
};

/// 单帧里某个参与者的快照（抽全：LCU 有则读）
#[derive(Debug, Clone, PartialEq)]
pub struct FrameSnapshot {
    pub timestamp_ms: i64,
    pub lane_cs: i64,
    pub jungle_cs: i64,
    pub total_gold: i64,
    pub current_gold: i64,
    pub gold_per_second: Option<i64>,
    pub xp: i64,
    pub level: i32,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub time_enemy_spent_controlled: Option<i64>,
    /// damageStats.totalDamageDoneToChampions（有则保留）
    pub total_damage_done_to_champions: Option<i64>,
    /// championStats 键数量（证明抽到了扩展块）
    pub champion_stats_keys: u32,
    /// damageStats 键数量
    pub damage_stats_keys: u32,
}

impl FrameSnapshot {
    /// 线上 + 野怪
    pub fn total_cs(&self) -> i64 {
        self.lane_cs + self.jungle_cs
    }
}

/// 时间线帧计算的副作用信息
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhaseComputationFlags {
    /// 存在没有目标玩家 participantFrame 的帧
    pub missing_target_frames: bool,
    /// 存在只有单个锚点帧、无法计算速率的阶段
    pub single_frame_phase: bool,
}

/// 读取时间线的 `frames` 数组
pub fn timeline_frames(timeline: &Value) -> Option<&Vec<Value>> {
    timeline.get("frames").and_then(Value::as_array)
}

/// 从一帧里读取指定参与者的快照
pub fn frame_snapshot(frame: &Value, participant_id: i32) -> Option<FrameSnapshot> {
    let timestamp_ms = frame.get("timestamp").and_then(Value::as_i64).unwrap_or(0);
    let participant = frame
        .get("participantFrames")
        .and_then(|frames| frames.get(participant_id.to_string()))?;

    let position = participant.get("position");

    let damage_stats = participant.get("damageStats");
    let champion_stats = participant.get("championStats");

    Some(FrameSnapshot {
        timestamp_ms,
        lane_cs: read_i64(participant, "minionsKilled"),
        jungle_cs: read_i64(participant, "jungleMinionsKilled"),
        total_gold: read_i64(participant, "totalGold"),
        current_gold: read_i64(participant, "currentGold"),
        gold_per_second: participant.get("goldPerSecond").and_then(Value::as_i64),
        xp: read_i64(participant, "xp"),
        level: read_i64(participant, "level") as i32,
        x: position.and_then(|p| p.get("x")).and_then(Value::as_f64),
        y: position.and_then(|p| p.get("y")).and_then(Value::as_f64),
        time_enemy_spent_controlled: participant.get("timeEnemySpentControlled").and_then(Value::as_i64),
        total_damage_done_to_champions: damage_stats
            .and_then(|d| d.get("totalDamageDoneToChampions"))
            .and_then(Value::as_i64),
        champion_stats_keys: object_key_count(champion_stats),
        damage_stats_keys: object_key_count(damage_stats),
    })
}

fn object_key_count(value: Option<&Value>) -> u32 {
    value
        .and_then(Value::as_object)
        .map(|obj| obj.len() as u32)
        .unwrap_or(0)
}

fn read_i64(value: &Value, key: &str) -> i64 {
    value.get(key).and_then(Value::as_i64).unwrap_or(0)
}

/// 时间线整体覆盖范围
pub fn timeline_span(frames: &[Value], target_participant_id: i32) -> Option<TimelineSpan> {
    if frames.is_empty() {
        return None;
    }

    let mut timestamps: Vec<i64> = frames
        .iter()
        .map(|f| f.get("timestamp").and_then(Value::as_i64).unwrap_or(0))
        .collect();
    timestamps.sort_unstable();

    let target_frame_count = frames
        .iter()
        .filter(|f| frame_snapshot(f, target_participant_id).is_some())
        .count() as u32;

    Some(TimelineSpan {
        start_ms: *timestamps.first()?,
        end_ms: *timestamps.last()?,
        frame_count: frames.len() as u32,
        target_frame_count,
    })
}

/// 计算全部存在的阶段证据
///
/// - 阶段边界是闭区间，边界帧同时作为上一阶段的终点与下一阶段的锚点，
///   这样 0-10 分钟的增量除以 10 分钟才是真实速率。
/// - 只包含真正存在目标帧的阶段：短局 / remake 不会被补出空的 mid/late。
/// - 阶段内只剩一个锚点帧时，速率为 `None`（不除零），但阶段末数值仍保留。
pub fn compute_phase_evidence(
    frames: &[Value],
    target_participant_id: i32,
    opponent_participant_id: Option<i32>,
) -> (Vec<PhaseEvidence>, PhaseComputationFlags) {
    let mut flags = PhaseComputationFlags::default();

    // 按时间排序，保证与输入顺序无关的确定性
    let mut ordered: Vec<&Value> = frames.iter().collect();
    ordered.sort_by_key(|f| f.get("timestamp").and_then(Value::as_i64).unwrap_or(0));

    let target_frames: Vec<(&Value, FrameSnapshot)> = ordered
        .iter()
        .filter_map(|frame| frame_snapshot(frame, target_participant_id).map(|snapshot| (*frame, snapshot)))
        .collect();

    flags.missing_target_frames = target_frames.len() < ordered.len();

    let mut phases = Vec::new();
    for phase in GamePhase::ALL {
        let selected: Vec<(&Value, FrameSnapshot)> = target_frames
            .iter()
            .filter(|(_, snapshot)| phase.contains(snapshot.timestamp_ms))
            .cloned()
            .collect();

        let (Some((_, first)), Some((last_frame, last))) = (selected.first(), selected.last()) else {
            continue;
        };

        let duration_ms = last.timestamp_ms - first.timestamp_ms;
        let duration_minutes = duration_ms as f64 / 60_000.0;
        let has_span = duration_ms > 0;
        if !has_span {
            flags.single_frame_phase = true;
        }

        let lane_cs = last.lane_cs - first.lane_cs;
        let jungle_cs = last.jungle_cs - first.jungle_cs;
        let total_cs = lane_cs + jungle_cs;

        let per_min = |delta: i64| -> Option<f64> {
            if has_span {
                Some(delta as f64 / duration_minutes)
            } else {
                None
            }
        };

        let opponent_diff = opponent_participant_id.and_then(|opponent_id| {
            frame_snapshot(last_frame, opponent_id).map(|opponent| phase_opponent_diff(last, &opponent, opponent_id))
        });

        phases.push(PhaseEvidence {
            phase,
            start_ms: first.timestamp_ms,
            end_ms: last.timestamp_ms,
            duration_minutes,
            frame_count: selected.len() as u32,
            lane_cs,
            jungle_cs,
            total_cs,
            lane_cs_per_min: per_min(lane_cs),
            jungle_cs_per_min: per_min(jungle_cs),
            cs_per_min: per_min(total_cs),
            gold_per_min: per_min(last.total_gold - first.total_gold),
            xp_per_min: per_min(last.xp - first.xp),
            end_gold: last.total_gold,
            end_xp: last.xp,
            end_level: last.level,
            opponent_diff,
        });
    }

    (phases, flags)
}

/// 取**对线期**（Early）的对手差
///
/// 「对线比较」只有在对线期才成立：中后期的差值反映的是团队节奏、带线与团战收益，
/// 把它当成对线结论会把「对线打崩但中期滚起来」误判成对线优势。
/// 因此这里固定取 Early，绝不退化成「最后一个有差值的阶段」。
pub fn laning_opponent_diff(phases: &[PhaseEvidence]) -> Option<&PhaseOpponentDiff> {
    phases
        .iter()
        .find(|p| p.phase == GamePhase::Early)
        .and_then(|p| p.opponent_diff.as_ref())
}

/// 阶段末的绝对差 + 归一化优势
fn phase_opponent_diff(target: &FrameSnapshot, opponent: &FrameSnapshot, opponent_id: i32) -> PhaseOpponentDiff {
    let normalized_cs_advantage = normalized_share_advantage(target.total_cs(), opponent.total_cs());
    let normalized_gold_advantage = normalized_share_advantage(target.total_gold, opponent.total_gold);
    let normalized_xp_advantage = normalized_share_advantage(target.xp, opponent.xp);

    let overall_advantage = (ADVANTAGE_WEIGHT_CS * normalized_cs_advantage
        + ADVANTAGE_WEIGHT_GOLD * normalized_gold_advantage
        + ADVANTAGE_WEIGHT_XP * normalized_xp_advantage)
        .clamp(-1.0, 1.0);

    PhaseOpponentDiff {
        opponent_participant_id: opponent_id,
        cs_diff: target.total_cs() - opponent.total_cs(),
        gold_diff: target.total_gold - opponent.total_gold,
        xp_diff: target.xp - opponent.xp,
        level_diff: target.level - opponent.level,
        normalized_cs_advantage,
        normalized_gold_advantage,
        normalized_xp_advantage,
        overall_advantage,
    }
}
