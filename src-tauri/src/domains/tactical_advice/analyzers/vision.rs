/// 视野建议分析器
///
/// 职责：
/// - 分析视野控制（视野得分）
/// - 识别视野问题

use super::base::AdviceAnalyzer;
use crate::domains::tactical_advice::context::AdviceContext;
use crate::domains::tactical_advice::types::GameAdvice;
use crate::domains::tactical_advice::strategies::{AdviceStrategy, ProblemData, ProblemType};
use crate::domains::tactical_advice::factory::AdviceStrategyFactory;
use crate::domains::analysis::thresholds;

pub struct VisionAdviceAnalyzer;

impl AdviceAnalyzer for VisionAdviceAnalyzer {
    fn analyze(&self, context: &AdviceContext) -> Vec<GameAdvice> {
        let mut advice = Vec::new();

        if context.game_count() < 5 {
            return advice;
        }

        let strategy = AdviceStrategyFactory::create(context.perspective);

        // 分析视野得分
        if let Some(advice_item) = self.analyze_vision_score(context, strategy.as_ref()) {
            advice.push(advice_item);
        }

        advice
    }

    fn name(&self) -> &str {
        "视野分析器"
    }
}

impl VisionAdviceAnalyzer {
    fn analyze_vision_score(
        &self,
        context: &AdviceContext,
        strategy: &dyn AdviceStrategy,
    ) -> Option<GameAdvice> {
        let vspm = context.stats.vspm;

        // 获取位置特定的视野阈值
        let (_high, low) = thresholds::vision::for_role(&context.role);

        // 只在视野得分明显低于标准时生成建议
        if vspm >= low {
            return None;  // 视野得分正常
        }

        let problem_data = ProblemData {
            severity: (low - vspm) / low,
            value: vspm,
            role: context.role.clone(),
            target_name: context.target_name.clone(),
            extra_info: Some(format!(
                "平均视野得分{:.1}/分钟，{}位置标准为{:.1}",
                vspm, context.role, low
            )),
        };

        strategy.generate_advice(ProblemType::LowVisionScore, &problem_data)
    }
}

