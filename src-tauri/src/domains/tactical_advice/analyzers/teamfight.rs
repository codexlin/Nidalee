/// 团战建议分析器
///
/// 职责：
/// - 分析团战表现（参团率、站位、存活）
/// - 识别团战问题
use super::base::AdviceAnalyzer;
use crate::domains::analysis::thresholds;
use crate::domains::tactical_advice::context::AdviceContext;
use crate::domains::tactical_advice::factory::AdviceStrategyFactory;
use crate::domains::tactical_advice::strategies::{AdviceStrategy, ProblemData, ProblemType};
use crate::domains::tactical_advice::types::GameAdvice;

pub struct TeamfightAdviceAnalyzer;

impl AdviceAnalyzer for TeamfightAdviceAnalyzer {
    fn analyze(&self, context: &AdviceContext) -> Vec<GameAdvice> {
        let mut advice = Vec::new();

        if context.game_count() < 10 {
            return advice;
        }

        let strategy = AdviceStrategyFactory::create(context.perspective);

        // 1. 分析KDA和参团情况
        if let Some(advice_item) = self.analyze_teamfight_participation(context, strategy.as_ref()) {
            advice.push(advice_item);
        }

        // 2. 分析死亡次数（存活能力）
        if let Some(advice_item) = self.analyze_survival(context, strategy.as_ref()) {
            advice.push(advice_item);
        }

        advice
    }

    fn name(&self) -> &str {
        "团战分析器"
    }
}

impl TeamfightAdviceAnalyzer {
    /// 分析团战参与度（基于助攻和KDA）
    fn analyze_teamfight_participation(
        &self,
        context: &AdviceContext,
        strategy: &dyn AdviceStrategy,
    ) -> Option<GameAdvice> {
        let stats = &context.stats;

        // 计算参团率（K+A）相对于队伍击杀的比例
        // 这里用一个简化版本：助攻数相对于击杀数的比例
        let participation_ratio = if stats.avg_kills > 0.0 {
            stats.avg_assists / (stats.avg_kills + stats.avg_assists)
        } else {
            stats.avg_assists / (stats.avg_assists + 1.0)
        };

        // 低参团率问题（助攻占比过低）
        if participation_ratio < 0.4 && stats.avg_assists < 5.0 {
            let problem_data = ProblemData {
                severity: (1.0 - participation_ratio).clamp(0.0, 1.0),
                value: stats.avg_assists,
                role: context.role.clone(),
                target_name: context.target_name.clone(),
                extra_info: Some(format!(
                    "平均助攻{:.1}，参团率{:.0}%，样本{}场",
                    stats.avg_assists,
                    participation_ratio * 100.0,
                    context.game_count()
                )),
            };

            return strategy.generate_advice(ProblemType::LowTeamfightParticipation, &problem_data);
        }

        None
    }

    /// 分析存活能力（死亡次数）
    fn analyze_survival(&self, context: &AdviceContext, strategy: &dyn AdviceStrategy) -> Option<GameAdvice> {
        let stats = &context.stats;

        // 死亡过多问题
        if stats.avg_deaths > thresholds::kda::DEATH_TOO_MANY {
            let problem_data = ProblemData {
                severity: ((stats.avg_deaths - thresholds::kda::DEATH_TOO_MANY) / 5.0).clamp(0.0, 1.0),
                value: stats.avg_deaths,
                role: context.role.clone(),
                target_name: context.target_name.clone(),
                extra_info: Some(format!(
                    "平均死亡{:.1}次/场，KDA {:.2}，样本{}场",
                    stats.avg_deaths,
                    stats.avg_kda,
                    context.game_count()
                )),
            };

            return strategy.generate_advice(ProblemType::HighDeathRate, &problem_data);
        }

        None
    }
}
