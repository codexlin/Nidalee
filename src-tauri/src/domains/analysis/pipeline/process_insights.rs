//! 过程复盘洞察：由抽全后的 Evidence 聚合，不以胜率为主结论

use std::collections::HashMap;

use crate::domains::analysis::evidence::activity::ActivityContext;
use crate::domains::analysis::evidence::position::EvidencePosition;
use crate::domains::analysis::evidence::types::{DeathCause, EventInvolvement, EvidenceEventKind, GamePhase};
use crate::domains::analysis::evidence::{EvidenceQuality, MatchEvidence, MIN_SAMPLE_FOR_CONCLUSION};
use crate::shared::types::types::{
    ActivityBucketCount, DeathBreakdownCard, LaningProcessCard, ObjectiveProcessCard, OpponentCompareCard,
    OpponentPhaseCompare, ProcessAction, ProcessInsight, ProcessKeyMoment, VisionProcessCard,
};

/// 按位置聚合过程复盘
pub fn build_process_insight(matches: &[MatchEvidence]) -> ProcessInsight {
    let sample_size = matches.len() as u32;
    let timeline_games = matches
        .iter()
        .filter(|m| m.events.is_some() && m.quality != EvidenceQuality::TimelineMissing)
        .count() as u32;

    if timeline_games == 0 {
        return ProcessInsight {
            sample_size,
            timeline_games: 0,
            has_timeline: false,
            degradation_message: Some(
                "这些对局没有可用时间线，暂时没法做过程复盘。请确认深度分析与时间线开关已打开。".into(),
            ),
            death_breakdown: None,
            laning_process: None,
            objective_process: None,
            vision_process: None,
            actions: Vec::new(),
        };
    }

    let position = dominant_position(matches);
    let death_breakdown = death_card(matches, position);
    let laning_process = laning_card(matches);
    let objective_process = objective_card(matches);
    let vision_process = vision_card(matches, position);
    let actions = build_actions(
        &death_breakdown,
        &laning_process,
        &objective_process,
        &vision_process,
        timeline_games,
        false,
        position,
    );

    ProcessInsight {
        sample_size,
        timeline_games,
        has_timeline: true,
        degradation_message: if timeline_games < MIN_SAMPLE_FOR_CONCLUSION {
            Some(format!(
                "只有 {} 场带时间线，结论先当参考，多打几把会更稳。",
                timeline_games
            ))
        } else {
            None
        },
        death_breakdown,
        laning_process,
        objective_process,
        vision_process,
        actions,
    }
}

/// 单局过程复盘：复用卡片聚合，文案与建议按「本局」语义收紧
pub fn build_single_match_process_insight(evidence: &MatchEvidence) -> ProcessInsight {
    let mut insight = build_process_insight(std::slice::from_ref(evidence));
    insight.sample_size = 1;

    if !insight.has_timeline {
        insight.degradation_message =
            Some("本局没有可用时间线，暂无法做过程复盘。请确认深度分析与时间线开关已打开。".into());
        insight.actions.clear();
        return insight;
    }

    insight.degradation_message = None;
    let position = evidence.position;
    insight.actions = build_actions(
        &insight.death_breakdown,
        &insight.laning_process,
        &insight.objective_process,
        &insight.vision_process,
        1,
        true,
        position,
    );

    if let Some(vision) = insight.vision_process.as_mut() {
        vision.summary = single_match_vision_summary(vision.wards_placed, vision.wards_killed, position);
        vision.games_with_wards = if vision.wards_placed + vision.wards_killed > 0 {
            1
        } else {
            0
        };
    }

    insight
}

/// 单局对位对比（阶段末经济/补刀差 + 对位英雄）
pub fn build_opponent_compare(evidence: &MatchEvidence) -> Option<OpponentCompareCard> {
    let opponent = evidence.opponent.as_ref()?;

    let mut phases = Vec::new();
    for phase in GamePhase::ALL {
        let Some(pe) = evidence.phase(phase) else {
            continue;
        };
        let Some(diff) = pe.opponent_diff.as_ref() else {
            continue;
        };
        let (label, phase_key) = match phase {
            GamePhase::Early => ("前 10 分", "early"),
            GamePhase::Mid => ("10–20 分", "mid"),
            GamePhase::Late => ("20 分后", "late"),
        };
        let my_gold = pe.end_gold;
        let opponent_gold = my_gold - diff.gold_diff;
        phases.push(OpponentPhaseCompare {
            phase: phase_key.into(),
            label: label.into(),
            cs_diff: diff.cs_diff,
            gold_diff: diff.gold_diff,
            xp_diff: diff.xp_diff,
            level_diff: diff.level_diff,
            overall_advantage_pct: round1(diff.overall_advantage * 100.0),
            my_gold,
            opponent_gold,
        });
    }

    let summary = opponent_compare_summary(evidence.position, &phases);

    Some(OpponentCompareCard {
        my_champion_id: evidence.champion_id,
        opponent_champion_id: opponent.champion_id,
        opponent_participant_id: opponent.participant_id,
        position: opponent.position.as_str().into(),
        confidence: opponent.confidence,
        phases,
        summary,
    })
}

