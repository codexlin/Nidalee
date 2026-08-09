//! 对线对手识别
//!
//! 优先级：
//! 1. 敌方存在**唯一**的 role+lane 完全一致玩家 → `LaneRole`
//! 2. 敌方存在**唯一**的同「统一位置」玩家 → `PositionMatch`
//! 3. 对线期帧的空间邻近（只比较敌方队伍，且只对真实分路开放）→ `SpatialProximity`
//!
//! 三条都不成立时返回 `None`。
//! 明确禁止「取敌方第一个玩家」这类会污染全部对线结论的兜底。

use serde_json::Value;

use super::position::{position_from_role_lane, EvidencePosition};
use super::timeline::{frame_snapshot, timeline_frames};
use super::types::{OpponentEvidence, OpponentMatchMethod, PHASE_EARLY_END_MS};

/// lane-role 完全一致的置信度
const CONFIDENCE_LANE_ROLE: f64 = 0.95;
/// 统一位置一致（role/lane 写法不同）的置信度
const CONFIDENCE_POSITION_MATCH: f64 = 0.85;
/// 空间邻近可接受的最大距离中位数（召唤师峡谷约 14700 单位见方）
const MAX_PROXIMITY_DISTANCE: f64 = 6000.0;
/// 空间邻近的置信度上限：永远低于 lane-role
const MAX_PROXIMITY_CONFIDENCE: f64 = 0.7;

/// 蓝方泉水中心（召唤师峡谷）
const BLUE_FOUNTAIN: (f64, f64) = (400.0, 400.0);
/// 红方泉水中心
const RED_FOUNTAIN: (f64, f64) = (14400.0, 14400.0);
/// 泉水判定半径
///
/// 阵亡等待复活、以及回城补给期间，`participantFrame.position` 会被钉在泉水，
/// 这类坐标不反映对线站位。开局 t0 更是全员在泉水，距离完全没有信息量。
const FOUNTAIN_RADIUS: f64 = 1500.0;
/// 判定对手所需的最少有效帧数
///
/// 剔除泉水 / 死亡帧后样本太少时，邻近判断本身就不可信，宁可返回 `None`。
const MIN_PROXIMITY_SAMPLES: usize = 3;

/// 参与者的基本身份信息
struct ParticipantInfo {
    participant_id: i32,
    team_id: i32,
    champion_id: i32,
    role: String,
    lane: String,
    position: EvidencePosition,
}

/// 识别对线对手
///
/// `timeline` 只在 lane-role 不可用时才被使用，因此没有时间线也能给出高置信对手。
pub fn resolve_lane_opponent(
    game: &Value,
    timeline: Option<&Value>,
    target_participant_id: i32,
    queue_id: i64,
) -> Option<OpponentEvidence> {
    let participants = collect_participants(game, queue_id);
    let target = participants
        .iter()
        .find(|p| p.participant_id == target_participant_id)?;

    let enemies: Vec<&ParticipantInfo> = participants
        .iter()
        .filter(|p| p.team_id != target.team_id && p.participant_id != target.participant_id)
        .collect();

    if enemies.is_empty() {
        return None;
    }

    if target.position.is_lane_position() {
        // 1. role + lane 完全一致，且唯一
        let exact: Vec<&ParticipantInfo> = enemies
            .iter()
            .copied()
            .filter(|e| e.role == target.role && e.lane == target.lane)
            .collect();
        if let [only] = exact.as_slice() {
            return Some(evidence(only, OpponentMatchMethod::LaneRole, CONFIDENCE_LANE_ROLE));
        }

        // 2. 统一位置一致，且唯一
        let same_position: Vec<&ParticipantInfo> = enemies
            .iter()
            .copied()
            .filter(|e| e.position == target.position)
            .collect();
        if let [only] = same_position.as_slice() {
            return Some(evidence(
                only,
                OpponentMatchMethod::PositionMatch,
                CONFIDENCE_POSITION_MATCH,
            ));
        }
    }

    // 3. 空间邻近回退：只对真正常驻某条线的位置开放
    //    打野 / ARAM / FLEX / UNKNOWN 走到这里一律返回 None，
    //    否则「最近的敌人」只是被 gank 的人或者同一条路上的所有人。
    if !target.position.allows_spatial_opponent_fallback() {
        return None;
    }

    resolve_by_proximity(timeline?, target_participant_id, &enemies)
}

fn evidence(info: &ParticipantInfo, method: OpponentMatchMethod, confidence: f64) -> OpponentEvidence {
    OpponentEvidence {
        participant_id: info.participant_id,
        team_id: info.team_id,
        champion_id: (info.champion_id != 0).then_some(info.champion_id),
        position: info.position,
        method,
        confidence,
    }
}

