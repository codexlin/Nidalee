/// 自我提升分析器
///
/// 职责：
/// - 基于时间线数据分析玩家自身表现
/// - 识别个人优缺点和改进空间
/// - 生成针对性的自我提升建议
use crate::domains::analysis::analyzers::core::timeline_analyzer::{
    TimelineAnalysis, PhaseAnalysis, KeyEvent
};
use crate::shared::types::{PlayerMatchStats, SummonerTrait};
use serde_json::Value;

/// 自我提升分析结果
#[derive(Debug, Clone)]
pub struct SelfImprovementAnalysis {
    // 个人表现分析
    pub performance_analysis: PerformanceAnalysis,

    // 改进建议
    pub improvement_suggestions: Vec<ImprovementSuggestion>,

    // 技能评估
    pub skill_assessment: SkillAssessment,

    // 训练计划
    pub training_plan: TrainingPlan,
}

/// 表现分析
#[derive(Debug, Clone)]
pub struct PerformanceAnalysis {
    // 各阶段表现评分
    pub early_game_score: f64,    // 对线期评分 0-100
    pub mid_game_score: f64,      // 中期评分 0-100
    pub late_game_score: f64,     // 后期评分 0-100

    // 关键指标分析
    pub cs_efficiency: f64,       // 补刀效率评分
    pub gold_efficiency: f64,     // 金币效率评分
    pub teamfight_participation: f64, // 团战参与度评分
    pub vision_control: f64,      // 视野控制评分

    // 问题识别
    pub major_issues: Vec<String>,    // 主要问题
    pub minor_issues: Vec<String>,    // 次要问题
    pub strengths: Vec<String>,       // 个人优势
}

/// 改进建议
#[derive(Debug, Clone)]
pub struct ImprovementSuggestion {
    pub priority: u8,             // 优先级 1-5
    pub category: String,         // 建议分类
    pub title: String,            // 建议标题
    pub description: String,      // 详细描述
    pub current_performance: String, // 当前表现
    pub target_performance: String,  // 目标表现
    pub specific_actions: Vec<String>, // 具体行动
    pub practice_methods: Vec<String>, // 练习方法
    pub expected_improvement: String,  // 预期改进
}

/// 技能评估
#[derive(Debug, Clone)]
pub struct SkillAssessment {
    pub laning_skill: u8,         // 对线技能 1-10
    pub farming_skill: u8,        // 发育技能 1-10
    pub teamfight_skill: u8,      // 团战技能 1-10
    pub vision_skill: u8,         // 视野技能 1-10
    pub positioning_skill: u8,    // 站位技能 1-10
    pub macro_skill: u8,          // 宏观技能 1-10
    pub overall_skill: u8,        // 综合技能 1-10
}

/// 训练计划
#[derive(Debug, Clone)]
pub struct TrainingPlan {
    pub daily_practice: Vec<String>,      // 每日练习
    pub weekly_goals: Vec<String>,        // 每周目标
    pub monthly_goals: Vec<String>,       // 每月目标
    pub specific_drills: Vec<String>,     // 专项训练
    pub review_points: Vec<String>,       // 复盘要点
}

/// 分析自我提升
pub fn analyze_self_improvement(
    participant_id: i32,
    match_data: &Value,
    timeline_analysis: &TimelineAnalysis,
    basic_stats: &PlayerMatchStats,
) -> SelfImprovementAnalysis {
    // 提取位置信息
    let lane_position = extract_lane_position(match_data, participant_id);

    // 分析个人表现
    let performance_analysis = analyze_performance(timeline_analysis, basic_stats);

    // 生成改进建议（传入位置信息）
    let improvement_suggestions = generate_improvement_suggestions(
        &performance_analysis,
        timeline_analysis,
        basic_stats,
        &lane_position
    );

    // 评估技能水平
    let skill_assessment = assess_skills(&performance_analysis, basic_stats);

    // 制定训练计划
    let training_plan = create_training_plan(&improvement_suggestions, &skill_assessment);

    SelfImprovementAnalysis {
        performance_analysis,
        improvement_suggestions,
        skill_assessment,
        training_plan,
    }
}

