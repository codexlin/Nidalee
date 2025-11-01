/// 智能队友分析器
///
/// 职责：
/// - 分析队友的优缺点和配合能力
/// - 识别队友的打法风格和习惯
/// - 生成团队配合建议
use crate::domains::analysis::analyzers::core::timeline_analyzer::{
    TimelineAnalysis
};
use crate::shared::types::PlayerMatchStats;
use serde_json::Value;

/// 队友分析结果
#[derive(Debug, Clone)]
pub struct TeammateAnalysis {
    pub participant_id: i32,
    pub champion_id: i32,
    pub lane_position: String,

    // 能力分析
    pub strengths: Vec<TeammateStrength>,
    pub weaknesses: Vec<TeammateWeakness>,

    // 配合能力
    pub cooperation_style: CooperationStyle,
    pub synergy_score: f64,  // 配合度评分 0-100

    // 配合建议
    pub cooperation_advice: Vec<CooperationAdvice>,
}

/// 队友优势
#[derive(Debug, Clone)]
pub struct TeammateStrength {
    pub category: String,      // 对线、团战、发育、视野等
    pub description: String,   // 具体描述
    pub evidence: String,      // 数据支撑
    pub reliability: u8,      // 可靠性 1-5
}

/// 队友劣势
#[derive(Debug, Clone)]
pub struct TeammateWeakness {
    pub category: String,      // 对线、团战、发育、视野等
    pub description: String,   // 具体描述
    pub evidence: String,      // 数据支撑
    pub support_needed: u8,   // 需要支持程度 1-5
}

/// 配合风格
#[derive(Debug, Clone)]
pub enum CooperationStyle {
    Proactive,     // 主动配合型
    Reactive,      // 被动配合型
    Independent,   // 独立型
    Supportive,    // 支援型
    Carry,         // 核心型
    Unknown,       // 未知
}

/// 配合建议
#[derive(Debug, Clone)]
pub struct CooperationAdvice {
    pub priority: u8,         // 优先级 1-5
    pub category: String,     // 建议分类
    pub title: String,        // 建议标题
    pub description: String,  // 详细描述
    pub actions: Vec<String>, // 具体行动建议
}

/// 分析队友
pub fn analyze_teammate(
    participant_id: i32,
    match_data: &Value,
    timeline_analysis: &TimelineAnalysis,
    basic_stats: &PlayerMatchStats,
) -> TeammateAnalysis {
    let champion_id = extract_champion_id(match_data, participant_id);
    let lane_position = extract_lane_position(match_data, participant_id);

    // 分析优缺点
    let strengths = analyze_teammate_strengths(timeline_analysis, basic_stats);
    let weaknesses = analyze_teammate_weaknesses(timeline_analysis, basic_stats);

    // 分析配合能力
    let cooperation_style = identify_cooperation_style(timeline_analysis, basic_stats);
    let synergy_score = calculate_synergy_score(&strengths, &weaknesses, &cooperation_style);

    // 生成配合建议
    let cooperation_advice = generate_cooperation_advice(
        &strengths,
        &weaknesses,
        &cooperation_style,
        &lane_position
    );

    TeammateAnalysis {
        participant_id,
        champion_id,
        lane_position,
        strengths,
        weaknesses,
        cooperation_style,
        synergy_score,
        cooperation_advice,
    }
}