/// 基于对线期帧的距离**中位数**挑选最近的敌方玩家
///
/// 用中位数而不是均值：一次远程支援、一次抓下路，就能让均值偏移几千单位，
/// 而中位数只关心「大多数时间站在谁旁边」。
fn resolve_by_proximity(
    timeline: &Value,
    target_participant_id: i32,
    enemies: &[&ParticipantInfo],
) -> Option<OpponentEvidence> {
    let frames = timeline_frames(timeline)?;

    // t0 全员在泉水，距离没有任何对线信息，直接排除
    let laning: Vec<&Value> = frames
        .iter()
        .filter(|frame| {
            frame
                .get("timestamp")
                .and_then(Value::as_i64)
                .is_some_and(|ts| ts > 0 && ts <= PHASE_EARLY_END_MS)
        })
        .collect();

    if laning.is_empty() {
        return None;
    }

    // participant_id 升序遍历，保证并列时结果稳定
    let mut ordered: Vec<&ParticipantInfo> = enemies.to_vec();
    ordered.sort_by_key(|e| e.participant_id);

    let mut best: Option<(&ParticipantInfo, f64)> = None;
    for enemy in ordered {
        let Some(distance) = median_distance(&laning, target_participant_id, enemy.participant_id) else {
            continue;
        };
        let is_better = match &best {
            Some((_, current)) => distance < *current,
            None => true,
        };
        if is_better {
            best = Some((enemy, distance));
        }
    }

    let (enemy, distance) = best?;
    if distance > MAX_PROXIMITY_DISTANCE {
        return None;
    }

    let confidence =
        ((1.0 - distance / MAX_PROXIMITY_DISTANCE) * MAX_PROXIMITY_CONFIDENCE).clamp(0.05, MAX_PROXIMITY_CONFIDENCE);
    Some(evidence(enemy, OpponentMatchMethod::SpatialProximity, confidence))
}

/// 两名玩家在有效帧上的欧氏距离中位数
///
/// 有效帧的定义：双方都有坐标，且**双方都不在任一泉水范围内**。
/// 有效样本少于 [`MIN_PROXIMITY_SAMPLES`] 时返回 `None`（证据不足，不猜）。
fn median_distance(frames: &[&Value], target_participant_id: i32, enemy_participant_id: i32) -> Option<f64> {
    let mut distances: Vec<f64> = Vec::with_capacity(frames.len());

    for frame in frames {
        let (Some(target), Some(enemy)) = (
            frame_snapshot(frame, target_participant_id),
            frame_snapshot(frame, enemy_participant_id),
        ) else {
            continue;
        };

        let (Some(tx), Some(ty), Some(ex), Some(ey)) = (target.x, target.y, enemy.x, enemy.y) else {
            continue;
        };

        // 任一方在泉水（阵亡等待复活 / 回城补给）时，这一帧不反映对线站位
        if in_any_fountain(tx, ty) || in_any_fountain(ex, ey) {
            continue;
        }

        distances.push(((ex - tx).powi(2) + (ey - ty).powi(2)).sqrt());
    }

    if distances.len() < MIN_PROXIMITY_SAMPLES {
        return None;
    }

    distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(median_of_sorted(&distances))
}

/// 已排序切片的中位数（偶数个取中间两个的均值）
fn median_of_sorted(sorted: &[f64]) -> f64 {
    let len = sorted.len();
    let middle = len / 2;
    if len % 2 == 0 {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    } else {
        sorted[middle]
    }
}

/// 坐标是否落在任一方泉水范围内
fn in_any_fountain(x: f64, y: f64) -> bool {
    [BLUE_FOUNTAIN, RED_FOUNTAIN]
        .iter()
        .any(|(fx, fy)| ((x - fx).powi(2) + (y - fy).powi(2)).sqrt() <= FOUNTAIN_RADIUS)
}

/// 读取全部参与者的身份信息（位置已归一化）
fn collect_participants(game: &Value, queue_id: i64) -> Vec<ParticipantInfo> {
    let Some(participants) = game.get("participants").and_then(Value::as_array) else {
        return Vec::new();
    };

    participants
        .iter()
        .map(|participant| {
            let timeline = participant.get("timeline");
            let role = timeline
                .and_then(|t| t.get("role"))
                .and_then(Value::as_str)
                .unwrap_or("NONE")
                .trim()
                .to_ascii_uppercase();
            let lane = timeline
                .and_then(|t| t.get("lane"))
                .and_then(Value::as_str)
                .unwrap_or("NONE")
                .trim()
                .to_ascii_uppercase();

            ParticipantInfo {
                participant_id: participant.get("participantId").and_then(Value::as_i64).unwrap_or(0) as i32,
                team_id: participant.get("teamId").and_then(Value::as_i64).unwrap_or(0) as i32,
                champion_id: participant.get("championId").and_then(Value::as_i64).unwrap_or(0) as i32,
                position: position_from_role_lane(&role, &lane, queue_id),
                role,
                lane,
            }
        })
        .collect()
}
