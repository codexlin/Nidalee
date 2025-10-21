/// 英雄池建议分析器
///
/// 职责：
/// - 分析英雄池情况
/// - 识别英雄池问题（过窄、过度依赖）
use super::base::AdviceAnalyzer;
use crate::domains::tactical_advice::context::AdviceContext;
use crate::domains::tactical_advice::factory::AdviceStrategyFactory;
use crate::domains::tactical_advice::strategies::{AdviceStrategy, ProblemData, ProblemType};
use crate::domains::tactical_advice::types::GameAdvice;

pub struct ChampionAdviceAnalyzer;

impl AdviceAnalyzer for ChampionAdviceAnalyzer {
    fn analyze(&self, context: &AdviceContext) -> Vec<GameAdvice> {
        let mut advice = Vec::new();

        if context.game_count() < 10 {
            return advice;
        }

        let strategy = AdviceStrategyFactory::create(context.perspective);

        // 分析英雄池
        if let Some(advice_item) = self.analyze_champion_pool(context, strategy.as_ref()) {
            advice.push(advice_item);
        }

        advice
    }

    fn name(&self) -> &str {
        "英雄池分析器"
    }
}

impl ChampionAdviceAnalyzer {
    fn analyze_champion_pool(&self, context: &AdviceContext, strategy: &dyn AdviceStrategy) -> Option<GameAdvice> {
        let champions = &context.stats.favorite_champions;

        if champions.is_empty() {
            return None;
        }

        // 检查是否过度依赖单一英雄
        if let Some(top_champion) = champions.first() {
            let specialization = top_champion.games as f64 / context.stats.total_games as f64;

            // 超过70%使用同一英雄，且胜率不高
            if specialization >= 0.7 && top_champion.win_rate < 50.0 {
                let problem_data = ProblemData {
                    severity: specialization,
                    value: top_champion.win_rate,
                    role: context.role.clone(),
                    target_name: context.target_name.clone(),
                    extra_info: Some(format!(
                        "使用单一英雄占比{:.0}%，但胜率仅{:.0}%",
                        specialization * 100.0,
                        top_champion.win_rate
                    )),
                };

                return strategy.generate_advice(ProblemType::ChampionDependency, &problem_data);
            }
        }

        // 检查英雄池是否过窄（只有1-2个英雄）
        if champions.len() <= 2 && context.stats.total_games >= 15 {
            let problem_data = ProblemData {
                severity: 0.6,
                value: champions.len() as f64,
                role: context.role.clone(),
                target_name: context.target_name.clone(),
                extra_info: Some(format!(
                    "{}场对局只使用{}个英雄，英雄池较窄",
                    context.stats.total_games,
                    champions.len()
                )),
            };

            return strategy.generate_advice(ProblemType::ChampionPoolNarrow, &problem_data);
        }

        None
    }
}