fn opponent_compare_summary(position: EvidencePosition, phases: &[OpponentPhaseCompare]) -> Option<String> {
    let early = phases.iter().find(|p| p.phase == "early");
    let focus = early.or_else(|| phases.last())?;
    let pct = focus.overall_advantage_pct;
    let when = focus.label.as_str();

    let verb = if pct >= 8.0 {
        "领先"
    } else if pct <= -8.0 {
        "落后"
    } else {
        "均势"
    };

    let role = match position {
        EvidencePosition::Jungle => "对位打野",
        EvidencePosition::Support => "对位辅助",
        _ => "对位",
    };

    if verb == "均势" {
        Some(format!(
            "{when}相对{role}大体均势（综合 {pct:+.0}%，经济 {gold:+}，补刀 {cs:+}）。",
            gold = focus.gold_diff,
            cs = focus.cs_diff
        ))
    } else {
        Some(format!(
            "{when}相对{role}{verb}约 {mag:.0}%（经济 {gold:+}，补刀 {cs:+}）。",
            mag = pct.abs(),
            gold = focus.gold_diff,
            cs = focus.cs_diff
        ))
    }
}

/// 从单局证据抽出关键时刻（死亡 / 龙先锋大龙）
pub fn build_key_moments(evidence: &MatchEvidence) -> Vec<ProcessKeyMoment> {
    let Some(events) = evidence.events.as_ref() else {
        return Vec::new();
    };

    let mut moments = Vec::new();
    for ke in &events.key_events {
        let moment = match ke.kind {
            EvidenceEventKind::ChampionDeath if ke.involvement == EventInvolvement::Victim => {
                let label = match ke.death_cause {
                    Some(DeathCause::Solo) => "被单杀",
                    // 数据含义是「击杀方有助攻」，不是「被打野抓」；打野更不该叫被抓
                    Some(DeathCause::GankOrMulti) => "多人集火",
                    Some(DeathCause::TowerOrMinion) => "塔刀/处决",
                    None => "阵亡",
                };
                Some(ProcessKeyMoment {
                    timestamp_ms: ke.timestamp_ms,
                    label: label.into(),
                    detail: ke.activity_context.map(activity_label).map(str::to_string),
                    opponent_detail: ke.opponent_activity_context.map(activity_label).map(str::to_string),
                })
            }
            EvidenceEventKind::Dragon | EvidenceEventKind::Herald | EvidenceEventKind::Baron => {
                let resource = match ke.kind {
                    EvidenceEventKind::Dragon => "小龙",
                    EvidenceEventKind::Herald => "先锋",
                    EvidenceEventKind::Baron => "大龙",
                    _ => "资源",
                };
                let label = match (ke.involvement, ke.allied_side) {
                    (EventInvolvement::Killer | EventInvolvement::Assist, _) => Some(format!("参与{resource}")),
                    (EventInvolvement::Uninvolved, Some(false)) => Some(format!("错过{resource}")),
                    (EventInvolvement::Uninvolved, Some(true)) => Some(format!("己方{resource}（未参与）")),
                    _ => None,
                };
                label.map(|label| ProcessKeyMoment {
                    timestamp_ms: ke.timestamp_ms,
                    label,
                    detail: ke
                        .activity_context
                        .map(activity_label)
                        .map(str::to_string)
                        .or_else(|| ke.detail.clone()),
                    opponent_detail: ke.opponent_activity_context.map(activity_label).map(str::to_string),
                })
            }
            _ => None,
        };

        if let Some(m) = moment {
            moments.push(m);
        }
        if moments.len() >= 24 {
            break;
        }
    }

    moments
}

