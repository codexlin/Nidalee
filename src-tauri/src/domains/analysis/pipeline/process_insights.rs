//! 过程复盘洞察：由抽全后的 Evidence 聚合，不以胜率为主结论

use std::collections::HashMap;

use crate::domains::analysis::evidence::activity::ActivityContext;
use crate::domains::analysis::evidence::types::{DeathCause, EvidenceEventKind, EventInvolvement, GamePhase};
use crate::domains::analysis::evidence::{EvidenceQuality, MatchEvidence, MIN_SAMPLE_FOR_CONCLUSION};
use crate::shared::types::types::{
    ActivityBucketCount, DeathBreakdownCard, LaningProcessCard, ObjectiveProcessCard, ProcessAction, ProcessInsight,
    VisionProcessCard,
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
                "这些对局没有可用时间线，暂时没法做过程复盘。请确认深度分析与时间线开关已打开。"
                    .into(),
            ),
            death_breakdown: None,
            laning_process: None,
            objective_process: None,
            vision_process: None,
            actions: Vec::new(),
        };
    }

    let death_breakdown = death_card(matches);
    let laning_process = laning_card(matches);
    let objective_process = objective_card(matches);
    let vision_process = vision_card(matches);
    let actions = build_actions(
        &death_breakdown,
        &laning_process,
        &objective_process,
        &vision_process,
        timeline_games,
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

fn death_card(matches: &[MatchEvidence]) -> Option<DeathBreakdownCard> {
    let mut solo = 0u32;
    let mut gank = 0u32;
    let mut tower = 0u32;
    for m in matches {
        let Some(events) = m.events.as_ref() else {
            continue;
        };
        let counted = events.deaths_solo + events.deaths_gank_or_multi + events.deaths_tower_or_minion;
        if counted > 0 {
            solo += events.deaths_solo;
            gank += events.deaths_gank_or_multi;
            tower += events.deaths_tower_or_minion;
            continue;
        }
        for ke in &events.key_events {
            if ke.kind != EvidenceEventKind::ChampionDeath {
                continue;
            }
            match ke.death_cause {
                Some(DeathCause::Solo) => solo += 1,
                Some(DeathCause::GankOrMulti) => gank += 1,
                Some(DeathCause::TowerOrMinion) => tower += 1,
                None => {}
            }
        }
    }
    let total = solo + gank + tower;
    if total == 0 {
        return None;
    }
    let solo_rate = solo as f64 / total as f64;
    let gank_rate = gank as f64 / total as f64;
    let summary = if solo_rate >= 0.55 {
        format!(
            "阵亡里约 {:.0}% 是被单杀（{solo}/{total}）。优先练对线换血与对位细节。",
            solo_rate * 100.0
        )
    } else if gank_rate >= 0.55 {
        format!(
            "阵亡里约 {:.0}% 是被抓或多人集火（{gank}/{total}）。优先补防 gank、视野与线权。",
            gank_rate * 100.0
        )
    } else if tower as f64 / total as f64 >= 0.4 {
        format!("有不少塔下/处决类阵亡（{tower}/{total}），注意血量与塔下站位。")
    } else {
        format!("阵亡构成比较分散：单杀 {solo}、被抓 {gank}、塔刀 {tower}（共 {total}）。")
    };

    Some(DeathBreakdownCard {
        total_deaths: total,
        solo,
        gank_or_multi: gank,
        tower_or_minion: tower,
        solo_rate,
        gank_rate,
        summary,
    })
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
        format!(
            "大型资源你基本都有参与（龙 {dragons_taken}/{dragons_seen}，先锋 {heralds_taken}/{heralds_seen}）。"
        )
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

fn vision_card(matches: &[MatchEvidence]) -> Option<VisionProcessCard> {
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
    let summary = format!(
        "时间线里记到插眼 {placed} 次、排眼 {killed} 次（{games} 场有眼位事件）。被抓偏多时，优先把眼补在侧路入口。"
    );
    Some(VisionProcessCard {
        wards_placed: placed,
        wards_killed: killed,
        games_with_wards: games,
        summary,
    })
}

fn build_actions(
    death: &Option<DeathBreakdownCard>,
    laning: &Option<LaningProcessCard>,
    objective: &Option<ObjectiveProcessCard>,
    vision: &Option<VisionProcessCard>,
    timeline_games: u32,
) -> Vec<ProcessAction> {
    let mut actions = Vec::new();
    if timeline_games < MIN_SAMPLE_FOR_CONCLUSION {
        return actions;
    }

    if let Some(d) = death {
        if d.solo_rate >= 0.55 {
            actions.push(ProcessAction {
                key: "death_solo".into(),
                title: "加强对线细节".into(),
                detail: "单杀占比偏高：先摸清对位强势期，少做没视野的硬换。".into(),
                priority: 9,
            });
        } else if d.gank_rate >= 0.55 {
            actions.push(ProcessAction {
                key: "death_gank".into(),
                title: "先防被抓".into(),
                detail: "被抓占比偏高：线权不好时主动让一波，侧向插眼再推进。".into(),
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
        if missed >= 3 {
            let where_ = o
                .missed_activity
                .first()
                .map(|b| b.label.as_str())
                .unwrap_or("别处");
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
            actions.push(ProcessAction {
                key: "vision_gank".into(),
                title: "用眼换安全".into(),
                detail: format!(
                    "被抓偏多且眼位不多（场均约 {:.1}）：先保证侧路入口有一颗眼。",
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

fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
