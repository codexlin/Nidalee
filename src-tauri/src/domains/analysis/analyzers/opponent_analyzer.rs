/// 智能对手分析器
///
/// 职责：
/// - 分析对手的优缺点
/// - 识别对手的打法风格
/// - 生成针对性的战术建议
use crate::domains::analysis::analyzers::core::timeline_analyzer::TimelineAnalysis;
use crate::shared::types::PlayerMatchStats;
use serde_json::Value;

/// 对手分析结果
#[derive(Debug, Clone)]
pub struct OpponentAnalysis {
    pub participant_id: i32,
    pub champion_id: i32,
    pub lane_position: String,

    // 优缺点分析
    pub strengths: Vec<OpponentStrength>,
    pub weaknesses: Vec<OpponentWeakness>,

    // 打法风格
    pub playstyle: PlayStyle,

    // 针对性建议
    pub tactical_advice: Vec<TacticalAdvice>,
}

/// 对手优势
#[derive(Debug, Clone)]
pub struct OpponentStrength {
    pub category: String,    // 对线、团战、发育等
    pub description: String, // 具体描述
    pub evidence: String,    // 数据支撑
    pub threat_level: u8,    // 威胁等级 1-5
}

/// 对手劣势
#[derive(Debug, Clone)]
pub struct OpponentWeakness {
    pub category: String,    // 对线、团战、发育等
    pub description: String, // 具体描述
    pub evidence: String,    // 数据支撑
    pub exploit_level: u8,   // 可利用程度 1-5
}

/// 打法风格
#[derive(Debug, Clone)]
pub enum PlayStyle {
    Aggressive, // 激进型
    Passive,    // 保守型
    Roaming,    // 游走型
    Farming,    // 发育型
    Teamfight,  // 团战型
    SplitPush,  // 分推型
    Unknown,    // 未知
}

/// 战术建议
#[derive(Debug, Clone)]
pub struct TacticalAdvice {
    pub priority: u8,                // 优先级 1-5
    pub category: String,            // 建议分类
    pub title: String,               // 建议标题
    pub description: String,         // 详细描述
    pub implementation: Vec<String>, // 具体实施方法
}

/// 分析对手
pub fn analyze_opponent(
    participant_id: i32,
    match_data: &Value,
    timeline_analysis: &TimelineAnalysis,
    basic_stats: &PlayerMatchStats,
) -> OpponentAnalysis {
    let champion_id = extract_champion_id(match_data, participant_id);
    let lane_position = extract_lane_position(match_data, participant_id);

    // 分析优缺点
    let strengths = analyze_opponent_strengths(timeline_analysis, basic_stats);
    let weaknesses = analyze_opponent_weaknesses(timeline_analysis, basic_stats);

    // 识别打法风格
    let playstyle = identify_playstyle(timeline_analysis, basic_stats);

    // 生成战术建议
    let tactical_advice = generate_tactical_advice(&strengths, &weaknesses, &playstyle);

    OpponentAnalysis {
        participant_id,
        champion_id,
        lane_position,
        strengths,
        weaknesses,
        playstyle,
        tactical_advice,
    }
}

/// 分析对手优势
fn analyze_opponent_strengths(timeline: &TimelineAnalysis, stats: &PlayerMatchStats) -> Vec<OpponentStrength> {
    let mut strengths = Vec::new();

    // 1. 对线期优势分析
    if timeline.early_game.cs_difference > 5.0 {
        strengths.push(OpponentStrength {
            category: "对线期".to_string(),
            description: "对线期补刀压制能力强".to_string(),
            evidence: format!("平均领先{:.1}刀", timeline.early_game.cs_difference),
            threat_level: 4,
        });
    }

    if timeline.early_game.xp_difference > 200.0 {
        strengths.push(OpponentStrength {
            category: "对线期".to_string(),
            description: "经验获取效率高，等级压制".to_string(),
            evidence: format!("经验领先{:.0}点", timeline.early_game.xp_difference),
            threat_level: 5,
        });
    }

    // 2. 团战优势分析
    if stats.avg_kda > 3.0 {
        strengths.push(OpponentStrength {
            category: "团战期".to_string(),
            description: "团战表现优秀，KDA较高".to_string(),
            evidence: format!("平均KDA {:.2}", stats.avg_kda),
            threat_level: 4,
        });
    }

    if stats.avg_assists > 8.0 {
        strengths.push(OpponentStrength {
            category: "团战期".to_string(),
            description: "团战参与度高，团队意识强".to_string(),
            evidence: format!("平均助攻{:.1}次", stats.avg_assists),
            threat_level: 3,
        });
    }

    // 3. 发育优势分析
    if timeline.mid_game.gold_per_minute > 400.0 {
        strengths.push(OpponentStrength {
            category: "发育期".to_string(),
            description: "中期经济发育能力强".to_string(),
            evidence: format!("中期金币效率{:.0}/分钟", timeline.mid_game.gold_per_minute),
            threat_level: 3,
        });
    }

    // 4. 视野控制优势
    if stats.vspm > 1.5 {
        strengths.push(OpponentStrength {
            category: "视野控制".to_string(),
            description: "视野控制能力强".to_string(),
            evidence: format!("视野得分{:.1}/分钟", stats.vspm),
            threat_level: 2,
        });
    }

    strengths
}

