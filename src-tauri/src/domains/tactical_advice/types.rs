/// 建议系统核心数据类型
///
/// 重新导出 shared::types 中的类型，避免重复定义
pub use crate::shared::types::{AdviceCategory, AdvicePerspective, GameAdvice};

impl AdviceCategory {
    /// 获取分类图标
    pub fn icon(&self) -> &str {
        match self {
            AdviceCategory::Laning => "⚔️",
            AdviceCategory::Farming => "💰",
            AdviceCategory::Teamfight => "🤝",
            AdviceCategory::Vision => "👁️",
            AdviceCategory::Positioning => "📍",
            AdviceCategory::Decision => "🧠",
            AdviceCategory::Champion => "🎮",
        }
    }

    /// 获取分类名称
    pub fn name(&self) -> &str {
        match self {
            AdviceCategory::Laning => "对线",
            AdviceCategory::Farming => "发育",
            AdviceCategory::Teamfight => "团战",
            AdviceCategory::Vision => "视野",
            AdviceCategory::Positioning => "站位",
            AdviceCategory::Decision => "决策",
            AdviceCategory::Champion => "英雄池",
        }
    }
}

impl AdvicePerspective {
    /// 获取视角描述
    pub fn description(&self) -> &str {
        match self {
            AdvicePerspective::SelfImprovement => "个人改进建议",
            AdvicePerspective::Targeting => "针对战术建议",
            AdvicePerspective::Collaboration => "团队协作建议",
        }
    }
}
