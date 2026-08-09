//! 时间线事件证据（抽全）
//!
//! - 已知事件类型全部结构化入库
//! - 未知 `type` 进入 `unknown_events`，禁止静默丢弃
//! - 资源/建筑即使目标未参与也记录，并附邻近帧活动上下文
//! - 死亡拆分 solo / gank_or_multi / tower_or_minion

use serde_json::Value;
use std::collections::BTreeMap;

use super::activity::{classify_activity, nearest_snapshot_at, ActivityContext};
use super::timeline::{frame_snapshot, FrameSnapshot};
use super::types::{
    DeathCause, EventEvidence, EventInvolvement, EvidenceEventKind, KeyEventEvidence, UnknownTimelineEvent,
};

/// 抽取上下文
#[derive(Debug, Clone, Copy)]
pub struct EventExtractContext {
    pub target_participant_id: i32,
    pub target_team_id: i32,
    pub opponent_participant_id: Option<i32>,
}

/// 从全部帧中提取目标玩家相关的全量事件证据
pub fn extract_event_evidence(frames: &[Value], ctx: EventExtractContext) -> EventEvidence {
    let mut evidence = EventEvidence::default();
    let mut unknown_counts: BTreeMap<(String, i64), u32> = BTreeMap::new();

    let mut ordered: Vec<&Value> = frames.iter().collect();
    ordered.sort_by_key(|f| f.get("timestamp").and_then(Value::as_i64).unwrap_or(0));

    let target_frames: Vec<(i64, FrameSnapshot)> = ordered
        .iter()
        .filter_map(|frame| {
            frame_snapshot(frame, ctx.target_participant_id).map(|snap| (snap.timestamp_ms, snap))
        })
        .collect();

    let opponent_frames: Vec<(i64, FrameSnapshot)> = ctx
        .opponent_participant_id
        .map(|opp_id| {
            ordered
                .iter()
                .filter_map(|frame| frame_snapshot(frame, opp_id).map(|snap| (snap.timestamp_ms, snap)))
                .collect()
        })
        .unwrap_or_default();

    let opponent_team_id = ctx
        .opponent_participant_id
        .and_then(team_of_participant)
        .unwrap_or(if ctx.target_team_id == 100 { 200 } else { 100 });

    let mut last_death_ms: Option<i64> = None;
    let mut last_opponent_death_ms: Option<i64> = None;

    for frame in &ordered {
        let Some(events) = frame.get("events").and_then(Value::as_array) else {
            continue;
        };
        for event in events {
            accumulate_event(
                &mut evidence,
                &mut unknown_counts,
                event,
                ctx,
                &target_frames,
                &opponent_frames,
                opponent_team_id,
                &mut last_death_ms,
                &mut last_opponent_death_ms,
            );
        }
    }

    evidence.unknown_events = unknown_counts
        .into_iter()
        .map(|((raw_type, timestamp_ms), count)| UnknownTimelineEvent {
            raw_type,
            timestamp_ms,
            count,
        })
        .collect();

    evidence
        .key_events
        .sort_by_key(|event| (event.timestamp_ms, event.kind, event.involvement));

    evidence
}

