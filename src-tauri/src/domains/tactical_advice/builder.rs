/// 建议建造者（Builder 模式）
///
/// 职责：
/// - 提供优雅的链式调用API
/// - 构建 GameAdvice 对象
/// - 确保必填字段完整
use super::types::{AdviceCategory, AdvicePerspective, GameAdvice};

/// 建议建造者
#[derive(Debug)]
pub struct AdviceBuilder {
    title: Option<String>,
    problem: Option<String>,
    evidence: Option<String>,
    suggestions: Vec<String>,
    priority: u8,
    category: AdviceCategory,
    perspective: AdvicePerspective,
    affected_role: Option<String>,
    target_player: Option<String>,
}

impl AdviceBuilder {
    /// 创建新的建造者
    pub fn new() -> Self {
        Self {
            title: None,
            problem: None,
            evidence: None,
            suggestions: Vec::new(),
            priority: 1,
            category: AdviceCategory::Laning,
            perspective: AdvicePerspective::SelfImprovement,
            affected_role: None,
            target_player: None,
        }
    }

    /// 设置标题
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 设置问题描述
    pub fn problem(mut self, problem: impl Into<String>) -> Self {
        self.problem = Some(problem.into());
        self
    }

    /// 设置证据/数据支持
    pub fn evidence(mut self, evidence: impl Into<String>) -> Self {
        self.evidence = Some(evidence.into());
        self
    }

    /// 添加一条建议
    pub fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// 设置所有建议
    pub fn suggestions(mut self, suggestions: Vec<String>) -> Self {
        self.suggestions = suggestions;
        self
    }

    /// 设置优先级（1-5）
    pub fn priority(mut self, priority: u8) -> Self {
        self.priority = priority.clamp(1, 5); // 限制在1-5范围
        self
    }

    /// 设置分类
    pub fn category(mut self, category: AdviceCategory) -> Self {
        self.category = category;
        self
    }

    /// 设置视角
    pub fn perspective(mut self, perspective: AdvicePerspective) -> Self {
        self.perspective = perspective;
        self
    }

    /// 设置影响的位置
    pub fn affected_role(mut self, role: impl Into<String>) -> Self {
        self.affected_role = Some(role.into());
        self
    }

    /// 设置目标玩家
    pub fn target_player(mut self, name: impl Into<String>) -> Self {
        self.target_player = Some(name.into());
        self
    }

    /// 构建 GameAdvice（检查必填字段）
    pub fn build(self) -> Option<GameAdvice> {
        // 检查必填字段
        let title = self.title?;
        let problem = self.problem?;
        let evidence = self.evidence?;

        // 至少要有1条建议
        if self.suggestions.is_empty() {
            return None;
        }

        Some(GameAdvice {
            title,
            problem,
            evidence,
            suggestions: self.suggestions,
            priority: self.priority as i32,
            category: self.category,
            perspective: self.perspective,
            affected_role: self.affected_role,
            target_player: self.target_player,
        })
    }
}

impl Default for AdviceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_success() {
        let advice = AdviceBuilder::new()
            .title("测试建议")
            .problem("测试问题")
            .evidence("测试证据")
            .suggestion("建议1")
            .suggestion("建议2")
            .priority(5)
            .category(AdviceCategory::Laning)
            .perspective(AdvicePerspective::SelfImprovement)
            .build();

        assert!(advice.is_some());
        let advice = advice.unwrap();
        assert_eq!(advice.title, "测试建议");
        assert_eq!(advice.priority, 5);
        assert_eq!(advice.suggestions.len(), 2);
    }

    #[test]
    fn test_builder_missing_required() {
        let advice = AdviceBuilder::new()
            .title("测试")
            // 缺少 problem 和 evidence
            .suggestion("建议")
            .build();

        assert!(advice.is_none()); // 应该失败
    }

    #[test]
    fn test_builder_no_suggestions() {
        let advice = AdviceBuilder::new()
            .title("测试")
            .problem("问题")
            .evidence("证据")
            // 没有 suggestions
            .build();

        assert!(advice.is_none()); // 应该失败
    }
}
