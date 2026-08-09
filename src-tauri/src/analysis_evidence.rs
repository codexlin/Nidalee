//! 确定性证据层的最窄公共门面
//!
//! `domains` 保持 crate 私有，外部（集成测试、类型导出）只能通过这里访问
//! `domains::analysis::evidence`，不暴露领域层其余实现细节。
//!
//! 这里只做重导出，不定义任何新语义。
//!
//! 证据层只能经由本门面访问：
//!
//! ```
//! use nidalee_lib::analysis_evidence::extract_match_evidence;
//! ```
//!
//! 领域层路径对外不可见（若下面这段能编译，说明 `domains` 被意外放宽为公开）：
//!
//! ```compile_fail
//! use nidalee_lib::domains::analysis::evidence::extract_match_evidence;
//! ```

pub use crate::domains::analysis::evidence::{
    advantage_to_percent, aggregate_match_evidence, build_evidence_bundle, extract_event_evidence,
    extract_match_evidence, is_aram_queue, laning_opponent_diff, normalized_share_advantage, position_from_role_lane,
    resolve_lane_opponent, ActivityContext, DeathCause, EventEvidence, EventExtractContext, EventInvolvement,
    EvidenceBundle, EvidenceConfidence, EvidenceDiagnostic, EvidenceEventKind, EvidenceEventRates,
    EvidenceExtractionError, EvidenceIssue, EvidencePosition, EvidenceQuality, EvidenceSummary, GamePhase,
    KeyEventEvidence, MatchEvidence, MatchEvidenceInput, OpponentEvidence, OpponentMatchMethod, PhaseAverages,
    PhaseEvidence, PhaseOpponentDiff, TimelineSpan, UnknownTimelineEvent, ADVANTAGE_PERCENT_SCALE, ADVANTAGE_WEIGHT_CS,
    ADVANTAGE_WEIGHT_GOLD, ADVANTAGE_WEIGHT_XP, MIN_SAMPLE_FOR_CONCLUSION, PHASE_EARLY_END_MS, PHASE_MID_END_MS,
};
pub use crate::domains::analysis::pipeline::process_insights::build_process_insight;