/// 分析队友优势
fn analyze_teammate_strengths(
    timeline: &TimelineAnalysis,
    stats: &PlayerMatchStats,
) -> Vec<TeammateStrength> {
    let mut strengths = Vec::new();

    // 1. 对线期优势
    if timeline.early_game.cs_difference > 0.0 {
        strengths.push(TeammateStrength {
            category: "对线期".to_string(),
            description: "对线期表现稳定，能够压制对手".to_string(),
            evidence: format!("平均领先{:.1}刀", timeline.early_game.cs_difference),
            reliability: 4,
        });
    }

    if timeline.early_game.xp_difference > 0.0 {
        strengths.push(TeammateStrength {
            category: "对线期".to_string(),
            description: "经验获取效率高，等级领先".to_string(),
            evidence: format!("经验领先{:.0}点", timeline.early_game.xp_difference),
            reliability: 5,
        });
    }

    // 2. 团战优势
    if stats.avg_kda > 2.5 {
        strengths.push(TeammateStrength {
            category: "团战期".to_string(),
            description: "团战表现优秀，KDA稳定".to_string(),
            evidence: format!("平均KDA {:.2}", stats.avg_kda),
            reliability: 4,
        });
    }

    if stats.avg_assists > 7.0 {
        strengths.push(TeammateStrength {
            category: "团战期".to_string(),
            description: "团战参与度高，团队意识强".to_string(),
            evidence: format!("平均助攻{:.1}次", stats.avg_assists),
            reliability: 5,
        });
    }

    // 3. 发育优势
    if timeline.mid_game.gold_per_minute > 350.0 {
        strengths.push(TeammateStrength {
            category: "发育期".to_string(),
            description: "中期经济发育能力强".to_string(),
            evidence: format!("中期金币效率{:.0}/分钟", timeline.mid_game.gold_per_minute),
            reliability: 3,
        });
    }

    // 4. 视野控制优势
    if stats.vspm > 1.2 {
        strengths.push(TeammateStrength {
            category: "视野控制".to_string(),
            description: "视野控制能力强，团队意识好".to_string(),
            evidence: format!("视野得分{:.1}/分钟", stats.vspm),
            reliability: 4,
        });
    }

    // 5. 生存能力优势
    if stats.avg_deaths < 4.0 {
        strengths.push(TeammateStrength {
            category: "生存能力".to_string(),
            description: "生存能力强，不容易被击杀".to_string(),
            evidence: format!("平均死亡{:.1}次", stats.avg_deaths),
            reliability: 4,
        });
    }

    strengths
}

/// 分析队友劣势
fn analyze_teammate_weaknesses(
    timeline: &TimelineAnalysis,
    stats: &PlayerMatchStats,
) -> Vec<TeammateWeakness> {
    let mut weaknesses = Vec::new();

    // 1. 对线期劣势
    if timeline.early_game.cs_difference < -3.0 {
        weaknesses.push(TeammateWeakness {
            category: "对线期".to_string(),
            description: "对线期补刀能力需要提升".to_string(),
            evidence: format!("平均落后{:.1}刀", -timeline.early_game.cs_difference),
            support_needed: 4,
        });
    }

    if timeline.early_game.xp_difference < -100.0 {
        weaknesses.push(TeammateWeakness {
            category: "对线期".to_string(),
            description: "经验获取效率偏低".to_string(),
            evidence: format!("经验落后{:.0}点", -timeline.early_game.xp_difference),
            support_needed: 3,
        });
    }

    // 2. 团战劣势
    if stats.avg_kda < 1.8 {
        weaknesses.push(TeammateWeakness {
            category: "团战期".to_string(),
            description: "团战表现需要提升".to_string(),
            evidence: format!("平均KDA {:.2}", stats.avg_kda),
            support_needed: 4,
        });
    }

    if stats.avg_deaths > 6.0 {
        weaknesses.push(TeammateWeakness {
            category: "团战期".to_string(),
            description: "死亡次数过多，生存能力弱".to_string(),
            evidence: format!("平均死亡{:.1}次", stats.avg_deaths),
            support_needed: 5,
        });
    }

    // 3. 发育劣势
    if timeline.mid_game.gold_per_minute < 280.0 {
        weaknesses.push(TeammateWeakness {
            category: "发育期".to_string(),
            description: "中期经济发育能力弱".to_string(),
            evidence: format!("中期金币效率{:.0}/分钟", timeline.mid_game.gold_per_minute),
            support_needed: 3,
        });
    }

    // 4. 视野控制劣势
    if stats.vspm < 0.6 {
        weaknesses.push(TeammateWeakness {
            category: "视野控制".to_string(),
            description: "视野控制能力需要提升".to_string(),
            evidence: format!("视野得分{:.1}/分钟", stats.vspm),
            support_needed: 3,
        });
    }

    weaknesses
}

