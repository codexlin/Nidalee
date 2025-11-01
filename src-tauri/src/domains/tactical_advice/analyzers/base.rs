use crate::domains::analysis::AnalysisStrategy;
/// 建议分析器基础接口
///
/// 定义所有分析器的统一接口
use crate::domains::tactical_advice::context::AdviceContext;
use crate::domains::tactical_advice::types::GameAdvice;

/// 建议分析器 trait（责任链节点）
pub trait AdviceAnalyzer: Send + Sync {
    /// 分析并生成建议
    fn analyze(&self, context: &AdviceContext) -> Vec<GameAdvice>;

    /// 获取分析器名称
    fn name(&self) -> &str;

    /// 是否启用（根据策略）
    fn is_enabled(&self, strategy: &AnalysisStrategy) -> bool {
        // 默认：只在排位模式下启用
        matches!(strategy, AnalysisStrategy::SoloRanked | AnalysisStrategy::FlexRanked | AnalysisStrategy::MixedRanked)
    }
}