fn death_card(matches: &[MatchEvidence], position: EvidencePosition) -> Option<DeathBreakdownCard> {
    let mut solo = 0u32;
    let mut multi = 0u32;
    let mut tower = 0u32;
    for m in matches {
        let Some(events) = m.events.as_ref() else {
            continue;
        };
        let counted = events.deaths_solo + events.deaths_gank_or_multi + events.deaths_tower_or_minion;
        if counted > 0 {
            solo += events.deaths_solo;
            multi += events.deaths_gank_or_multi;
            tower += events.deaths_tower_or_minion;
            continue;
        }
        for ke in &events.key_events {
            if ke.kind != EvidenceEventKind::ChampionDeath {
                continue;
            }
            match ke.death_cause {
                Some(DeathCause::Solo) => solo += 1,
                Some(DeathCause::GankOrMulti) => multi += 1,
                Some(DeathCause::TowerOrMinion) => tower += 1,
                None => {}
            }
        }
    }
    let total = solo + multi + tower;
    if total == 0 {
        return None;
    }
    let solo_rate = solo as f64 / total as f64;
    let gank_rate = multi as f64 / total as f64;
    let summary = death_summary(position, solo, multi, tower, total, solo_rate, gank_rate);

    Some(DeathBreakdownCard {
        total_deaths: total,
        solo,
        gank_or_multi: multi,
        tower_or_minion: tower,
        solo_rate,
        gank_rate,
        summary,
    })
}

fn death_summary(
    position: EvidencePosition,
    solo: u32,
    multi: u32,
    tower: u32,
    total: u32,
    solo_rate: f64,
    multi_rate: f64,
) -> String {
    if solo_rate >= 0.55 {
        return match position {
            EvidencePosition::Jungle => format!(
                "阵亡里约 {:.0}% 是被单杀（{solo}/{total}）。优先练反野换血、对位强弱期与撤退时机。",
                solo_rate * 100.0
            ),
            EvidencePosition::Support => format!(
                "阵亡里约 {:.0}% 是被单杀（{solo}/{total}）。游走前先确认自身位置安全，少在没视野处独走。",
                solo_rate * 100.0
            ),
            _ => format!(
                "阵亡里约 {:.0}% 是被单杀（{solo}/{total}）。优先练对线换血与对位细节。",
                solo_rate * 100.0
            ),
        };
    }

    if multi_rate >= 0.55 {
        return match position {
            EvidencePosition::Jungle => format!(
                "阵亡里约 {:.0}% 是多人集火（{multi}/{total}）。进野/河道前先看人数与视野，别在劣势人数下硬开。",
                multi_rate * 100.0
            ),
            EvidencePosition::Support => format!(
                "阵亡里约 {:.0}% 是多人集火（{multi}/{total}）。注意进场时机与后排站位，别先手送掉。",
                multi_rate * 100.0
            ),
            _ => format!(
                "阵亡里约 {:.0}% 是多人集火（{multi}/{total}）。优先补视野、控线权，减少被包夹。",
                multi_rate * 100.0
            ),
        };
    }

    if tower as f64 / total as f64 >= 0.4 {
        return format!("有不少塔下/处决类阵亡（{tower}/{total}），注意血量与塔下站位。");
    }

    format!("阵亡构成比较分散：单杀 {solo}、多人集火 {multi}、塔刀 {tower}（共 {total}）。")
}

fn laning_card(matches: &[MatchEvidence]) -> Option<LaningProcessCard> {
    let mut cs = Vec::new();
    let mut gold = Vec::new();
    let mut adv = Vec::new();
    for m in matches {
        let Some(early) = m.phase(GamePhase::Early) else {
            continue;
        };
        let Some(diff) = early.opponent_diff.as_ref() else {
            continue;
        };
        cs.push(diff.cs_diff as f64);
        gold.push(diff.gold_diff as f64);
        adv.push(diff.overall_advantage * 100.0);
    }
    let sample_size = cs.len() as u32;
    if sample_size == 0 {
        return None;
    }
    let avg_cs = mean(&cs);
    let avg_gold = mean(&gold);
    let avg_adv = mean(&adv);
    let summary = if avg_adv <= -8.0 {
        format!(
            "对线前 10 分钟整体落后对位约 {:.0}%（补刀差 {avg_cs:+.0}，经济差 {avg_gold:+.0}）。问题更偏前期过程。",
            avg_adv.abs()
        )
    } else if avg_adv >= 8.0 {
        format!(
            "对线前 10 分钟整体领先对位约 {avg_adv:.0}%（补刀差 {avg_cs:+.0}，经济差 {avg_gold:+.0}）。前期节奏不错。"
        )
    } else {
        format!("对线期大体均势（综合约 {avg_adv:+.0}%，补刀差 {avg_cs:+.0}，经济差 {avg_gold:+.0}）。")
    };
    Some(LaningProcessCard {
        sample_size,
        avg_cs_diff: round1(avg_cs),
        avg_gold_diff: round1(avg_gold),
        avg_overall_advantage_pct: round1(avg_adv),
        summary,
    })
}

