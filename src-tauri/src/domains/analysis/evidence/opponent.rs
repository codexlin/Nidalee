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

use super::position::{matchup_position_from_role_lane, EvidencePosition};
use super::timeline::{frame_snapshot, timeline_frames};
use super::types::{OpponentEvidence, OpponentMatchMethod, PHASE_EARLY_END_MS};
use crate::domains::analysis::queue_config::QueueType;

/// lane-role 完全一致的置信度
const CONFIDENCE_LANE_ROLE: f64 = 0.95;
/// 统一位置一致（role/lane 写法不同）的置信度
const CONFIDENCE_POSITION_MATCH: f64 = 0.85;
/// 惩戒互认（打野事实信号）
const CONFIDENCE_SMITE_MATCH: f64 = 0.88;
/// 空间邻近可接受的最大距离中位数（召唤师峡谷约 14700 单位见方）
const MAX_PROXIMITY_DISTANCE: f64 = 6000.0;
/// 空间邻近的置信度上限：永远低于 lane-role
const MAX_PROXIMITY_CONFIDENCE: f64 = 0.7;
/// 召唤师技能：惩戒
const SMITE_SPELL_ID: i64 = 11;

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
    has_smite: bool,
    /// `stats.neutralMinionsKilled`（打野消歧）
    jungle_cs: i64,
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

    // 过程复盘 / 对位只服务排位（420/440）；匹配与娱乐局不做对位
    if !QueueType::from_queue_id(queue_id as i32).is_ranked() {
        return None;
    }

    // 大乱斗无分路对位
    if target.position == EvidencePosition::Aram {
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

        // 2. 统一位置一致，且唯一（本局位置 → 对面同位置）
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

        // 2a. 同位置多人时，用惩戒 / 野刀消歧（常见于 LCU 把多人标成 JUNGLE）
        if let Some(only) = disambiguate_same_position(&same_position, target) {
            return Some(evidence(
                only,
                OpponentMatchMethod::PositionMatch,
                CONFIDENCE_SMITE_MATCH,
            ));
        }

        // 2b. 同位置字面回退
        let pos_key = target.position.as_str();
        let literal: Vec<&ParticipantInfo> = enemies
            .iter()
            .copied()
            .filter(|e| {
                e.position == target.position
                    || e.role.eq_ignore_ascii_case(pos_key)
                    || e.lane.eq_ignore_ascii_case(pos_key)
                    || (target.position == EvidencePosition::Adc
                        && (e.role.eq_ignore_ascii_case("BOTTOM")
                            || e.lane.eq_ignore_ascii_case("BOTTOM")
                            || e.role.eq_ignore_ascii_case("DUO_CARRY")))
                    || (target.position == EvidencePosition::Support
                        && (e.role.eq_ignore_ascii_case("UTILITY")
                            || e.role.eq_ignore_ascii_case("DUO_SUPPORT")))
                    || (target.position == EvidencePosition::Mid
                        && (e.role.eq_ignore_ascii_case("MIDDLE")
                            || e.lane.eq_ignore_ascii_case("MIDDLE")))
            })
            .collect();
        if let [only] = literal.as_slice() {
            return Some(evidence(
                only,
                OpponentMatchMethod::PositionMatch,
                CONFIDENCE_POSITION_MATCH,
            ));
        }
    }

    // 2c. 打野事实信号：惩戒 / 打野位 / 野刀（位置字段缺失时仍能找到对面打野）
    if let Some(opp) = resolve_jungler_opponent(target, &enemies) {
        return Some(opp);
    }

    // 3. 空间邻近回退：只对真正常驻某条线的位置开放
    //    打野不用邻近（最近敌人常是被 gank 的线上）
    if !target.position.allows_spatial_opponent_fallback() {
        return None;
    }

    resolve_by_proximity(timeline?, target_participant_id, &enemies)
}

