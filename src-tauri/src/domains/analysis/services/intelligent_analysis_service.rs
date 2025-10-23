/// 智能分析服务
///
/// 职责：
/// - 整合所有分析器（时间线、对手、队友、自我提升）
/// - 提供统一的分析接口
/// - 生成综合性的战术建议
use crate::domains::analysis::analyzers::core::timeline_parser::{
    parse_timeline_data, TimelineAnalysis
};
use crate::domains::analysis::analyzers::opponent_analyzer::{
    analyze_opponent, OpponentAnalysis
};
use crate::domains::analysis::analyzers::teammate_analyzer::{
    analyze_teammate, TeammateAnalysis
};
use crate::domains::analysis::analyzers::self_improvement_analyzer::{
    analyze_self_improvement, SelfImprovementAnalysis
};
use crate::shared::types::{PlayerMatchStats, GameAdvice};
use serde_json::Value;

/// 智能分析结果
#[derive(Debug, Clone)]
pub struct IntelligentAnalysisResult {
    // 基础信息
    pub target_puuid: String,
    pub target_participant_id: i32,
    pub match_id: String,

    // 时间线分析
    pub timeline_analysis: Option<TimelineAnalysis>,

    // 对手分析
    pub opponent_analyses: Vec<OpponentAnalysis>,

    // 队友分析
    pub teammate_analyses: Vec<TeammateAnalysis>,

    // 自我提升分析
    pub self_improvement: Option<SelfImprovementAnalysis>,

    // 综合建议
    pub comprehensive_advice: Vec<ComprehensiveAdvice>,

    // 战术总结
    pub tactical_summary: TacticalSummary,
}

/// 综合建议
#[derive(Debug, Clone)]
pub struct ComprehensiveAdvice {
    pub priority: u8,             // 优先级 1-5
    pub category: String,         // 建议分类
    pub title: String,            // 建议标题
    pub description: String,      // 详细描述
    pub target: AdviceTarget,     // 建议目标
    pub specific_actions: Vec<String>, // 具体行动
    pub expected_outcome: String, // 预期结果
}

/// 建议目标
#[derive(Debug, Clone)]
pub enum AdviceTarget {
    MySelf,         // 针对自己 (Self是关键字，用MySelf替代)
    Teammate(i32),  // 针对特定队友
    Opponent(i32),  // 针对特定对手
    Team,           // 针对整个团队
}

/// 战术总结
#[derive(Debug, Clone)]
pub struct TacticalSummary {
    pub game_phase_analysis: String,      // 游戏阶段分析
    pub key_opportunities: Vec<String>,   // 关键机会
    pub major_threats: Vec<String>,       // 主要威胁
    pub team_synergy: f64,                // 团队配合度
    pub win_conditions: Vec<String>,      // 胜利条件
    pub recommended_strategy: String,     // 推荐策略
}

/// 执行智能分析
pub fn perform_intelligent_analysis(
    match_data: &Value,
    target_puuid: &str,
    target_participant_id: i32,
) -> Result<IntelligentAnalysisResult, String> {
    let match_id = extract_match_id(match_data)?;

    // 1. 解析时间线数据
    let timeline_analysis = if let Some(timeline_json) = match_data.get("match_timeline_json") {
        parse_timeline_data(timeline_json)
    } else {
        None
    };

    // 2. 获取基础统计数据
    let basic_stats = extract_basic_stats(match_data, target_participant_id)?;

    // 3. 分析所有对手
    let opponent_analyses = analyze_all_opponents(match_data, target_participant_id, &timeline_analysis, &basic_stats);

    // 4. 分析所有队友
    let teammate_analyses = analyze_all_teammates(match_data, target_participant_id, &timeline_analysis, &basic_stats);

    // 5. 自我提升分析
    let self_improvement = if let Some(ref timeline) = timeline_analysis {
        Some(analyze_self_improvement(target_participant_id, match_data, timeline, &basic_stats))
    } else {
        None
    };

    // 6. 生成综合建议
    let comprehensive_advice = generate_comprehensive_advice(
        &opponent_analyses,
        &teammate_analyses,
        &self_improvement,
        &timeline_analysis
    );

    // 7. 生成战术总结
    let tactical_summary = generate_tactical_summary(
        &opponent_analyses,
        &teammate_analyses,
        &self_improvement,
        &timeline_analysis
    );

    Ok(IntelligentAnalysisResult {
        target_puuid: target_puuid.to_string(),
        target_participant_id,
        match_id,
        timeline_analysis,
        opponent_analyses,
        teammate_analyses,
        self_improvement,
        comprehensive_advice,
        tactical_summary,
    })
}

