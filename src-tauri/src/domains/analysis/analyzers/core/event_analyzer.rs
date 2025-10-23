/// 事件分析器
///
/// 职责：
/// - 分析游戏事件（击杀、推塔、打龙）
/// - 统计玩家参与度
/// - 识别关键事件
use crate::domains::analysis::analyzers::core::timeline_parser::{TimelineFrame, GameEvent};
use std::collections::HashMap;

/// 事件统计
#[derive(Debug, Clone, Default)]
pub struct EventStatistics {
    pub kills: usize,
    pub deaths: usize,
    pub assists: usize,
    pub tower_kills: usize,
    pub dragon_kills: usize,
    pub baron_kills: usize,
    pub first_blood: bool,
}

/// 关键时刻
#[derive(Debug, Clone)]
pub struct KeyMoment {
    pub timestamp: i64,
    pub event_type: String,
    pub description: String,
    pub impact_score: f64,  // 影响分数 0-10
}

/// 事件分析器
pub struct EventAnalyzer;

impl EventAnalyzer {
    /// 分析玩家事件
    pub fn analyze_player_events(
        &self,
        player_id: i32,
        frames: &[TimelineFrame],
    ) -> EventStatistics {
        let mut stats = EventStatistics::default();

        for frame in frames {
            for event in &frame.events {
                match event.event_type.as_str() {
                    "CHAMPION_KILL" => {
                        if let Some(killer_id) = event.killer_id {
                            if killer_id == player_id {
                                stats.kills += 1;
                            }
                        }

                        if let Some(victim_id) = event.victim_id {
                            if victim_id == player_id {
                                stats.deaths += 1;
                            }
                        }

                        if event.assisting_participant_ids.contains(&player_id) {
                            stats.assists += 1;
                        }

                        // 检查是否是一血
                        if event.timestamp < 120000 && stats.kills == 1 {
                            stats.first_blood = true;
                        }
                    },
                    "BUILDING_KILL" => {
                        if let Some(killer_id) = event.killer_id {
                            if killer_id == player_id || event.assisting_participant_ids.contains(&player_id) {
                                stats.tower_kills += 1;
                            }
                        }
                    },
                    "ELITE_MONSTER_KILL" => {
                        if let Some(monster_type) = &event.monster_type {
                            match monster_type.as_str() {
                                "DRAGON" => {
                                    if let Some(killer_id) = event.killer_id {
                                        if killer_id == player_id || event.assisting_participant_ids.contains(&player_id) {
                                            stats.dragon_kills += 1;
                                        }
                                    }
                                },
                                "BARON_NASHOR" | "RIFTHERALD" => {
                                    if let Some(killer_id) = event.killer_id {
                                        if killer_id == player_id || event.assisting_participant_ids.contains(&player_id) {
                                            stats.baron_kills += 1;
                                        }
                                    }
                                },
                                _ => {},
                            }
                        }
                    },
                    _ => {},
                }
            }
        }

        stats
    }

    /// 识别关键时刻
    pub fn identify_key_moments(
        &self,
        player_id: i32,
        frames: &[TimelineFrame],
    ) -> Vec<KeyMoment> {
        let mut moments = Vec::new();

        for frame in frames {
            for event in &frame.events {
                if let Some(moment) = self.analyze_event_importance(player_id, event, frame.timestamp) {
                    moments.push(moment);
                }
            }
        }

        // 按影响分数排序
        moments.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());

        // 只返回前10个最重要的时刻
        moments.truncate(10);

