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

// 重新导出增强分析服务
pub use enhanced_analysis_service::{
    EnhancedAnalysisService,
    AnalysisConfig,
    UnifiedAnalysisResult,
    AnalysisMetadata,
    analyze_with_default_config,
    analyze_with_full_features,
};
