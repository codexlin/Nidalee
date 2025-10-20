/// 对线期建议分析器
///
/// 职责：
/// - 分析对线期表现（基于时间线数据）
/// - 识别对线期问题（补刀落后、被压制等）
/// - 根据视角生成对应建议

use super::base::AdviceAnalyzer;
use crate::domains::tactical_advice::context::AdviceContext;
use crate::domains::tactical_advice::types::GameAdvice;
use crate::domains::tactical_advice::strategies::{AdviceStrategy, ProblemData, ProblemType};
use crate::domains::tactical_advice::factory::AdviceStrategyFactory;
use crate::domains::analysis::thresholds;

pub struct LaningAdviceAnalyzer;

impl AdviceAnalyzer for LaningAdviceAnalyzer {
    fn analyze(&self, context: &AdviceContext) -> Vec<GameAdvice> {
        let mut advice = Vec::new();

        // 数据量不足，不分析
        if context.games.len() < 5 {
            return advice;
        }

        // 创建策略
        let strategy = AdviceStrategyFactory::create(context.perspective);

        // 1. 分析对线期CS差
        if let Some(advice_item) = self.analyze_cs_difference(context, strategy.as_ref()) {
            advice.push(advice_item);
        }

        // 2. 分析经验差（等级压制）
        if let Some(advice_item) = self.analyze_xp_difference(context, strategy.as_ref()) {
            advice.push(advice_item);
        }

        advice
    }

    fn name(&self) -> &str {
        "对线期分析器"
    }
}

impl LaningAdviceAnalyzer {
    /// 分析CS差（补刀差）
    fn analyze_cs_difference(
        &self,
        context: &AdviceContext,
        strategy: &dyn AdviceStrategy,
    ) -> Option<GameAdvice> {
        let mut total_cs_diff = 0.0;
        let mut valid_games = 0;

        // 统计对线期CS差
        for game in &context.games {
            if let Some(timeline) = &game.player_data.timeline_data {
                if let Some(cs_diff) = timeline.cs_diff_0_10 {
                    total_cs_diff += cs_diff;
                    valid_games += 1;
                }
            }
        }

        if valid_games < 5 {
            return None;
        }

        let avg_cs_diff = total_cs_diff / valid_games as f64;

        // 判断是否有问题（只在落后时生成建议）
        if avg_cs_diff >= thresholds::laning_phase::CS_DIFF_DISADVANTAGE {
            return None;  // CS差不大或领先，无需建议
        }

        // 识别问题类型
        let problem_type = if avg_cs_diff <= thresholds::laning_phase::CS_DIFF_SUPPRESSED {
            ProblemType::LaningDominated  // 被严重压制
        } else {
            ProblemType::LaningCsDeficit  // 一般落后
        };

        // 构建问题数据
        let problem_data = ProblemData {
            severity: (-avg_cs_diff / 30.0).clamp(0.0, 1.0),  // 30刀=100%严重度
            value: avg_cs_diff,
            role: context.role.clone(),
            target_name: context.target_name.clone(),
            extra_info: Some(format!(
                "前10分钟平均补刀差{:+.1}，有效样本{}场",
                avg_cs_diff, valid_games
            )),
        };

        // 使用策略生成建议
        strategy.generate_advice(problem_type, &problem_data)
    }

    /// 分析经验差（等级优势/劣势）
    fn analyze_xp_difference(
        &self,
        context: &AdviceContext,
        strategy: &dyn AdviceStrategy,
    ) -> Option<GameAdvice> {
        let mut total_xp_diff = 0.0;
        let mut valid_games = 0;

        // 统计对线期经验差
        for game in &context.games {
            if let Some(timeline) = &game.player_data.timeline_data {
                if let Some(xp_diff) = timeline.xp_diff_0_10 {
                    total_xp_diff += xp_diff;
                    valid_games += 1;
                }
            }
        }

        if valid_games < 5 {
            return None;
        }

        let avg_xp_diff = total_xp_diff / valid_games as f64;

        // 只在经验落后时生成建议
        if avg_xp_diff >= thresholds::laning_phase::XP_DIFF_DISADVANTAGE {
            return None;  // 经验不落后或领先，无需建议
        }

        // 经验严重落后 - 这是对线被压制的表现
        let problem_data = ProblemData {
            severity: (-avg_xp_diff / 1000.0).clamp(0.0, 1.0),
            value: avg_xp_diff,
            role: context.role.clone(),
            target_name: context.target_name.clone(),
            extra_info: Some(format!(
                "前10分钟平均经验差{:+.0}，经常被压等级",
                avg_xp_diff
            )),
        };

        // 使用"被压制"问题类型
        strategy.generate_advice(ProblemType::LaningDominated, &problem_data)
    }
}