        moments
    }

    /// 分析事件重要性
    fn analyze_event_importance(
        &self,
        player_id: i32,
        event: &GameEvent,
        timestamp: i64,
    ) -> Option<KeyMoment> {
        match event.event_type.as_str() {
            "CHAMPION_KILL" => {
                if let (Some(killer_id), Some(victim_id)) = (event.killer_id, event.victim_id) {
                    if killer_id == player_id {
                        let impact = self.calculate_kill_impact(timestamp, &event.assisting_participant_ids);
                        return Some(KeyMoment {
                            timestamp,
                            event_type: "击杀".to_string(),
                            description: format!("击杀敌方英雄 ({}助攻)", event.assisting_participant_ids.len()),
                            impact_score: impact,
                        });
                    } else if victim_id == player_id {
                        return Some(KeyMoment {
                            timestamp,
                            event_type: "阵亡".to_string(),
                            description: "被敌方击杀".to_string(),
                            impact_score: 3.0, // 阵亡是负面事件
                        });
                    } else if event.assisting_participant_ids.contains(&player_id) {
                        return Some(KeyMoment {
                            timestamp,
                            event_type: "助攻".to_string(),
                            description: "参与击杀".to_string(),
                            impact_score: 4.0,
                        });
                    }
                }
            },
            "ELITE_MONSTER_KILL" => {
                if let Some(monster_type) = &event.monster_type {
                    let is_involved = event.killer_id == Some(player_id)
                        || event.assisting_participant_ids.contains(&player_id);

                    if is_involved {
                        let (description, impact) = match monster_type.as_str() {
                            "BARON_NASHOR" => ("击杀大龙".to_string(), 10.0),
                            "DRAGON" => {
                                let sub_type = event.monster_sub_type.as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("普通龙");
                                (format!("击杀{}", sub_type), 7.0)
                            },
                            "RIFTHERALD" => ("击杀峡谷先锋".to_string(), 6.0),
                            _ => return None,
                        };

                        return Some(KeyMoment {
                            timestamp,
                            event_type: "史诗野怪".to_string(),
                            description,
                            impact_score: impact,
                        });
                    }
                }
            },
            "BUILDING_KILL" => {
                if event.killer_id == Some(player_id) || event.assisting_participant_ids.contains(&player_id) {
                    return Some(KeyMoment {
                        timestamp,
                        event_type: "推塔".to_string(),
                        description: "摧毁敌方防御塔".to_string(),
                        impact_score: 5.0,
                    });
                }
            },
            _ => {},
        }

        None
    }

    /// 计算击杀影响力
    fn calculate_kill_impact(&self, timestamp: i64, assists: &[i32]) -> f64 {
        let mut impact: f64 = 6.0; // 基础影响力

        // 游戏早期的击杀更重要
        if timestamp < 180000 { // 3分钟内
            impact += 2.0;
        } else if timestamp < 600000 { // 10分钟内
            impact += 1.0;
        }

        // 单杀更有影响力
        if assists.is_empty() {
            impact += 2.0;
        }

        impact.min(10.0_f64)
    }

    /// 计算团队参与度
    pub fn calculate_participation_rate(
        &self,
        player_stats: &EventStatistics,
        team_total_kills: usize,
    ) -> f64 {
        if team_total_kills == 0 {
            return 0.0;
        }

        let player_involvement = player_stats.kills + player_stats.assists;
        (player_involvement as f64 / team_total_kills as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::analysis::analyzers::core::timeline_parser::{GameEvent, Position};

    #[test]
    fn test_calculate_kill_impact() {
        let analyzer = EventAnalyzer;

        // 早期单杀
        let early_solo_kill = analyzer.calculate_kill_impact(120000, &vec![]);
        assert!(early_solo_kill > 8.0);

        // 中期团战击杀
        let mid_team_kill = analyzer.calculate_kill_impact(600000, &vec![2, 3]);
        assert!(mid_team_kill < early_solo_kill);
    }

    #[test]
    fn test_calculate_participation_rate() {
        let analyzer = EventAnalyzer;
        let stats = EventStatistics {
            kills: 5,
            assists: 10,
            ..Default::default()
        };

        let participation = analyzer.calculate_participation_rate(&stats, 20);
        assert!((participation - 75.0).abs() < 0.01);
    }
}