fn objective_card(matches: &[MatchEvidence]) -> Option<ObjectiveProcessCard> {
    let mut dragons_seen = 0u32;
    let mut dragons_taken = 0u32;
    let mut dragons_missed = 0u32;
    let mut heralds_seen = 0u32;
    let mut heralds_taken = 0u32;
    let mut heralds_missed = 0u32;
    let mut barons_seen = 0u32;
    let mut barons_taken = 0u32;
    let mut barons_missed = 0u32;
    let mut activity: HashMap<ActivityContext, u32> = HashMap::new();

    for m in matches {
        let Some(events) = m.events.as_ref() else {
            continue;
        };
        dragons_seen += events.dragons_seen;
        dragons_taken += events.dragon_takedowns;
        dragons_missed += events.dragons_missed;
        heralds_seen += events.heralds_seen;
        heralds_taken += events.herald_takedowns;
        heralds_missed += events.heralds_missed;
        barons_seen += events.barons_seen;
        barons_taken += events.baron_takedowns;
        barons_missed += events.barons_missed;

        for ke in &events.key_events {
            if !matches!(
                ke.kind,
                EvidenceEventKind::Dragon | EvidenceEventKind::Herald | EvidenceEventKind::Baron
            ) {
                continue;
            }
            if ke.involvement != EventInvolvement::Uninvolved {
                continue;
            }
            if ke.allied_side != Some(false) {
                continue;
            }
            if let Some(ctx) = ke.activity_context {
                *activity.entry(ctx).or_insert(0) += 1;
            }
        }
    }

    if dragons_seen + heralds_seen + barons_seen == 0 {
        return None;
    }

    let missed_activity = activity_buckets(&activity);
    let top_miss = missed_activity.first().map(|b| b.label.as_str()).unwrap_or("未知位置");
    let missed_total = dragons_missed + heralds_missed + barons_missed;
    let summary = if missed_total == 0 {
        format!("大型资源你基本都有参与（龙 {dragons_taken}/{dragons_seen}，先锋 {heralds_taken}/{heralds_seen}）。")
    } else {
        format!(
            "敌方独拿资源约 {missed_total} 次（小龙错过 {dragons_missed}、先锋 {heralds_missed}、大龙 {barons_missed}）。错过时你多半在{top_miss}。"
        )
    };

    Some(ObjectiveProcessCard {
        dragons_seen,
        dragons_taken,
        dragons_missed,
        heralds_seen,
        heralds_taken,
        heralds_missed,
        barons_seen,
        barons_taken,
        barons_missed,
        missed_activity,
        summary,
    })
}

fn vision_card(matches: &[MatchEvidence], position: EvidencePosition) -> Option<VisionProcessCard> {
    let mut placed = 0u32;
    let mut killed = 0u32;
    let mut games = 0u32;
    for m in matches {
        let Some(events) = m.events.as_ref() else {
            continue;
        };
        if events.wards_placed > 0 || events.wards_killed > 0 {
            games += 1;
        }
        placed += events.wards_placed;
        killed += events.wards_killed;
    }
    if placed + killed == 0 {
        return None;
    }
    let tip = match position {
        EvidencePosition::Jungle => "多人集火偏多时，优先保证河蟹/进野路口有视野。",
        _ => "多人集火偏多时，优先把眼补在侧路入口。",
    };
    let summary = format!("时间线里记到插眼 {placed} 次、排眼 {killed} 次（{games} 场有眼位事件）。{tip}");
    Some(VisionProcessCard {
        wards_placed: placed,
        wards_killed: killed,
        games_with_wards: games,
        summary,
    })
}

fn single_match_vision_summary(placed: u32, killed: u32, position: EvidencePosition) -> String {
    let tip = match position {
        EvidencePosition::Jungle => "多人集火偏多时，优先保证河蟹/进野路口有视野。",
        _ => "多人集火偏多时，优先把眼补在侧路入口。",
    };
    format!("本局时间线记到插眼 {placed} 次、排眼 {killed} 次。{tip}")
}

