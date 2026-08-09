//! 对局分析契约的最窄公共门面
//!
//! `domains` 保持 crate 私有，外部（集成测试、类型导出）只能通过这里访问
//! `domains::analysis::pipeline` 的契约与策略解析，不暴露领域层其余实现细节。
//!
//! 这里只做重导出，不定义任何新语义。
//!
//! 契约只能经由本门面访问：
//!
//! ```
//! use nidalee_lib::analysis_contract::MatchAnalysisRequest;
//! ```
//!
//! 领域层路径对外不可见（若下面这段能编译，说明 `domains` 被意外放宽为公开）：
//!
//! ```compile_fail
//! use nidalee_lib::domains::analysis::pipeline::MatchAnalysisRequest;
//! ```

pub use crate::domains::analysis::pipeline::{
    is_ranked_queue, orchestrate_analysis, resolve_analysis_policy, AnalysisCapabilities, AnalysisDegradationCode,
    AnalysisDepth, AnalysisDiagnostic, AnalysisFeature, AnalysisFeatureFlags, AnalysisMode, AnalysisPolicy,
    AnalysisQueueScope, DeepMatchInput, DeterministicAdvice, DeterministicTrait, MatchAnalysisRequest,
    MatchAnalysisResult, OrchestratorInput, TraitSentiment, DEFAULT_ANALYSIS_GAME_COUNT, UNKNOWN_POSITION,
};

/// 建议视角属于请求契约的一部分，随契约一起对外暴露
pub use crate::shared::types::AdvicePerspective;