/// 分析所有对手
fn analyze_all_opponents(
    match_data: &Value,
    target_participant_id: i32,
    timeline_analysis: &Option<TimelineAnalysis>,
    basic_stats: &PlayerMatchStats,
) -> Vec<OpponentAnalysis> {
    let mut analyses = Vec::new();

    if let Some(participants) = match_data.get("participants").and_then(|p| p.as_array()) {
        for participant in participants {
            if let Some(participant_id) = participant.get("participantId").and_then(|id| id.as_i64()) {
                let participant_id = participant_id as i32;

                // 只分析敌方玩家
                if is_opponent(target_participant_id, participant_id) {
                    if let Some(ref timeline) = timeline_analysis {
                        let analysis = analyze_opponent(participant_id, match_data, timeline, basic_stats);
                        analyses.push(analysis);
                    }
                }
            }
        }
    }

    analyses
}

/// 分析所有队友
fn analyze_all_teammates(
    match_data: &Value,
    target_participant_id: i32,
    timeline_analysis: &Option<TimelineAnalysis>,
    basic_stats: &PlayerMatchStats,
) -> Vec<TeammateAnalysis> {
    let mut analyses = Vec::new();

    if let Some(participants) = match_data.get("participants").and_then(|p| p.as_array()) {
        for participant in participants {
            if let Some(participant_id) = participant.get("participantId").and_then(|id| id.as_i64()) {
                let participant_id = participant_id as i32;

                // 只分析友方玩家（排除自己）
                if is_teammate(target_participant_id, participant_id) {
                    if let Some(ref timeline) = timeline_analysis {
                        let analysis = analyze_teammate(participant_id, match_data, timeline, basic_stats);
                        analyses.push(analysis);
                    }
                }
            }
        }
    }

    analyses
}

/// 生成综合建议
fn generate_comprehensive_advice(
    opponent_analyses: &[OpponentAnalysis],
    teammate_analyses: &[TeammateAnalysis],
    self_improvement: &Option<SelfImprovementAnalysis>,
    timeline_analysis: &Option<TimelineAnalysis>,
) -> Vec<ComprehensiveAdvice> {
    let mut advice = Vec::new();

    // 1. 基于对手分析生成建议
    for opponent in opponent_analyses {
        for tactical_advice in &opponent.tactical_advice {
            advice.push(ComprehensiveAdvice {
                priority: tactical_advice.priority,
                category: tactical_advice.category.clone(),
                title: format!("针对{}: {}", opponent.lane_position, tactical_advice.title),
                description: tactical_advice.description.clone(),
                target: AdviceTarget::Opponent(opponent.participant_id),
                specific_actions: tactical_advice.implementation.clone(),
                expected_outcome: format!("限制{}的发挥", opponent.lane_position),
            });
        }
    }

    // 2. 基于队友分析生成建议
    for teammate in teammate_analyses {
        for cooperation_advice in &teammate.cooperation_advice {
            advice.push(ComprehensiveAdvice {
                priority: cooperation_advice.priority,
                category: cooperation_advice.category.clone(),
                title: format!("配合{}: {}", teammate.lane_position, cooperation_advice.title),
                description: cooperation_advice.description.clone(),
                target: AdviceTarget::Teammate(teammate.participant_id),
                specific_actions: cooperation_advice.actions.clone(),
                expected_outcome: format!("提升与{}的配合效果", teammate.lane_position),
            });
        }
    }

    // 3. 基于自我提升分析生成建议
    if let Some(ref self_improvement) = self_improvement {
        for suggestion in &self_improvement.improvement_suggestions {
            advice.push(ComprehensiveAdvice {
                priority: suggestion.priority,
                category: suggestion.category.clone(),
                title: suggestion.title.clone(),
                description: suggestion.description.clone(),
                target: AdviceTarget::MySelf,
                specific_actions: suggestion.specific_actions.clone(),
                expected_outcome: suggestion.expected_improvement.clone(),
            });
        }
    }

    // 4. 基于时间线分析生成战术建议
    if let Some(ref timeline) = timeline_analysis {
        advice.extend(generate_timeline_based_advice(timeline));
    }

    // 按优先级排序
    advice.sort_by(|a, b| b.priority.cmp(&a.priority));

    advice
}