/// 目标是打野（或带惩戒）时，用敌方惩戒 / 打野位 / 野刀找唯一对位
fn resolve_jungler_opponent(
    target: &ParticipantInfo,
    enemies: &[&ParticipantInfo],
) -> Option<OpponentEvidence> {
    if !(target.has_smite || target.position == EvidencePosition::Jungle) {
        return None;
    }

    let smite_enemies: Vec<&ParticipantInfo> =
        enemies.iter().copied().filter(|e| e.has_smite).collect();
    if let [only] = smite_enemies.as_slice() {
        return Some(evidence(
            only,
            OpponentMatchMethod::PositionMatch,
            CONFIDENCE_SMITE_MATCH,
        ));
    }

    let jungle_pos: Vec<&ParticipantInfo> = enemies
        .iter()
        .copied()
        .filter(|e| e.position == EvidencePosition::Jungle)
        .collect();
    if let [only] = jungle_pos.as_slice() {
        return Some(evidence(
            only,
            OpponentMatchMethod::PositionMatch,
            CONFIDENCE_POSITION_MATCH,
        ));
    }

    // 双惩戒等歧义：带惩戒的人里再按打野位 / 野刀消歧
    if let Some(only) = disambiguate_jungler_candidates(&smite_enemies) {
        return Some(evidence(
            only,
            OpponentMatchMethod::PositionMatch,
            CONFIDENCE_SMITE_MATCH,
        ));
    }

    let jungler_like: Vec<&ParticipantInfo> = enemies
        .iter()
        .copied()
        .filter(|e| e.has_smite || e.position == EvidencePosition::Jungle)
        .collect();
    if let Some(only) = disambiguate_jungler_candidates(&jungler_like) {
        return Some(evidence(
            only,
            OpponentMatchMethod::PositionMatch,
            CONFIDENCE_SMITE_MATCH,
        ));
    }

    // 敌方位置/惩戒全糊：野刀遥遥领先的唯一敌人
    clear_jungle_cs_leader(enemies).map(|only| {
        evidence(
            only,
            OpponentMatchMethod::PositionMatch,
            CONFIDENCE_SMITE_MATCH * 0.9,
        )
    })
}

fn disambiguate_same_position<'a>(
    same_position: &[&'a ParticipantInfo],
    target: &ParticipantInfo,
) -> Option<&'a ParticipantInfo> {
    if same_position.len() < 2 {
        return None;
    }
    if target.position == EvidencePosition::Jungle || target.has_smite {
        return disambiguate_jungler_candidates(same_position);
    }
    None
}

fn disambiguate_jungler_candidates<'a>(candidates: &[&'a ParticipantInfo]) -> Option<&'a ParticipantInfo> {
    if let [only] = candidates {
        return Some(only);
    }
    if candidates.len() < 2 {
        return None;
    }

    let tagged: Vec<&ParticipantInfo> = candidates
        .iter()
        .copied()
        .filter(|e| e.position == EvidencePosition::Jungle)
        .collect();
    if let [only] = tagged.as_slice() {
        return Some(only);
    }

    clear_jungle_cs_leader(candidates)
}

/// 野刀明显最高的唯一候选人（避免把摸了几个野的线上当成打野）
fn clear_jungle_cs_leader<'a>(candidates: &[&'a ParticipantInfo]) -> Option<&'a ParticipantInfo> {
    if candidates.is_empty() {
        return None;
    }
    let mut ranked: Vec<&ParticipantInfo> = candidates.to_vec();
    ranked.sort_by_key(|p| std::cmp::Reverse((p.jungle_cs, -p.participant_id)));
    let best = ranked[0];
    if best.jungle_cs < 40 {
        return None;
    }
    let second_cs = ranked.get(1).map(|p| p.jungle_cs).unwrap_or(0);
    if best.jungle_cs >= second_cs + 30 {
        Some(best)
    } else {
        None
    }
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

