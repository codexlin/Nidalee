use super::strategies::{AdviceStrategy, CollaborationStrategy, SelfImprovementStrategy, TargetingStrategy};
/// 建议策略工厂（Factory 模式）
///
/// 职责：
/// - 根据视角创建对应的策略
/// - 封装策略创建逻辑
use super::types::AdvicePerspective;

/// 建议策略工厂
pub struct AdviceStrategyFactory;

impl AdviceStrategyFactory {
    /// 根据视角创建策略
    pub fn create(perspective: AdvicePerspective) -> Box<dyn AdviceStrategy> {
        match perspective {
            AdvicePerspective::SelfImprovement => Box::new(SelfImprovementStrategy),
            AdvicePerspective::Targeting => Box::new(TargetingStrategy),
            AdvicePerspective::Collaboration => Box::new(CollaborationStrategy),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creates_correct_strategy() {
        let strategy = AdviceStrategyFactory::create(AdvicePerspective::SelfImprovement);
        assert_eq!(strategy.name(), "改进建议策略");
        assert_eq!(strategy.perspective(), AdvicePerspective::SelfImprovement);

        let strategy = AdviceStrategyFactory::create(AdvicePerspective::Targeting);
        assert_eq!(strategy.name(), "针对战术策略");
        assert_eq!(strategy.perspective(), AdvicePerspective::Targeting);

        let strategy = AdviceStrategyFactory::create(AdvicePerspective::Collaboration);
        assert_eq!(strategy.name(), "团队协作策略");
        assert_eq!(strategy.perspective(), AdvicePerspective::Collaboration);
    }
}