/// 生成基于时间线的建议
fn generate_timeline_based_advice(timeline: &TimelineAnalysis) -> Vec<ComprehensiveAdvice> {
    let mut advice = Vec::new();

    // 对线期建议
    if timeline.early_game.cs_difference < -5.0 {
        advice.push(ComprehensiveAdvice {
            priority: 5,
            category: "对线期".to_string(),
            title: "对线期补刀落后".to_string(),
            description: "对线期补刀严重落后，需要重点改善".to_string(),
            target: AdviceTarget::MySelf,
            specific_actions: vec![
                "加强补刀练习".to_string(),
                "改善对线站位".to_string(),
                "学习兵线管理".to_string(),
            ],
            expected_outcome: "对线期补刀差提升".to_string(),
        });
    }

    // 中期建议
    if timeline.mid_game.gold_per_minute < 300.0 {
        advice.push(ComprehensiveAdvice {
            priority: 4,
            category: "发育期".to_string(),
            title: "中期发育不足".to_string(),
            description: "中期经济发育能力需要提升".to_string(),
            target: AdviceTarget::MySelf,
            specific_actions: vec![
                "提高补刀效率".to_string(),
                "寻找更多经济来源".to_string(),
                "控制地图资源".to_string(),
            ],
            expected_outcome: "中期经济效率提升".to_string(),
        });
    }

    advice
}

/// 生成战术总结
fn generate_tactical_summary(
    opponent_analyses: &[OpponentAnalysis],
    teammate_analyses: &[TeammateAnalysis],
    self_improvement: &Option<SelfImprovementAnalysis>,
    timeline_analysis: &Option<TimelineAnalysis>,
) -> TacticalSummary {
    // 分析游戏阶段
    let game_phase_analysis = analyze_game_phases(timeline_analysis);

    // 识别关键机会
    let key_opportunities = identify_key_opportunities(opponent_analyses, teammate_analyses);

    // 识别主要威胁
    let major_threats = identify_major_threats(opponent_analyses);

    // 计算团队配合度
    let team_synergy = calculate_team_synergy(teammate_analyses);

    // 确定胜利条件
    let win_conditions = determine_win_conditions(opponent_analyses, teammate_analyses);

    // 推荐策略
    let recommended_strategy = recommend_strategy(opponent_analyses, teammate_analyses, self_improvement);

    TacticalSummary {
        game_phase_analysis,
        key_opportunities,
        major_threats,
        team_synergy,
        win_conditions,
        recommended_strategy,
    }
}

/// 分析游戏阶段
fn analyze_game_phases(timeline_analysis: &Option<TimelineAnalysis>) -> String {
    if let Some(ref timeline) = timeline_analysis {
        let mut analysis = String::new();

        // 对线期分析
        if timeline.early_game.cs_difference > 0.0 {
            analysis.push_str("对线期表现良好，建立了优势；");
        } else if timeline.early_game.cs_difference < -5.0 {
            analysis.push_str("对线期表现不佳，处于劣势；");
        } else {
            analysis.push_str("对线期表现平稳；");
        }

        // 中期分析
        if timeline.mid_game.gold_per_minute > 400.0 {
            analysis.push_str("中期发育能力强；");
        } else if timeline.mid_game.gold_per_minute < 300.0 {
            analysis.push_str("中期发育能力需要提升；");
        }

        // 后期分析
        if timeline.late_game.gold_per_minute > timeline.mid_game.gold_per_minute {
            analysis.push_str("后期仍有成长空间。");
        } else {
            analysis.push_str("后期表现稳定。");
        }

        analysis
    } else {
        "无法获取时间线数据，建议关注基础表现指标。".to_string()
    }
}

/// 识别关键机会
fn identify_key_opportunities(
    opponent_analyses: &[OpponentAnalysis],
    teammate_analyses: &[TeammateAnalysis],
) -> Vec<String> {
    let mut opportunities = Vec::new();

    // 基于对手劣势识别机会
    for opponent in opponent_analyses {
        for weakness in &opponent.weaknesses {
            if weakness.exploit_level >= 4 {
                opportunities.push(format!("{}的{}可以利用", opponent.lane_position, weakness.description));
            }
        }
    }

    // 基于队友优势识别机会
    for teammate in teammate_analyses {
        for strength in &teammate.strengths {
            if strength.reliability >= 4 {
                opportunities.push(format!("配合{}的{}建立优势", teammate.lane_position, strength.description));
            }
        }
    }

    opportunities
}

/// 识别主要威胁
fn identify_major_threats(opponent_analyses: &[OpponentAnalysis]) -> Vec<String> {
    let mut threats = Vec::new();

    for opponent in opponent_analyses {
        for strength in &opponent.strengths {
            if strength.threat_level >= 4 {
                threats.push(format!("{}的{}需要重点防范", opponent.lane_position, strength.description));
            }
        }
    }

    threats
}