/// 分析对手劣势
fn analyze_opponent_weaknesses(timeline: &TimelineAnalysis, stats: &PlayerMatchStats) -> Vec<OpponentWeakness> {
    let mut weaknesses = Vec::new();

    // 1. 对线期劣势分析
    if timeline.early_game.cs_difference < -5.0 {
        weaknesses.push(OpponentWeakness {
            category: "对线期".to_string(),
            description: "对线期补刀能力较弱".to_string(),
            evidence: format!("平均落后{:.1}刀", -timeline.early_game.cs_difference),
            exploit_level: 4,
        });
    }

    if timeline.early_game.xp_difference < -200.0 {
        weaknesses.push(OpponentWeakness {
            category: "对线期".to_string(),
            description: "经验获取效率低，等级落后".to_string(),
            evidence: format!("经验落后{:.0}点", -timeline.early_game.xp_difference),
            exploit_level: 5,
        });
    }

    // 2. 团战劣势分析
    if stats.avg_kda < 1.5 {
        weaknesses.push(OpponentWeakness {
            category: "团战期".to_string(),
            description: "团战表现较差，KDA偏低".to_string(),
            evidence: format!("平均KDA {:.2}", stats.avg_kda),
            exploit_level: 4,
        });
    }

    if stats.avg_deaths > 6.0 {
        weaknesses.push(OpponentWeakness {
            category: "团战期".to_string(),
            description: "死亡次数过多，生存能力弱".to_string(),
            evidence: format!("平均死亡{:.1}次", stats.avg_deaths),
            exploit_level: 5,
        });
    }

    // 3. 发育劣势分析
    if timeline.mid_game.gold_per_minute < 300.0 {
        weaknesses.push(OpponentWeakness {
            category: "发育期".to_string(),
            description: "中期经济发育能力弱".to_string(),
            evidence: format!("中期金币效率{:.0}/分钟", timeline.mid_game.gold_per_minute),
            exploit_level: 3,
        });
    }

    // 4. 视野控制劣势
    if stats.vspm < 0.8 {
        weaknesses.push(OpponentWeakness {
            category: "视野控制".to_string(),
            description: "视野控制能力弱".to_string(),
            evidence: format!("视野得分{:.1}/分钟", stats.vspm),
            exploit_level: 3,
        });
    }

    weaknesses
}

/// 识别打法风格
fn identify_playstyle(timeline: &TimelineAnalysis, stats: &PlayerMatchStats) -> PlayStyle {
    // 基于多个指标综合判断

    // 1. 激进型判断
    if stats.avg_kills > 8.0 && stats.avg_deaths > 5.0 {
        return PlayStyle::Aggressive;
    }

    // 2. 游走型判断
    if timeline.mid_game.gold_per_minute > timeline.early_game.gold_per_minute * 1.2 {
        return PlayStyle::Roaming;
    }

    // 3. 发育型判断
    if stats.cspm > 7.0 && stats.avg_kills < 5.0 {
        return PlayStyle::Farming;
    }

    // 4. 团战型判断
    if stats.avg_assists > 10.0 && stats.avg_kda > 2.5 {
        return PlayStyle::Teamfight;
    }

    // 5. 保守型判断
    if stats.avg_deaths < 3.0 && stats.avg_kills < 4.0 {
        return PlayStyle::Passive;
    }

    PlayStyle::Unknown
}

