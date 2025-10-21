/// 策略模式
///
/// 核心设计思想：
/// - 排位模式：使用完整的5层深度分析（核心功能）
/// - 其他模式：使用简化的2层基础分析（快速展示）
///
/// 目的：
/// - 区分分析的复杂度和深度
/// - 为排位模式提供最精准的洞察
/// - 为其他模式节省计算资源
///
/// 阈值配置独立维护在 thresholds.rs
use super::parser::ParsedGame;

/// 分析策略（决定分析深度）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisStrategy {
    /// 排位模式 - 完整深度分析
    /// - 5层分析：基础 → 深度 → 位置 → 分布 → 胜负
    /// - 位置识别
    /// - 稳定性分析
    /// - 趋势分析
    /// - 高光/崩盘识别
    Ranked,

    /// 其他模式 - 简化快速分析
    /// - 2层分析：基础 → 简单胜负
    /// - 基础统计（胜率、KDA、连胜）
    /// - 简单评价
    Other,
}

impl AnalysisStrategy {
    /// 根据队列ID自动选择策略
    pub fn from_queue_id(queue_id: i64) -> Self {
        match queue_id {
            420 | 440 => AnalysisStrategy::Ranked, // 单排/灵活排 - 核心模式
            _ => AnalysisStrategy::Other,          // 其他所有模式 - 简化分析
        }
    }

    /// 根据对局列表自动推断策略（取主要模式）
    pub fn from_games(games: &[ParsedGame]) -> Self {
        if games.is_empty() {
            return AnalysisStrategy::Other;
        }

        // 统计各队列的场次
        let mut queue_counts: std::collections::HashMap<i64, u32> = std::collections::HashMap::new();

        for game in games {
            *queue_counts.entry(game.queue_id).or_insert(0) += 1;
        }

        // 找出最多的队列
        if let Some((main_queue, _)) = queue_counts.iter().max_by_key(|(_, count)| *count) {
            Self::from_queue_id(*main_queue)
        } else {
            AnalysisStrategy::Other
        }
    }

    /// 是否启用深度分析（参团率、伤害占比、稳定性、趋势）
    pub fn enable_advanced_analysis(&self) -> bool {
        matches!(self, AnalysisStrategy::Ranked)
    }

    /// 是否启用位置分析（位置识别、位置专项评价）
    pub fn enable_role_analysis(&self) -> bool {
        matches!(self, AnalysisStrategy::Ranked)
    }

    /// 是否启用分布分析（高光时刻、崩盘场次、稳定性分布）
    pub fn enable_distribution_analysis(&self) -> bool {
        matches!(self, AnalysisStrategy::Ranked)
    }

    /// 获取特征数量限制
    pub fn max_traits(&self) -> usize {
        match self {
            AnalysisStrategy::Ranked => 12, // 排位模式：展示更多特征
            AnalysisStrategy::Other => 6,   // 其他模式：精简特征
        }
    }

    /// 获取分析层次描述（用于日志）
    pub fn analysis_layers(&self) -> &str {
        match self {
            AnalysisStrategy::Ranked => "5层完整分析（基础→深度→位置→分布→胜负）",
            AnalysisStrategy::Other => "2层简化分析（基础→简单胜负）",
        }
    }

    /// 获取策略描述
    pub fn description(&self) -> &str {
        match self {
            AnalysisStrategy::Ranked => "排位模式 - 核心深度分析",
            AnalysisStrategy::Other => "娱乐模式 - 快速简化分析",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ranked_strategy() {
        let strategy = AnalysisStrategy::from_queue_id(420);
        assert_eq!(strategy, AnalysisStrategy::Ranked);
        assert!(strategy.enable_advanced_analysis());
        assert!(strategy.enable_role_analysis());
        assert!(strategy.enable_distribution_analysis());
        assert_eq!(strategy.max_traits(), 12);
    }

    #[test]
    fn test_other_strategy() {
        let strategy = AnalysisStrategy::from_queue_id(450); // ARAM
        assert_eq!(strategy, AnalysisStrategy::Other);
        assert!(!strategy.enable_advanced_analysis());
        assert!(!strategy.enable_role_analysis());
        assert!(!strategy.enable_distribution_analysis());
        assert_eq!(strategy.max_traits(), 6);
    }
}