fn build_actions(
    death: &Option<DeathBreakdownCard>,
    laning: &Option<LaningProcessCard>,
    objective: &Option<ObjectiveProcessCard>,
    vision: &Option<VisionProcessCard>,
    timeline_games: u32,
    allow_without_min_sample: bool,
    position: EvidencePosition,
) -> Vec<ProcessAction> {
    let mut actions = Vec::new();
    if !allow_without_min_sample && timeline_games < MIN_SAMPLE_FOR_CONCLUSION {
        return actions;
    }

    if let Some(d) = death {
        if d.solo_rate >= 0.55 {
            let (title, detail) = match position {
                EvidencePosition::Jungle => ("练反野与撤退", "单杀占比偏高：先摸清对位强弱期，没视野别硬反。"),
                EvidencePosition::Support => ("少独走送掉", "单杀占比偏高：游走前先确认安全，没视野少独自探。"),
                _ => ("加强对线细节", "单杀占比偏高：先摸清对位强势期，少做没视野的硬换。"),
            };
            actions.push(ProcessAction {
                key: "death_solo".into(),
                title: title.into(),
                detail: detail.into(),
                priority: 9,
            });
        } else if d.gank_rate >= 0.55 {
            let (title, detail) = match position {
                EvidencePosition::Jungle => ("别人数劣势硬开", "多人集火偏多：进野/开龙前先确认己方人数与视野。"),
                EvidencePosition::Support => ("控进场时机", "多人集火偏多：先手别太深，跟团站位靠后排。"),
                _ => ("减少被包夹", "多人集火偏多：线权不好时主动让一波，侧向插眼再推进。"),
            };
            actions.push(ProcessAction {
                key: "death_multi".into(),
                title: title.into(),
                detail: detail.into(),
                priority: 9,
            });
        }
    }

    if let Some(l) = laning {
        if l.avg_overall_advantage_pct <= -8.0 {
            actions.push(ProcessAction {
                key: "laning_behind".into(),
                title: "稳住前期补刀".into(),
                detail: format!(
                    "对线期综合落后约 {:.0}%：把「回城前清完一波」当成固定动作。",
                    l.avg_overall_advantage_pct.abs()
                ),
                priority: 8,
            });
        }
    }

    if let Some(o) = objective {
        let missed = o.dragons_missed + o.heralds_missed + o.barons_missed;
        let missed_threshold = if allow_without_min_sample { 1 } else { 3 };
        if missed >= missed_threshold {
            let where_ = o.missed_activity.first().map(|b| b.label.as_str()).unwrap_or("别处");
            actions.push(ProcessAction {
                key: "objective_timing".into(),
                title: "资源刷新前提前靠".into(),
                detail: format!("错过大型资源时你常在{where_}：刷新前 30 秒开始清线并向河道靠拢。"),
                priority: 7,
            });
        }
    }

    if let Some(v) = vision {
        if v.wards_placed < timeline_games && death.as_ref().is_some_and(|d| d.gank_rate >= 0.5) {
            let tip = match position {
                EvidencePosition::Jungle => "先保证河蟹与进野路口有一颗眼。",
                _ => "先保证侧路入口有一颗眼。",
            };
            actions.push(ProcessAction {
                key: "vision_multi".into(),
                title: "用眼换安全".into(),
                detail: format!(
                    "多人集火偏多且眼位不多（场均约 {:.1}）：{tip}",
                    v.wards_placed as f64 / timeline_games.max(1) as f64
                ),
                priority: 6,
            });
        }
    }

    actions.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.key.cmp(&b.key)));
    actions
}

fn activity_buckets(map: &HashMap<ActivityContext, u32>) -> Vec<ActivityBucketCount> {
    let mut items: Vec<ActivityBucketCount> = map
        .iter()
        .map(|(ctx, count)| ActivityBucketCount {
            activity: activity_key(*ctx).into(),
            label: activity_label(*ctx).into(),
            count: *count,
        })
        .collect();
    items.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.activity.cmp(&b.activity)));
    items
}

fn activity_key(ctx: ActivityContext) -> &'static str {
    match ctx {
        ActivityContext::Dead => "dead",
        ActivityContext::Base => "base",
        ActivityContext::OwnJungle => "ownJungle",
        ActivityContext::EnemyJungle => "enemyJungle",
        ActivityContext::RiverOrObjective => "riverOrObjective",
        ActivityContext::Lane => "lane",
        ActivityContext::Unknown => "unknown",
    }
}

fn activity_label(ctx: ActivityContext) -> &'static str {
    match ctx {
        ActivityContext::Dead => "死亡状态",
        ActivityContext::Base => "泉水/回城",
        ActivityContext::OwnJungle => "己方野区",
        ActivityContext::EnemyJungle => "敌方野区",
        ActivityContext::RiverOrObjective => "河道/资源区",
        ActivityContext::Lane => "线上",
        ActivityContext::Unknown => "未知位置",
    }
}

fn dominant_position(matches: &[MatchEvidence]) -> EvidencePosition {
    let mut counts: HashMap<EvidencePosition, u32> = HashMap::new();
    for m in matches {
        *counts.entry(m.position).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.as_str().cmp(b.0.as_str())))
        .map(|(pos, _)| pos)
        .unwrap_or(EvidencePosition::Unknown)
}

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