/// 识别配合风格
fn identify_cooperation_style(
    timeline: &TimelineAnalysis,
    stats: &PlayerMatchStats,
) -> CooperationStyle {
    // 基于多个指标综合判断

    // 1. 主动配合型 - 高助攻，积极参与团战
    if stats.avg_assists > 8.0 && stats.avg_kda > 2.0 {
        return CooperationStyle::Proactive;
    }

    // 2. 支援型 - 视野好，助攻多，死亡少
    if stats.vspm > 1.5 && stats.avg_assists > 6.0 && stats.avg_deaths < 4.0 {
        return CooperationStyle::Supportive;
    }

    // 3. 核心型 - 高KDA，高输出
    if stats.avg_kda > 3.0 && stats.avg_kills > 6.0 {
        return CooperationStyle::Carry;
    }

    // 4. 独立型 - 补刀好，但助攻少
    if stats.cspm > 6.0 && stats.avg_assists < 5.0 {
        return CooperationStyle::Independent;
    }

    // 5. 被动配合型 - 各项指标中等
    if stats.avg_kda > 1.5 && stats.avg_kda < 2.5 {
        return CooperationStyle::Reactive;
    }

    CooperationStyle::Unknown
}

/// 计算配合度评分
fn calculate_synergy_score(
    strengths: &[TeammateStrength],
    weaknesses: &[TeammateWeakness],
    cooperation_style: &CooperationStyle,
) -> f64 {
    let mut score: f64 = 50.0; // 基础分

    // 基于优势加分
    for strength in strengths {
        score += match strength.reliability {
            5 => 10.0,
            4 => 8.0,
            3 => 5.0,
            2 => 3.0,
            1 => 1.0,
            _ => 0.0,
        };
    }

    // 基于劣势减分
    for weakness in weaknesses {
        score -= match weakness.support_needed {
            5 => 8.0,
            4 => 6.0,
            3 => 4.0,
            2 => 2.0,
            1 => 1.0,
            _ => 0.0,
        };
    }

    // 基于配合风格调整
    match cooperation_style {
        CooperationStyle::Proactive => score += 15.0,
        CooperationStyle::Supportive => score += 12.0,
        CooperationStyle::Carry => score += 10.0,
        CooperationStyle::Reactive => score += 5.0,
        CooperationStyle::Independent => score -= 5.0,
        CooperationStyle::Unknown => score -= 10.0,
    }

    // 限制在0-100范围内
    score.max(0.0_f64).min(100.0_f64)
}

