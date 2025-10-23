/// 通用玩家战绩分析器模块（优化架构 v2）
///
/// 核心设计：
/// - Parser 模式：隔离 LCU 数据结构变化
/// - Strategy 模式：决定分析深度（排位=深度分析，其他=简化分析）
/// - Thresholds 模块：独立管理所有阈值
/// - 5层分析：基础 → 深度 → 位置 → 分布 → 优化
///
/// 架构：
/// - parser.rs: LCU 数据解析层（唯一需要适配 API 变化的地方）
/// - strategy.rs: 分析策略（决定使用哪些分析层）
/// - thresholds.rs: 阈值配置（独立维护，便于调整）
/// - analyzer.rs: 量化指标计算
/// - traits_analyzer.rs: 基础特征分析
/// - advanced_analyzer.rs: 深度特征分析（仅排位模式）
/// - role_analyzer.rs: 位置特征分析（仅排位模式）
/// - distribution_analyzer.rs: 分布特征分析（仅排位模式）
/// - trait_merger.rs: 智能去重优化

/// 对局分析领域 (Analysis Domain)
///
/// 新架构：
/// - analyzers/core: 核心分析（解析、统计、策略）
/// - analyzers/traits: 特征分析（6个分析器）
/// - analyzers/opponent_analyzer: 对手分析器
/// - analyzers/teammate_analyzer: 队友分析器
/// - analyzers/self_improvement_analyzer: 自我提升分析器
/// - services: 高级分析服务
/// - thresholds: 阈值配置
/// - queue_config: 队列特定配置
pub mod analyzers;
pub mod services;
pub mod thresholds;
pub mod queue_config;

// 重新导出核心API（保持向后兼容）
pub use analyzers::core::parser::{identify_main_role, identify_position_from_game, parse_games, ParsedGame};
pub use analyzers::core::stats::{analyze_player_stats, AnalysisContext};
pub use analyzers::core::strategy::AnalysisStrategy;

pub use analyzers::traits::advanced::analyze_advanced_traits;
pub use analyzers::traits::basic::analyze_traits;
pub use analyzers::traits::distribution::{analyze_distribution_traits, analyze_win_loss_pattern};
pub use analyzers::traits::merger::optimize_traits;
pub use analyzers::traits::role::{analyze_role_based_traits, identify_player_roles};
pub use analyzers::traits::timeline::analyze_timeline_traits;

// 重新导出新的智能分析API
pub use services::{
    perform_intelligent_analysis,
    IntelligentAnalysisResult,
    ComprehensiveAdvice,
    AdviceTarget,
    TacticalSummary,
    EnhancedAnalysisService,
    AnalysisConfig,
    UnifiedAnalysisResult,
    AnalysisMetadata,
    analyze_with_default_config,
    analyze_with_full_features,
};