/// 分析个人表现
fn analyze_performance(
    timeline: &TimelineAnalysis,
    stats: &PlayerMatchStats,
) -> PerformanceAnalysis {
    // 计算各阶段评分
    let early_game_score = calculate_phase_score(&timeline.early_game, "对线期");
    let mid_game_score = calculate_phase_score(&timeline.mid_game, "中期");
    let late_game_score = calculate_phase_score(&timeline.late_game, "后期");

    // 计算关键指标评分
    let cs_efficiency = calculate_cs_efficiency_score(timeline, stats);
    let gold_efficiency = calculate_gold_efficiency_score(timeline, stats);
    let teamfight_participation = calculate_teamfight_score(stats);
    let vision_control = calculate_vision_score(stats);

    // 识别问题和优势
    let (major_issues, minor_issues, strengths) = identify_performance_issues(
        timeline, stats, early_game_score, mid_game_score, late_game_score
    );

    PerformanceAnalysis {
        early_game_score,
        mid_game_score,
        late_game_score,
        cs_efficiency,
        gold_efficiency,
        teamfight_participation,
        vision_control,
        major_issues,
        minor_issues,
        strengths,
    }
}

/// 计算阶段评分
fn calculate_phase_score(phase: &PhaseAnalysis, _phase_name: &str) -> f64 {
    let score = 50.0; // 基础分

    // 基于补刀效率评分
    let cs_score = match phase.cs_per_minute {
        x if x >= 8.0 => 20.0,
        x if x >= 7.0 => 15.0,
        x if x >= 6.0 => 10.0,
        x if x >= 5.0 => 5.0,
        _ => 0.0,
    };

    // 基于金币效率评分
    let gold_score = match phase.gold_per_minute {
        x if x >= 450.0 => 20.0,
        x if x >= 400.0 => 15.0,
        x if x >= 350.0 => 10.0,
        x if x >= 300.0 => 5.0,
        _ => 0.0,
    };

    // 基于相对优势评分
    let advantage_score = if phase.cs_difference > 0.0 {
        (phase.cs_difference / 10.0).min(10.0) // 最多10分
    } else {
        0.0
    };

    score + cs_score + gold_score + advantage_score
}

/// 计算补刀效率评分
fn calculate_cs_efficiency_score(timeline: &TimelineAnalysis, _stats: &PlayerMatchStats) -> f64 {
    let avg_cs_per_minute = (
        timeline.early_game.cs_per_minute +
        timeline.mid_game.cs_per_minute +
        timeline.late_game.cs_per_minute
    ) / 3.0;

    match avg_cs_per_minute {
        x if x >= 7.5 => 90.0,
        x if x >= 7.0 => 80.0,
        x if x >= 6.5 => 70.0,
        x if x >= 6.0 => 60.0,
        x if x >= 5.5 => 50.0,
        x if x >= 5.0 => 40.0,
        _ => 30.0,
    }
}

/// 计算金币效率评分
fn calculate_gold_efficiency_score(timeline: &TimelineAnalysis, _stats: &PlayerMatchStats) -> f64 {
    let avg_gold_per_minute = (
        timeline.early_game.gold_per_minute +
        timeline.mid_game.gold_per_minute +
        timeline.late_game.gold_per_minute
    ) / 3.0;

    match avg_gold_per_minute {
        x if x >= 450.0 => 90.0,
        x if x >= 400.0 => 80.0,
        x if x >= 350.0 => 70.0,
        x if x >= 300.0 => 60.0,
        x if x >= 250.0 => 50.0,
        _ => 40.0,
    }
}

/// 计算团战评分
fn calculate_teamfight_score(stats: &PlayerMatchStats) -> f64 {
    let kda_score: f64 = match stats.avg_kda {
        x if x >= 4.0 => 30.0,
        x if x >= 3.0 => 25.0,
        x if x >= 2.5 => 20.0,
        x if x >= 2.0 => 15.0,
        x if x >= 1.5 => 10.0,
        _ => 5.0,
    };

    let assist_score: f64 = match stats.avg_assists {
        x if x >= 10.0 => 30.0,
        x if x >= 8.0 => 25.0,
        x if x >= 6.0 => 20.0,
        x if x >= 4.0 => 15.0,
        x if x >= 2.0 => 10.0,
        _ => 5.0,
    };

    let death_penalty = if stats.avg_deaths > 6.0 {
        -20.0
    } else if stats.avg_deaths > 4.0 {
        -10.0
    } else {
        0.0
    };

    (kda_score + assist_score + death_penalty).max(0.0_f64).min(100.0_f64)
}