/// 计算团队配合度
fn calculate_team_synergy(teammate_analyses: &[TeammateAnalysis]) -> f64 {
    if teammate_analyses.is_empty() {
        return 50.0;
    }

    let total_synergy: f64 = teammate_analyses.iter()
        .map(|teammate| teammate.synergy_score)
        .sum();

    total_synergy / teammate_analyses.len() as f64
}

/// 确定胜利条件
fn determine_win_conditions(
    opponent_analyses: &[OpponentAnalysis],
    teammate_analyses: &[TeammateAnalysis],
) -> Vec<String> {
    let mut conditions = Vec::new();

    // 基于对手分析
    for opponent in opponent_analyses {
        for weakness in &opponent.weaknesses {
            if weakness.exploit_level >= 4 {
                conditions.push(format!("重点针对{}的{}", opponent.lane_position, weakness.description));
            }
        }
    }

    // 基于队友分析
    for teammate in teammate_analyses {
        for strength in &teammate.strengths {
            if strength.reliability >= 4 {
                conditions.push(format!("发挥{}的{}优势", teammate.lane_position, strength.description));
            }
        }
    }

    conditions
}

/// 推荐策略
fn recommend_strategy(
    opponent_analyses: &[OpponentAnalysis],
    teammate_analyses: &[TeammateAnalysis],
    self_improvement: &Option<SelfImprovementAnalysis>,
) -> String {
    let mut strategy = String::new();

    // 基于对手分析推荐策略
    let strong_opponents: Vec<_> = opponent_analyses.iter()
        .filter(|opp| opp.strengths.iter().any(|s| s.threat_level >= 4))
        .collect();

    if !strong_opponents.is_empty() {
        strategy.push_str("面对强势对手，建议采用稳健策略，避免正面冲突；");
    }

    // 基于队友分析推荐策略
    let high_synergy_teammates: Vec<_> = teammate_analyses.iter()
        .filter(|tm| tm.synergy_score >= 70.0)
        .collect();

    if !high_synergy_teammates.is_empty() {
        strategy.push_str("队友配合度高，可以采取积极策略；");
    }

    // 基于自我提升分析推荐策略
    if let Some(ref self_improvement) = self_improvement {
        if self_improvement.performance_analysis.early_game_score < 60.0 {
            strategy.push_str("对线期需要加强，建议选择稳健英雄；");
        }
    }

    if strategy.is_empty() {
        strategy = "建议根据具体对局情况灵活调整策略。".to_string();
    }

    strategy
}

/// 判断是否为对手
fn is_opponent(target_participant_id: i32, participant_id: i32) -> bool {
    // 简化判断：假设1-5为蓝队，6-10为红队
    let target_team = if target_participant_id <= 5 { 1 } else { 2 };
    let participant_team = if participant_id <= 5 { 1 } else { 2 };

    target_team != participant_team
}

/// 判断是否为队友
fn is_teammate(target_participant_id: i32, participant_id: i32) -> bool {
    // 简化判断：假设1-5为蓝队，6-10为红队
    let target_team = if target_participant_id <= 5 { 1 } else { 2 };
    let participant_team = if participant_id <= 5 { 1 } else { 2 };

    target_team == participant_team && participant_id != target_participant_id
}

/// 提取比赛ID
fn extract_match_id(match_data: &Value) -> Result<String, String> {
    match_data.get("gameId")
        .and_then(|id| id.as_i64())
        .map(|id| id.to_string())
        .ok_or_else(|| "无法提取比赛ID".to_string())
}

/// 提取基础统计数据
fn extract_basic_stats(match_data: &Value, participant_id: i32) -> Result<PlayerMatchStats, String> {
    // 这里需要实现从match_data中提取基础统计数据的逻辑
    // 暂时返回一个默认值
    Ok(PlayerMatchStats {
        avg_kills: 5.0,
        avg_deaths: 3.0,
        avg_assists: 7.0,
        avg_kda: 4.0,
        cspm: 6.5,
        vspm: 1.2,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_is_opponent() {
        assert!(is_opponent(1, 6)); // 1是蓝队，6是红队
        assert!(!is_opponent(1, 2)); // 1和2都是蓝队
        assert!(!is_opponent(1, 1)); // 自己不是对手
    }

    #[test]
    fn test_is_teammate() {
        assert!(is_teammate(1, 2)); // 1和2都是蓝队
        assert!(!is_teammate(1, 6)); // 1是蓝队，6是红队
        assert!(!is_teammate(1, 1)); // 自己不是队友
    }
}