/// 读取全部参与者的身份信息（对位用位置，不塌成 FLEX）
///
/// 数据源优先级：
/// 1. `timeline.role` / `timeline.lane`
/// 2. `teamPosition` / `individualPosition`
/// 3. 惩戒 → 打野
/// 4. 同队野刀遥遥领先 → 打野（字段全糊时）
///
/// 对位原则：本局位置 → 敌方**唯一**同位置（上/野/中/射/辅）。
fn collect_participants(game: &Value, queue_id: i64) -> Vec<ParticipantInfo> {
    let Some(participants) = game.get("participants").and_then(Value::as_array) else {
        return Vec::new();
    };

    let mut infos: Vec<ParticipantInfo> = participants
        .iter()
        .map(|participant| {
            let timeline = participant.get("timeline");
            let mut role = timeline
                .and_then(|t| t.get("role"))
                .and_then(Value::as_str)
                .unwrap_or("NONE")
                .trim()
                .to_ascii_uppercase();
            let mut lane = timeline
                .and_then(|t| t.get("lane"))
                .and_then(Value::as_str)
                .unwrap_or("NONE")
                .trim()
                .to_ascii_uppercase();

            let team_position = participant
                .get("teamPosition")
                .or_else(|| participant.get("individualPosition"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .trim()
                .to_ascii_uppercase();

            if !team_position.is_empty() && team_position != "NONE" && team_position != "INVALID" {
                if role == "NONE" || role.is_empty() {
                    role = team_position.clone();
                }
                if lane == "NONE" || lane.is_empty() {
                    lane = team_position.clone();
                }
            }

            let has_smite = participant_has_smite(participant);
            let jungle_cs = participant_jungle_cs(participant);

            // 对位位置：仅排位解析五分路（非排位 → FLEX）
            let mut position = matchup_position_from_role_lane(&role, &lane, queue_id);
            if position == EvidencePosition::Unknown
                && !team_position.is_empty()
                && team_position != "NONE"
                && team_position != "INVALID"
            {
                position = matchup_position_from_role_lane(&team_position, &team_position, queue_id);
            }
            if position == EvidencePosition::Unknown && has_smite {
                position = EvidencePosition::Jungle;
            }

            ParticipantInfo {
                participant_id: json_i64(participant.get("participantId")).unwrap_or(0) as i32,
                team_id: json_i64(participant.get("teamId")).unwrap_or(0) as i32,
                champion_id: json_i64(participant.get("championId")).unwrap_or(0) as i32,
                position,
                role,
                lane,
                has_smite,
                jungle_cs,
            }
        })
        .collect();

    infer_jungle_from_team_cs(&mut infos);
    infos
}

/// 同队唯一「野刀遥遥领先」且位置未知的人，补标为打野
fn infer_jungle_from_team_cs(infos: &mut [ParticipantInfo]) {
    let team_ids: Vec<i32> = {
        let mut ids: Vec<i32> = infos.iter().map(|p| p.team_id).collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    };

    for team_id in team_ids {
        let unknowns: Vec<usize> = infos
            .iter()
            .enumerate()
            .filter(|(_, p)| p.team_id == team_id && p.position == EvidencePosition::Unknown)
            .map(|(i, _)| i)
            .collect();
        if unknowns.is_empty() {
            continue;
        }
        let refs: Vec<&ParticipantInfo> = unknowns.iter().map(|&i| &infos[i]).collect();
        if let Some(leader) = clear_jungle_cs_leader(&refs) {
            let id = leader.participant_id;
            if let Some(p) = infos.iter_mut().find(|p| p.participant_id == id) {
                p.position = EvidencePosition::Jungle;
            }
        }
    }
}

fn participant_has_smite(participant: &Value) -> bool {
    participant_spell_ids(participant)
        .into_iter()
        .any(|id| id == SMITE_SPELL_ID)
}

fn participant_spell_ids(participant: &Value) -> [i64; 2] {
    let root1 = json_i64(participant.get("spell1Id")).or_else(|| json_i64(participant.get("summoner1Id")));
    let root2 = json_i64(participant.get("spell2Id")).or_else(|| json_i64(participant.get("summoner2Id")));
    let stats = participant.get("stats");
    let stats1 = stats
        .and_then(|s| json_i64(s.get("spell1Id")).or_else(|| json_i64(s.get("summoner1Id"))));
    let stats2 = stats
        .and_then(|s| json_i64(s.get("spell2Id")).or_else(|| json_i64(s.get("summoner2Id"))));
    [root1.or(stats1).unwrap_or(0), root2.or(stats2).unwrap_or(0)]
}

fn participant_jungle_cs(participant: &Value) -> i64 {
    json_i64(
        participant
            .get("stats")
            .and_then(|s| s.get("neutralMinionsKilled"))
            .or_else(|| participant.get("neutralMinionsKilled")),
    )
    .unwrap_or(0)
}

/// LCU / 部分序列化会把整数写成浮点，`Value::as_i64` 会丢
fn json_i64(value: Option<&Value>) -> Option<i64> {
    let value = value?;
    value
        .as_i64()
        .or_else(|| value.as_u64().and_then(|n| i64::try_from(n).ok()))
        .or_else(|| value.as_f64().map(|n| n as i64))
}