/// 计算视野评分
fn calculate_vision_score(stats: &PlayerMatchStats) -> f64 {
    match stats.vspm {
        x if x >= 2.0 => 90.0,
        x if x >= 1.5 => 80.0,
        x if x >= 1.2 => 70.0,
        x if x >= 1.0 => 60.0,
        x if x >= 0.8 => 50.0,
        x if x >= 0.6 => 40.0,
        _ => 30.0,
    }
}

/// 识别表现问题
fn identify_performance_issues(
    timeline: &TimelineAnalysis,
    stats: &PlayerMatchStats,
    early_score: f64,
    _mid_score: f64,
    _late_score: f64,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut major_issues = Vec::new();
    let mut minor_issues = Vec::new();
    let mut strengths = Vec::new();

    // 对线期问题
    if early_score < 40.0 {
        major_issues.push("对线期表现严重不足".to_string());
    } else if early_score < 60.0 {
        minor_issues.push("对线期表现需要提升".to_string());
    } else if early_score > 80.0 {
        strengths.push("对线期表现优秀".to_string());
    }

    // 补刀问题
    if timeline.early_game.cs_difference < -10.0 {
        major_issues.push("对线期补刀严重落后".to_string());
    } else if timeline.early_game.cs_difference < -5.0 {
        minor_issues.push("对线期补刀需要提升".to_string());
    }

    // 团战问题
    if stats.avg_kda < 1.5 {
        major_issues.push("团战表现严重不足".to_string());
    } else if stats.avg_kda < 2.0 {
        minor_issues.push("团战表现需要提升".to_string());
    } else if stats.avg_kda > 3.0 {
        strengths.push("团战表现优秀".to_string());
    }

    // 死亡问题
    if stats.avg_deaths > 7.0 {
        major_issues.push("死亡次数过多，生存能力弱".to_string());
    } else if stats.avg_deaths > 5.0 {
        minor_issues.push("死亡次数偏多，需要改善".to_string());
    }

    // 视野问题
    if stats.vspm < 0.5 {
        major_issues.push("视野控制严重不足".to_string());
    } else if stats.vspm < 0.8 {
        minor_issues.push("视野控制需要提升".to_string());
    } else if stats.vspm > 1.5 {
        strengths.push("视野控制能力强".to_string());
    }

    (major_issues, minor_issues, strengths)
}

