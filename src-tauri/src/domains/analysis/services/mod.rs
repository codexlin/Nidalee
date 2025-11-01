/// 分析服务模块
///
/// 职责：
/// - 提供高级分析服务
/// - 整合多个分析器
/// - 生成综合性的分析结果
/// - 整合新旧系统
pub mod intelligent_analysis_service;
pub mod enhanced_analysis_service;

// 重新导出智能分析服务
pub use intelligent_analysis_service::{
    perform_intelligent_analysis,
    IntelligentAnalysisResult,
    ComprehensiveAdvice,
    AdviceTarget,
    TacticalSummary,
};

// 注意：EnhancedAnalysisService 已废弃，已集成到旧系统中
// 如需使用，请参考 infrastructure/match_management/matches/service.rs
