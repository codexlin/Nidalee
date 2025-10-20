/// 发育建议分析器
///
/// 职责：
/// - 分析发育能力（CS效率、经济曲线）
/// - 识别发育问题（中期乏力、补刀效率低）

use super::base::AdviceAnalyzer;
use crate::domains::tactical_advice::context::AdviceContext;
use crate::domains::tactical_advice::types::GameAdvice;
use crate::domains::tactical_advice::strategies::{AdviceStrategy, ProblemData, ProblemType};
use crate::domains::tactical_advice::factory::AdviceStrategyFactory;
use crate::domains::analysis::thresholds;

pub struct FarmingAdviceAnalyzer;

impl AdviceAnalyzer for FarmingAdviceAnalyzer {
    fn analyze(&self, context: &AdviceContext) -> Vec<GameAdvice> {
        let mut advice = Vec::new();

        if context.games.len() < 5 {
            return advice;
        }

        let strategy = AdviceStrategyFactory::create(context.perspective);

        // 1. 分析中期经济曲线
        if let Some(advice_item) = self.analyze_midgame_economy(context, strategy.as_ref()) {
            advice.push(advice_item);
        }

        // 2. 分析整体补刀效率
        if let Some(advice_item) = self.analyze_cs_efficiency(context, strategy.as_ref()) {
            advice.push(advice_item);
        }

        advice
    }

    fn name(&self) -> &str {
        "发育分析器"
    }
}

impl FarmingAdviceAnalyzer {
    /// 分析中期经济（发育曲线）
    fn analyze_midgame_economy(
        &self,
        context: &AdviceContext,
        strategy: &dyn AdviceStrategy,
    ) -> Option<GameAdvice> {
        let mut early_gold_sum = 0.0;
        let mut mid_gold_sum = 0.0;
        let mut valid_games = 0;

        for game in &context.games {
            if let Some(timeline) = &game.player_data.timeline_data {
                if let (Some(early), Some(mid)) = (
                    timeline.gold_per_min_0_10,
                    timeline.gold_per_min_10_20,
                ) {
                    early_gold_sum += early;
                    mid_gold_sum += mid;
                    valid_games += 1;
                }
            }
        }

        if valid_games < 5 {
            return None;
        }

        let avg_early_gold = early_gold_sum / valid_games as f64;
        let avg_mid_gold = mid_gold_sum / valid_games as f64;

        // 只在中期明显下降时生成建议
        if avg_mid_gold >= avg_early_gold * thresholds::growth::MID_GAME_DECLINE {
            return None;  // 中期发育正常
        }

        let decline_rate = 1.0 - (avg_mid_gold / avg_early_gold);

        let problem_data = ProblemData {
            severity: decline_rate,
            value: avg_mid_gold,
            role: context.role.clone(),
            target_name: context.target_name.clone(),
            extra_info: Some(format!(
                "对线期{:.0}金币/分 → 中期{:.0}金币/分，下降{:.0}%",
                avg_early_gold, avg_mid_gold, decline_rate * 100.0
            )),
        };

        strategy.generate_advice(ProblemType::MidGameDecline, &problem_data)
    }

    /// 分析整体补刀效率
    fn analyze_cs_efficiency(
        &self,
        context: &AdviceContext,
        strategy: &dyn AdviceStrategy,
    ) -> Option<GameAdvice> {
        let cspm = context.stats.cspm;

        // 只在CSPM明显偏低时生成建议
        if cspm >= thresholds::cs::GOOD {
            return None;  // 补刀效率正常
        }

        let problem_data = ProblemData {
            severity: (thresholds::cs::GOOD - cspm) / thresholds::cs::GOOD,
            value: cspm,
            role: context.role.clone(),
            target_name: context.target_name.clone(),
            extra_info: Some(format!(
                "平均补刀{:.1}/分钟，低于标准{:.1}",
                cspm, thresholds::cs::GOOD
            )),
        };

        strategy.generate_advice(ProblemType::PoorFarming, &problem_data)
    }
}