/// 生成战术建议
fn generate_tactical_advice(
    strengths: &[OpponentStrength],
    weaknesses: &[OpponentWeakness],
    playstyle: &PlayStyle,
) -> Vec<TacticalAdvice> {
    let mut advice = Vec::new();

    // 基于劣势生成针对性建议
    for weakness in weaknesses {
        match weakness.category.as_str() {
            "对线期" => {
                advice.push(TacticalAdvice {
                    priority: 5,
                    category: "对线策略".to_string(),
                    title: "利用对线优势".to_string(),
                    description: format!("对手{}，可以采取更激进的策略", weakness.description),
                    implementation: vec![
                        "选择压制型英雄".to_string(),
                        "主动换血和消耗".to_string(),
                        "控制兵线，限制对手发育".to_string(),
                        "呼叫打野进行Gank".to_string(),
                    ],
                });
            }
            "团战期" => {
                advice.push(TacticalAdvice {
                    priority: 4,
                    category: "团战策略".to_string(),
                    title: "团战针对".to_string(),
                    description: format!("对手{}，团战时重点针对", weakness.description),
                    implementation: vec![
                        "优先集火该对手".to_string(),
                        "利用其站位失误".to_string(),
                        "控制技能优先给该对手".to_string(),
                    ],
                });
            }
            "发育期" => {
                advice.push(TacticalAdvice {
                    priority: 3,
                    category: "发育策略".to_string(),
                    title: "发育压制".to_string(),
                    description: format!("对手{}，可以压制其发育", weakness.description),
                    implementation: vec![
                        "入侵其野区资源".to_string(),
                        "控制地图资源".to_string(),
                        "限制其经济来源".to_string(),
                    ],
                });
            }
            "视野控制" => {
                advice.push(TacticalAdvice {
                    priority: 2,
                    category: "视野策略".to_string(),
                    title: "视野压制".to_string(),
                    description: format!("对手{}，可以利用视野优势", weakness.description),
                    implementation: vec![
                        "在关键位置埋伏".to_string(),
                        "控制视野盲区".to_string(),
                        "利用视野差进行偷袭".to_string(),
                    ],
                });
            }
            _ => {}
        }
    }

    // 基于打法风格生成建议
    match playstyle {
        PlayStyle::Aggressive => {
            advice.push(TacticalAdvice {
                priority: 4,
                category: "风格应对".to_string(),
                title: "应对激进型对手".to_string(),
                description: "对手打法激进，容易上头".to_string(),
                implementation: vec![
                    "选择反打能力强的英雄".to_string(),
                    "利用其激进行为进行反杀".to_string(),
                    "保持冷静，等待机会".to_string(),
                ],
            });
        }
        PlayStyle::Passive => {
            advice.push(TacticalAdvice {
                priority: 3,
                category: "风格应对".to_string(),
                title: "应对保守型对手".to_string(),
                description: "对手打法保守，发育为主".to_string(),
                implementation: vec![
                    "主动寻找机会".to_string(),
                    "控制地图资源".to_string(),
                    "逼迫其参与团战".to_string(),
                ],
            });
        }
        PlayStyle::Roaming => {
            advice.push(TacticalAdvice {
                priority: 4,
                category: "风格应对".to_string(),
                title: "应对游走型对手".to_string(),
                description: "对手喜欢游走支援".to_string(),
                implementation: vec![
                    "做好视野，提前预警".to_string(),
                    "反游走，支援其他路".to_string(),
                    "利用其游走时机推塔".to_string(),
                ],
            });
        }
        _ => {}
    }

    // 按优先级排序
    advice.sort_by(|a, b| b.priority.cmp(&a.priority));

    advice
}

/// 提取英雄ID
fn extract_champion_id(match_data: &Value, participant_id: i32) -> i32 {
    if let Some(participants) = match_data.get("participants").and_then(|p| p.as_array()) {
        for participant in participants {
            if participant.get("participantId").and_then(|id| id.as_i64()) == Some(participant_id as i64) {
                return participant.get("championId").and_then(|id| id.as_i64()).unwrap_or(0) as i32;
            }
        }
    }
    0
}

/// 提取位置信息
fn extract_lane_position(match_data: &Value, participant_id: i32) -> String {
    if let Some(participants) = match_data.get("participants").and_then(|p| p.as_array()) {
        for participant in participants {
            if participant.get("participantId").and_then(|id| id.as_i64()) == Some(participant_id as i64) {
                if let Some(timeline) = participant.get("timeline") {
                    let role = timeline.get("role").and_then(|r| r.as_str()).unwrap_or("NONE");
                    let lane = timeline.get("lane").and_then(|l| l.as_str()).unwrap_or("NONE");

                    return match (role, lane) {
                        ("DUO_CARRY", _) => "ADC".to_string(),
                        ("DUO_SUPPORT", _) => "辅助".to_string(),
                        ("SOLO", "TOP") => "上单".to_string(),
                        ("SOLO", "MIDDLE") | ("SOLO", "MID") => "中单".to_string(),
                        ("NONE", "JUNGLE") | ("JUNGLE", _) => "打野".to_string(),
                        _ => "未知".to_string(),
                    };
                }
            }
        }
    }
    "未知".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::analysis::analyzers::core::timeline_analyzer::{
        OpponentComparison, PhaseAnalysis, TimelineAnalysis,
    };

    #[test]
    fn test_identify_playstyle() {
        let timeline = TimelineAnalysis {
            early_game: PhaseAnalysis {
                cs_per_minute: 6.0,
                gold_per_minute: 350.0,
                xp_per_minute: 500.0,
                ..Default::default()
            },
            mid_game: PhaseAnalysis {
                cs_per_minute: 5.5,
                gold_per_minute: 420.0, // 比早期高
                xp_per_minute: 450.0,
                ..Default::default()
            },
            late_game: PhaseAnalysis {
                cs_per_minute: 5.0,
                gold_per_minute: 400.0,
                xp_per_minute: 400.0,
                ..Default::default()
            },
            key_events: vec![],
            opponent_comparison: OpponentComparison::default(),
            // 三个阶段都是手工构造的零值证据：这里只测风格判定，不测阶段存在性
            phases: vec![],
        };

        let stats = PlayerMatchStats {
            avg_kills: 3.0,
            avg_deaths: 2.0,
            avg_assists: 12.0, // 修改为12.0以满足团战型的条件 (> 10.0)
            avg_kda: 5.5,
            cspm: 6.0,
            vspm: 1.2,
            ..Default::default()
        };

        let playstyle = identify_playstyle(&timeline, &stats);
        assert!(matches!(playstyle, PlayStyle::Teamfight));
    }
}