/// 生成改进建议
fn generate_improvement_suggestions(
    performance: &PerformanceAnalysis,
    timeline: &TimelineAnalysis,
    stats: &PlayerMatchStats,
    lane_position: &str,
) -> Vec<ImprovementSuggestion> {
    let mut suggestions = Vec::new();

    // 对线期改进建议
    if performance.early_game_score < 60.0 {
        suggestions.push(ImprovementSuggestion {
            priority: 5,
            category: "对线期".to_string(),
            title: "提升对线期表现".to_string(),
            description: "对线期是游戏的基础阶段，需要重点提升".to_string(),
            current_performance: format!("对线期评分: {:.1}", performance.early_game_score),
            target_performance: "对线期评分: 70+".to_string(),
            specific_actions: vec![
                "练习补刀基本功，每天训练模式10分钟".to_string(),
                "学习对线换血技巧和时机".to_string(),
                "研究英雄克制关系和技能释放".to_string(),
                "改善对线站位，避免被消耗".to_string(),
            ],
            practice_methods: vec![
                "训练模式练习补刀".to_string(),
                "观看高手对线视频".to_string(),
                "学习兵线管理技巧".to_string(),
            ],
            expected_improvement: "对线期评分提升15-20分".to_string(),
        });
    }

    // 补刀改进建议
    if timeline.early_game.cs_difference < -5.0 {
        suggestions.push(ImprovementSuggestion {
            priority: 4,
            category: "补刀技巧".to_string(),
            title: "提升补刀能力".to_string(),
            description: "补刀是经济来源的基础，需要重点练习".to_string(),
            current_performance: format!("平均CS差: {:.1}", timeline.early_game.cs_difference),
            target_performance: "平均CS差: 0+".to_string(),
            specific_actions: vec![
                "每天训练模式练习补刀15分钟".to_string(),
                "学习不同兵种的补刀时机".to_string(),
                "练习技能补刀和普攻补刀".to_string(),
                "研究对手补刀习惯，进行干扰".to_string(),
            ],
            practice_methods: vec![
                "训练模式无装备补刀练习".to_string(),
                "自定义游戏练习对线补刀".to_string(),
                "观看补刀教学视频".to_string(),
            ],
            expected_improvement: "补刀差提升5-10刀".to_string(),
        });
    }

    // 团战改进建议
    if performance.teamfight_participation < 60.0 {
        suggestions.push(ImprovementSuggestion {
            priority: 4,
            category: "团战技巧".to_string(),
            title: "提升团战参与度".to_string(),
            description: "团战是决定胜负的关键，需要积极参与".to_string(),
            current_performance: format!("团战评分: {:.1}", performance.teamfight_participation),
            target_performance: "团战评分: 70+".to_string(),
            specific_actions: vec![
                "提高地图意识，及时支援团战".to_string(),
                "学习团战站位和技能释放时机".to_string(),
                "研究不同英雄的团战定位".to_string(),
                "练习团战中的目标选择".to_string(),
            ],
            practice_methods: vec![
                "观看团战教学视频".to_string(),
                "复盘团战录像，分析决策".to_string(),
                "练习团战英雄的技能连招".to_string(),
            ],
            expected_improvement: "团战评分提升10-15分".to_string(),
        });
    }

    // 生存能力改进建议
    if stats.avg_deaths > 5.0 {
        suggestions.push(ImprovementSuggestion {
            priority: 5,
            category: "生存能力".to_string(),
            title: "减少死亡次数".to_string(),
            description: "生存是输出的前提，需要重点改善".to_string(),
            current_performance: format!("平均死亡: {:.1}次", stats.avg_deaths),
            target_performance: "平均死亡: 4次以下".to_string(),
            specific_actions: vec![
                "学习安全站位，避免被集火".to_string(),
                "提高地图意识，避免被Gank".to_string(),
                "学会撤退时机，不要贪心".to_string(),
                "研究保命装备和技能使用".to_string(),
            ],
            practice_methods: vec![
                "观看生存技巧教学".to_string(),
                "练习走位和躲避技能".to_string(),
                "学习不同情况的撤退路线".to_string(),
            ],
            expected_improvement: "死亡次数减少1-2次".to_string(),
        });
    }

    // 视野控制改进建议
    if performance.vision_control < 60.0 {
        suggestions.push(ImprovementSuggestion {
            priority: 3,
            category: "视野控制".to_string(),
            title: "提升视野控制能力".to_string(),
            description: "视野是团队配合的基础，需要重视".to_string(),
            current_performance: format!("视野评分: {:.1}", performance.vision_control),
            target_performance: "视野评分: 70+".to_string(),
            specific_actions: vec![
                "养成买眼的习惯，每次回城都买眼".to_string(),
                "学习关键眼位的放置位置".to_string(),
                "学会使用扫描清理敌方视野".to_string(),
                "提高地图观察频率".to_string(),
            ],
            practice_methods: vec![
                "观看视野教学视频".to_string(),
                "学习职业选手的眼位布置".to_string(),
                "练习不同地图的视野控制".to_string(),
            ],
            expected_improvement: "视野评分提升10-15分".to_string(),
        });
    }

    // 按优先级排序
    suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));

    suggestions
}

/// 评估技能水平
fn assess_skills(performance: &PerformanceAnalysis, stats: &PlayerMatchStats) -> SkillAssessment {
    // 对线技能评估
    let laning_skill = match performance.early_game_score {
        x if x >= 80.0 => 9,
        x if x >= 70.0 => 8,
        x if x >= 60.0 => 7,
        x if x >= 50.0 => 6,
        x if x >= 40.0 => 5,
        x if x >= 30.0 => 4,
        x if x >= 20.0 => 3,
        x if x >= 10.0 => 2,
        _ => 1,
    };

    // 发育技能评估
    let farming_skill = match performance.cs_efficiency {
        x if x >= 80.0 => 9,
        x if x >= 70.0 => 8,
        x if x >= 60.0 => 7,
        x if x >= 50.0 => 6,
        x if x >= 40.0 => 5,
        _ => 3,
    };

    // 团战技能评估
    let teamfight_skill = match performance.teamfight_participation {
        x if x >= 80.0 => 9,
        x if x >= 70.0 => 8,
        x if x >= 60.0 => 7,
        x if x >= 50.0 => 6,
        x if x >= 40.0 => 5,
        _ => 3,
    };

    // 视野技能评估
    let vision_skill = match performance.vision_control {
        x if x >= 80.0 => 9,
        x if x >= 70.0 => 8,
        x if x >= 60.0 => 7,
        x if x >= 50.0 => 6,
        x if x >= 40.0 => 5,
        _ => 3,
    };

    // 站位技能评估（基于死亡次数）
    let positioning_skill = match stats.avg_deaths {
        x if x <= 2.0 => 9,
        x if x <= 3.0 => 8,
        x if x <= 4.0 => 7,
        x if x <= 5.0 => 6,
        x if x <= 6.0 => 5,
        x if x <= 7.0 => 4,
        x if x <= 8.0 => 3,
        x if x <= 9.0 => 2,
        _ => 1,
    };

    // 宏观技能评估（基于综合表现）
    let macro_skill = ((laning_skill + farming_skill + teamfight_skill + vision_skill) as f64 / 4.0) as u8;

    // 综合技能评估
    let overall_skill = ((laning_skill + farming_skill + teamfight_skill + vision_skill + positioning_skill + macro_skill) as f64 / 6.0) as u8;

    SkillAssessment {
        laning_skill,
        farming_skill,
        teamfight_skill,
        vision_skill,
        positioning_skill,
        macro_skill,
        overall_skill,
    }
}