/// 生成配合建议
fn generate_cooperation_advice(
    strengths: &[TeammateStrength],
    weaknesses: &[TeammateWeakness],
    cooperation_style: &CooperationStyle,
    lane_position: &str,
) -> Vec<CooperationAdvice> {
    let mut advice = Vec::new();

    // 基于优势生成配合建议
    for strength in strengths {
        match strength.category.as_str() {
            "对线期" => {
                advice.push(CooperationAdvice {
                    priority: 4,
                    category: "对线配合".to_string(),
                    title: "利用对线优势".to_string(),
                    description: format!("队友{}，可以配合其建立优势", strength.description),
                    actions: vec![
                        "及时支援其所在路线".to_string(),
                        "配合其进行换血和消耗".to_string(),
                        "在其优势时控制地图资源".to_string(),
                    ],
                });
            },
            "团战期" => {
                advice.push(CooperationAdvice {
                    priority: 5,
                    category: "团战配合".to_string(),
                    title: "团战核心配合".to_string(),
                    description: format!("队友{}，是团战的重要力量", strength.description),
                    actions: vec![
                        "保护其输出环境".to_string(),
                        "配合其开团时机".to_string(),
                        "为其创造击杀机会".to_string(),
                    ],
                });
            },
            "视野控制" => {
                advice.push(CooperationAdvice {
                    priority: 3,
                    category: "视野配合".to_string(),
                    title: "视野协作".to_string(),
                    description: format!("队友{}，可以配合其控制视野", strength.description),
                    actions: vec![
                        "配合其布置关键眼位".to_string(),
                        "利用其视野信息进行决策".to_string(),
                        "协助其排眼和反眼".to_string(),
                    ],
                });
            },
            _ => {}
        }
    }

    // 基于劣势生成支援建议
    for weakness in weaknesses {
        match weakness.category.as_str() {
            "对线期" => {
                advice.push(CooperationAdvice {
                    priority: 5,
                    category: "对线支援".to_string(),
                    title: "对线期支援".to_string(),
                    description: format!("队友{}，需要及时支援", weakness.description),
                    actions: vec![
                        "优先支援其所在路线".to_string(),
                        "帮助其控制兵线".to_string(),
                        "呼叫打野进行Gank".to_string(),
                        "选择支援型英雄".to_string(),
                    ],
                });
            },
            "团战期" => {
                advice.push(CooperationAdvice {
                    priority: 4,
                    category: "团战支援".to_string(),
                    title: "团战保护".to_string(),
                    description: format!("队友{}，需要团战保护", weakness.description),
                    actions: vec![
                        "优先保护其安全".to_string(),
                        "为其提供控制技能".to_string(),
                        "避免其被集火".to_string(),
                    ],
                });
            },
            "视野控制" => {
                advice.push(CooperationAdvice {
                    priority: 3,
                    category: "视野支援".to_string(),
                    title: "视野补充".to_string(),
                    description: format!("队友{}，需要补充视野", weakness.description),
                    actions: vec![
                        "主动承担视野责任".to_string(),
                        "购买更多控制守卫".to_string(),
                        "提醒其注意视野".to_string(),
                    ],
                });
            },
            _ => {}
        }
    }

    // 基于配合风格生成建议
    match cooperation_style {
        CooperationStyle::Proactive => {
            advice.push(CooperationAdvice {
                priority: 4,
                category: "风格配合".to_string(),
                title: "配合主动型队友".to_string(),
                description: "队友主动配合意识强".to_string(),
                actions: vec![
                    "积极响应其信号".to_string(),
                    "配合其开团时机".to_string(),
                    "及时跟上其节奏".to_string(),
                ],
            });
        },
        CooperationStyle::Supportive => {
            advice.push(CooperationAdvice {
                priority: 3,
                category: "风格配合".to_string(),
                title: "配合支援型队友".to_string(),
                description: "队友支援能力强".to_string(),
                actions: vec![
                    "信任其支援能力".to_string(),
                    "配合其视野控制".to_string(),
                    "利用其团队意识".to_string(),
                ],
            });
        },
        CooperationStyle::Carry => {
            advice.push(CooperationAdvice {
                priority: 5,
                category: "风格配合".to_string(),
                title: "配合核心型队友".to_string(),
                description: "队友是团队核心".to_string(),
                actions: vec![
                    "围绕其制定战术".to_string(),
                    "保护其发育环境".to_string(),
                    "为其创造输出空间".to_string(),
                ],
            });
        },
        CooperationStyle::Independent => {
            advice.push(CooperationAdvice {
                priority: 2,
                category: "风格配合".to_string(),
                title: "配合独立型队友".to_string(),
                description: "队友喜欢独立作战".to_string(),
                actions: vec![
                    "不要过度依赖其支援".to_string(),
                    "在其需要时提供帮助".to_string(),
                    "利用其发育能力".to_string(),
                ],
            });
        },
        _ => {}
    }

    // 基于位置生成特定建议
    match lane_position {
        "打野" => {
            advice.push(CooperationAdvice {
                priority: 4,
                category: "位置配合".to_string(),
                title: "配合打野队友".to_string(),
                description: "队友是打野位置".to_string(),
                actions: vec![
                    "及时响应其Gank信号".to_string(),
                    "配合其控制野区资源".to_string(),
                    "为其提供线上信息".to_string(),
                ],
            });
        },
        "辅助" => {
            advice.push(CooperationAdvice {
                priority: 3,
                category: "位置配合".to_string(),
                title: "配合辅助队友".to_string(),
                description: "队友是辅助位置".to_string(),
                actions: vec![
                    "配合其视野布置".to_string(),
                    "响应其开团信号".to_string(),
                    "保护其安全".to_string(),
                ],
            });
        },
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

    #[test]
    fn test_calculate_synergy_score() {
        let strengths = vec![
            TeammateStrength {
                category: "团战期".to_string(),
                description: "团战表现优秀".to_string(),
                evidence: "KDA 3.5".to_string(),
                reliability: 4,
            }
        ];

        let weaknesses = vec![
            TeammateWeakness {
                category: "对线期".to_string(),
                description: "对线能力一般".to_string(),
                evidence: "CS落后".to_string(),
                support_needed: 3,
            }
        ];

        let score = calculate_synergy_score(&strengths, &weaknesses, &CooperationStyle::Proactive);
        assert!(score > 50.0); // 应该高于基础分
    }
}
