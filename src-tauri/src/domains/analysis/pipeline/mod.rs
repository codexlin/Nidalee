/// 对局分析管线（统一契约层 + 唯一编排器）
///
/// 职责边界：
/// - `types`: 请求 / 功能开关 / 策略 / 结果 / 能力 / 降级诊断的唯一定义
/// - `policy`: 唯一的策略解析函数（请求 → 策略）
/// - `insights`: 由 Evidence 产出确定性特征与建议（唯一来源）
/// - `orchestrator`: 唯一的分析执行路径（纯领域，不做数据抓取、不调用 AI）
///
/// 数据抓取由 `infrastructure` 的统一获取层负责，编排器只吃纯视图。
pub mod insights;
pub mod orchestrator;
pub mod policy;
pub mod process_insights;
pub mod trait_strategies;
pub mod types;

pub use orchestrator::{dedupe_diagnostics, orchestrate_analysis, DeepMatchInput, OrchestratorInput};
pub use policy::resolve_analysis_policy;
pub use trait_strategies::{analyze_traits, TraitAnalysisContext};
pub use types::{
    is_ranked_queue, AnalysisCapabilities, AnalysisDegradationCode, AnalysisDepth, AnalysisDiagnostic, AnalysisFeature,
    AnalysisFeatureFlags, AnalysisMode, AnalysisPolicy, AnalysisQueueScope, DeterministicAdvice, DeterministicTrait,
    MatchAnalysisRequest, MatchAnalysisResult, TraitSentiment, DEFAULT_ANALYSIS_GAME_COUNT, UNKNOWN_POSITION,
};