fn accumulate_event(
    evidence: &mut EventEvidence,
    unknown_counts: &mut BTreeMap<(String, i64), u32>,
    event: &Value,
    ctx: EventExtractContext,
    target_frames: &[(i64, FrameSnapshot)],
    opponent_frames: &[(i64, FrameSnapshot)],
    opponent_team_id: i32,
    last_death_ms: &mut Option<i64>,
    last_opponent_death_ms: &mut Option<i64>,
) {
    let Some(event_type) = event.get("type").and_then(Value::as_str) else {
        return;
    };
    let timestamp_ms = event.get("timestamp").and_then(Value::as_i64).unwrap_or(0);
    let killer_id = event.get("killerId").and_then(Value::as_i64).unwrap_or(0) as i32;
    let victim_id = event.get("victimId").and_then(Value::as_i64).unwrap_or(0) as i32;
    let assistants = assisting_ids(event);
    let assisted = assistants.contains(&ctx.target_participant_id);
    let killed_by_target = killer_id != 0 && killer_id == ctx.target_participant_id;
    let (pos_x, pos_y) = event_position(event);

    match event_type {
        "CHAMPION_KILL" => {
            if killed_by_target {
                evidence.kills += 1;
                let mut ke = KeyEventEvidence::basic(
                    EvidenceEventKind::ChampionKill,
                    timestamp_ms,
                    EventInvolvement::Killer,
                    None,
                );
                ke.assistant_count = assistants.len() as u32;
                ke.position_x = pos_x;
                ke.position_y = pos_y;
                ke.bounty = read_i32(event, "bounty");
                ke.kill_streak_length = read_i32(event, "killStreakLength");
                ke.victim_damage_received_count = damage_count(event, "victimDamageReceived");
                if ctx.opponent_participant_id.is_some_and(|opp| victim_id == opp) {
                    ke.opponent_activity_context = Some(activity_at(
                        opponent_frames,
                        opponent_team_id,
                        *last_opponent_death_ms,
                        timestamp_ms,
                    ));
                }
                evidence.key_events.push(ke);
            }
            if victim_id == ctx.target_participant_id {
                evidence.deaths += 1;
                let cause = death_cause(killer_id, assistants.len());
                match cause {
                    DeathCause::Solo => evidence.deaths_solo += 1,
                    DeathCause::GankOrMulti => evidence.deaths_gank_or_multi += 1,
                    DeathCause::TowerOrMinion => evidence.deaths_tower_or_minion += 1,
                }
                let mut ke = KeyEventEvidence::basic(
                    EvidenceEventKind::ChampionDeath,
                    timestamp_ms,
                    EventInvolvement::Victim,
                    None,
                );
                ke.death_cause = Some(cause);
                ke.killer_participant_id = if killer_id != 0 { Some(killer_id) } else { None };
                ke.assistant_count = assistants.len() as u32;
                ke.lane_opponent_kill = ctx
                    .opponent_participant_id
                    .is_some_and(|opp| opp == killer_id && assistants.is_empty());
                ke.position_x = pos_x;
                ke.position_y = pos_y;
                ke.bounty = read_i32(event, "bounty");
                ke.victim_damage_received_count = damage_count(event, "victimDamageReceived");
                // 阵亡点看邻近帧位置；不吃「近期死亡态」（否则永远显示死亡状态）
                ke.activity_context = Some(activity_at(
                    target_frames,
                    ctx.target_team_id,
                    None,
                    timestamp_ms,
                ));
                if ctx.opponent_participant_id.is_some() {
                    ke.opponent_activity_context = Some(activity_at(
                        opponent_frames,
                        opponent_team_id,
                        *last_opponent_death_ms,
                        timestamp_ms,
                    ));
                }
                evidence.key_events.push(ke);
                *last_death_ms = Some(timestamp_ms);
            }
            if assisted {
                evidence.assists += 1;
                let mut ke = KeyEventEvidence::basic(
                    EvidenceEventKind::ChampionAssist,
                    timestamp_ms,
                    EventInvolvement::Assist,
                    None,
                );
                ke.killer_participant_id = if killer_id != 0 { Some(killer_id) } else { None };
                ke.assistant_count = assistants.len() as u32;
                evidence.key_events.push(ke);
            }
            if ctx
                .opponent_participant_id
                .is_some_and(|opp| victim_id == opp)
            {
                *last_opponent_death_ms = Some(timestamp_ms);
            }
        }
        "CHAMPION_SPECIAL_KILL" => {
            let actor = event
                .get("killerId")
                .or_else(|| event.get("participantId"))
                .and_then(Value::as_i64)
                .unwrap_or(0) as i32;
            if actor == ctx.target_participant_id {
                evidence.special_kills += 1;
                let mut ke = KeyEventEvidence::basic(
                    EvidenceEventKind::ChampionSpecialKill,
                    timestamp_ms,
                    EventInvolvement::Actor,
                    event
                        .get("killType")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                );
                ke.actor_participant_id = Some(actor);
                ke.position_x = pos_x;
                ke.position_y = pos_y;
                evidence.key_events.push(ke);
            }
        }
        "ELITE_MONSTER_KILL" => {
            let monster_type = event
                .get("monsterType")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_ascii_uppercase();
            let killer_team_id = event
                .get("killerTeamId")
                .and_then(Value::as_i64)
                .map(|v| v as i32)
                .or_else(|| team_of_participant(killer_id));
            let allied = killer_team_id.map(|tid| tid == ctx.target_team_id);
            let involved = killed_by_target || assisted;
            let kind = match monster_type.as_str() {
                "DRAGON" => {
                    evidence.dragons_seen += 1;
                    if involved {
                        evidence.dragon_takedowns += 1;
                    } else if allied == Some(false) {
                        evidence.dragons_missed += 1;
                    }
                    EvidenceEventKind::Dragon
                }
                "RIFTHERALD" => {
                    evidence.heralds_seen += 1;
                    if involved {
                        evidence.herald_takedowns += 1;
                    } else if allied == Some(false) {
                        evidence.heralds_missed += 1;
                    }
                    EvidenceEventKind::Herald
                }
                "BARON_NASHOR" => {
                    evidence.barons_seen += 1;
                    if involved {
                        evidence.baron_takedowns += 1;
                    } else if allied == Some(false) {
                        evidence.barons_missed += 1;
                    }
                    EvidenceEventKind::Baron
                }
                "HORDE" => {
                    if involved {
                        evidence.horde_takedowns += 1;
                    }
                    EvidenceEventKind::Horde
                }
                _ => EvidenceEventKind::EliteMonster,
            };

            let involvement = if killed_by_target {
                EventInvolvement::Killer
            } else if assisted {
                EventInvolvement::Assist
            } else {
                EventInvolvement::Uninvolved
            };

            let activity = if !involved {
                Some(activity_at(
                    target_frames,
                    ctx.target_team_id,
                    *last_death_ms,
                    timestamp_ms,
                ))
            } else {
                None
            };
            let opponent_activity = if !involved && ctx.opponent_participant_id.is_some() {
                Some(activity_at(
                    opponent_frames,
                    opponent_team_id,
                    *last_opponent_death_ms,
                    timestamp_ms,
                ))
            } else {
                None
            };

            let mut ke = KeyEventEvidence::basic(
                kind,
                timestamp_ms,
                involvement,
                event
                    .get("monsterSubType")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or(Some(monster_type)),
            );
            ke.killer_participant_id = if killer_id != 0 { Some(killer_id) } else { None };
            ke.assistant_count = assistants.len() as u32;
            ke.killer_team_id = killer_team_id;
            ke.allied_side = allied;
            ke.activity_context = activity;
            ke.opponent_activity_context = opponent_activity;
            ke.position_x = pos_x;
            ke.position_y = pos_y;
            evidence.key_events.push(ke);
        }
        "BUILDING_KILL" => {
            evidence.buildings_seen += 1;
            let involved = killed_by_target || assisted;
            if involved {
                evidence.building_takedowns += 1;
            }
            let involvement = if killed_by_target {
                EventInvolvement::Killer
            } else if assisted {
                EventInvolvement::Assist
            } else {
                EventInvolvement::Uninvolved
            };
            let activity = if !involved {
                Some(activity_at(
                    target_frames,
                    ctx.target_team_id,
                    *last_death_ms,
                    timestamp_ms,
                ))
            } else {
                None
            };
            let opponent_activity = if !involved && ctx.opponent_participant_id.is_some() {
                Some(activity_at(
                    opponent_frames,
                    opponent_team_id,
                    *last_opponent_death_ms,
                    timestamp_ms,
                ))
            } else {
                None
            };
            let mut ke = KeyEventEvidence::basic(
                EvidenceEventKind::Building,
                timestamp_ms,
                involvement,
                event
                    .get("buildingType")
                    .or_else(|| event.get("towerType"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
            );
            ke.activity_context = activity;
            ke.opponent_activity_context = opponent_activity;
            ke.killer_team_id = event
                .get("teamId")
                .and_then(Value::as_i64)
                .map(|v| v as i32);
            evidence.key_events.push(ke);
        }
        "TURRET_PLATE_DESTROYED" => {
            let actor = event
                .get("killerId")
                .or_else(|| event.get("participantId"))
                .and_then(Value::as_i64)
                .unwrap_or(0) as i32;
            if actor == 0 || actor == ctx.target_participant_id {
                evidence.turret_plates += 1;
            }
            let involvement = if actor == ctx.target_participant_id {
                EventInvolvement::Actor
            } else {
                EventInvolvement::Uninvolved
            };
            let mut ke = KeyEventEvidence::basic(
                EvidenceEventKind::TurretPlate,
                timestamp_ms,
                involvement,
                event
                    .get("laneType")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            );
            ke.actor_participant_id = if actor != 0 { Some(actor) } else { None };
            evidence.key_events.push(ke);
        }
        "WARD_PLACED" => {
            let creator = event
                .get("creatorId")
                .or_else(|| event.get("participantId"))
                .and_then(Value::as_i64)
                .unwrap_or(0) as i32;
            if creator == ctx.target_participant_id {
                evidence.wards_placed += 1;
                let mut ke = KeyEventEvidence::basic(
                    EvidenceEventKind::WardPlaced,
                    timestamp_ms,
                    EventInvolvement::Actor,
                    event
                        .get("wardType")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                );
                ke.actor_participant_id = Some(creator);
                evidence.key_events.push(ke);
            }
        }
        "WARD_KILL" => {
            let killer = event.get("killerId").and_then(Value::as_i64).unwrap_or(0) as i32;
            if killer == ctx.target_participant_id {
                evidence.wards_killed += 1;
                let mut ke = KeyEventEvidence::basic(
                    EvidenceEventKind::WardKill,
                    timestamp_ms,
                    EventInvolvement::Killer,
                    event
                        .get("wardType")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                );
                ke.killer_participant_id = Some(killer);
                evidence.key_events.push(ke);
            }
        }
        "ITEM_PURCHASED" => {
            if push_item_event(
                evidence,
                event,
                ctx.target_participant_id,
                timestamp_ms,
                EvidenceEventKind::ItemPurchased,
            ) {
                evidence.items_purchased += 1;
            }
        }
        "ITEM_SOLD" => {
            if push_item_event(
                evidence,
                event,
                ctx.target_participant_id,
                timestamp_ms,
                EvidenceEventKind::ItemSold,
            ) {
                evidence.items_sold += 1;
            }
        }
        "ITEM_DESTROYED" => {
            if push_item_event(
                evidence,
                event,
                ctx.target_participant_id,
                timestamp_ms,
                EvidenceEventKind::ItemDestroyed,
            ) {
                evidence.items_destroyed += 1;
            }
        }
        "ITEM_UNDO" => {
            if push_item_event(
                evidence,
                event,
                ctx.target_participant_id,
                timestamp_ms,
                EvidenceEventKind::ItemUndo,
            ) {
                evidence.items_undo += 1;
            }
        }
        "SKILL_LEVEL_UP" => {
            let pid = event
                .get("participantId")
                .and_then(Value::as_i64)
                .unwrap_or(0) as i32;
            if pid == ctx.target_participant_id {
                evidence.skill_level_ups += 1;
                let mut ke = KeyEventEvidence::basic(
                    EvidenceEventKind::SkillLevelUp,
                    timestamp_ms,
                    EventInvolvement::Actor,
                    event
                        .get("levelUpType")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                );
                ke.actor_participant_id = Some(pid);
                ke.skill_slot = read_i32(event, "skillSlot");
                evidence.key_events.push(ke);
            }
        }
        "LEVEL_UP" => {
            let pid = event
                .get("participantId")
                .and_then(Value::as_i64)
                .unwrap_or(0) as i32;
            if pid == ctx.target_participant_id {
                evidence.level_ups += 1;
                let mut ke = KeyEventEvidence::basic(
                    EvidenceEventKind::LevelUp,
                    timestamp_ms,
                    EventInvolvement::Actor,
                    None,
                );
                ke.actor_participant_id = Some(pid);
                evidence.key_events.push(ke);
            }
        }
        "DRAGON_SOUL_GIVEN" => {
            evidence.dragon_souls += 1;
            let team = event.get("teamId").and_then(Value::as_i64).map(|v| v as i32);
            let mut ke = KeyEventEvidence::basic(
                EvidenceEventKind::DragonSoul,
                timestamp_ms,
                if team == Some(ctx.target_team_id) {
                    EventInvolvement::Actor
                } else {
                    EventInvolvement::Uninvolved
                },
                event
                    .get("name")
                    .or_else(|| event.get("dragonType"))
                    .and_then(Value::as_str)
                    .map(str::to_string),
            );
            ke.killer_team_id = team;
            ke.allied_side = team.map(|t| t == ctx.target_team_id);
            evidence.key_events.push(ke);
        }
        "GAME_END" => {
            evidence.key_events.push(KeyEventEvidence::basic(
                EvidenceEventKind::GameEnd,
                timestamp_ms,
                EventInvolvement::Uninvolved,
                event
                    .get("winningTeam")
                    .map(|v| v.to_string()),
            ));
        }
        "PAUSE_END" => {
            evidence.key_events.push(KeyEventEvidence::basic(
                EvidenceEventKind::PauseEnd,
                timestamp_ms,
                EventInvolvement::Uninvolved,
                None,
            ));
        }
        other => {
            *unknown_counts
                .entry((other.to_string(), timestamp_ms))
                .or_insert(0) += 1;
            evidence.key_events.push(KeyEventEvidence::basic(
                EvidenceEventKind::Unknown,
                timestamp_ms,
                EventInvolvement::Uninvolved,
                Some(other.to_string()),
            ));
        }
    }
}

fn push_item_event(
    evidence: &mut EventEvidence,
    event: &Value,
    target_id: i32,
    timestamp_ms: i64,
    kind: EvidenceEventKind,
) -> bool {
    let pid = event
        .get("participantId")
        .and_then(Value::as_i64)
        .unwrap_or(0) as i32;
    if pid != target_id {
        return false;
    }
    let mut ke = KeyEventEvidence::basic(kind, timestamp_ms, EventInvolvement::Actor, None);
    ke.actor_participant_id = Some(pid);
    ke.item_id = read_i32(event, "itemId");
    evidence.key_events.push(ke);
    true
}

fn death_cause(killer_id: i32, assistant_count: usize) -> DeathCause {
    if killer_id == 0 {
        DeathCause::TowerOrMinion
    } else if assistant_count == 0 {
        DeathCause::Solo
    } else {
        DeathCause::GankOrMulti
    }
}

fn activity_at(
    target_frames: &[(i64, FrameSnapshot)],
    team_id: i32,
    last_death_ms: Option<i64>,
    event_ms: i64,
) -> ActivityContext {
    classify_activity(
        nearest_snapshot_at(target_frames, event_ms),
        team_id,
        last_death_ms,
        event_ms,
    )
}

/// 粗略：1-5 蓝方，6-10 红方（召唤师峡谷标准）
fn team_of_participant(participant_id: i32) -> Option<i32> {
    match participant_id {
        1..=5 => Some(100),
        6..=10 => Some(200),
        _ => None,
    }
}

fn assisting_ids(event: &Value) -> Vec<i32> {
    event
        .get("assistingParticipantIds")
        .and_then(Value::as_array)
        .map(|ids| ids.iter().filter_map(Value::as_i64).map(|id| id as i32).collect())
        .unwrap_or_default()
}

fn event_position(event: &Value) -> (Option<f64>, Option<f64>) {
    let position = event.get("position");
    (
        position.and_then(|p| p.get("x")).and_then(Value::as_f64),
        position.and_then(|p| p.get("y")).and_then(Value::as_f64),
    )
}

fn read_i32(value: &Value, key: &str) -> Option<i32> {
    value.get(key).and_then(Value::as_i64).map(|v| v as i32)
}

fn damage_count(event: &Value, key: &str) -> Option<u32> {
    event
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| arr.len() as u32)
}
