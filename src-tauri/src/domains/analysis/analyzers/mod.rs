/// 分析器模块
///
/// 分组：
/// - core: 核心分析（统计、解析、策略）
/// - traits: 特征分析（6个分析器）
/// - opponent_analyzer: 对手分析器
/// - teammate_analyzer: 队友分析器
/// - self_improvement_analyzer: 自我提升分析器
pub mod core;
pub mod opponent_analyzer;
pub mod self_improvement_analyzer;
pub mod teammate_analyzer;
pub mod traits;

// 重新导出核心API