/// 制定训练计划
fn create_training_plan(
    suggestions: &[ImprovementSuggestion],
    skills: &SkillAssessment,
) -> TrainingPlan {
    let mut daily_practice = Vec::new();
    let mut weekly_goals = Vec::new();
    let mut monthly_goals = Vec::new();
    let mut specific_drills = Vec::new();
    let mut review_points = Vec::new();

    // 基于改进建议制定计划
    for suggestion in suggestions {
        if suggestion.priority >= 4 {
            daily_practice.extend(suggestion.practice_methods.clone());
        }

        if suggestion.priority >= 3 {
            weekly_goals.push(suggestion.title.clone());
        }

        monthly_goals.push(format!("{} - {}", suggestion.title, suggestion.expected_improvement));
    }

    // 基于技能评估制定专项训练
    if skills.laning_skill < 6 {
        specific_drills.push("对线期专项训练".to_string());
        specific_drills.push("补刀基本功练习".to_string());
    }

    if skills.teamfight_skill < 6 {
        specific_drills.push("团战技巧训练".to_string());
        specific_drills.push("站位和走位练习".to_string());
    }

    if skills.vision_skill < 6 {
        specific_drills.push("视野控制训练".to_string());
        specific_drills.push("地图意识提升".to_string());
    }

    // 复盘要点
    review_points.push("分析每场对局的死亡原因".to_string());
    review_points.push("总结团战中的决策失误".to_string());
    review_points.push("记录对线期的失误和亮点".to_string());
    review_points.push("分析视野布置的效果".to_string());

    TrainingPlan {
        daily_practice,
        weekly_goals,
        monthly_goals,
        specific_drills,
        review_points,
    }
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
                        ("CARRY", _) | ("DUO_CARRY", _) => "ADC".to_string(),
                        ("SUPPORT", _) | ("DUO_SUPPORT", _) => "辅助".to_string(),
                        ("SOLO", "TOP") | ("DUO", "TOP") => "上单".to_string(),
                        ("SOLO", "MIDDLE") | ("SOLO", "MID") => "中单".to_string(),
                        ("NONE", "JUNGLE") | ("JUNGLE", _) => "打野".to_string(),
                        (_, "TOP") => "上单".to_string(),
                        (_, "JUNGLE") => "打野".to_string(),
                        (_, "MIDDLE") | (_, "MID") => "中单".to_string(),
                        ("SUPPORT", "BOTTOM") => "辅助".to_string(),
                        (_, "BOTTOM") => "ADC".to_string(),
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
    use crate::domains::analysis::analyzers::core::timeline_parser::{
        TimelineAnalysis, PhaseAnalysis, OpponentComparison
    };

    #[test]
    fn test_calculate_phase_score() {
        let phase = PhaseAnalysis {
            duration_minutes: 10.0,
            cs_per_minute: 7.5,
            gold_per_minute: 400.0,
            xp_per_minute: 500.0,
            cs_difference: 5.0,
            xp_difference: 100.0,
            gold_difference: 200.0,
            level_difference: 1,
        };

        let score = calculate_phase_score(&phase, "对线期");
        assert!(score > 50.0); // 应该高于基础分
    }
}
