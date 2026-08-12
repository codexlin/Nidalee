//! Match-analysis domain.
//!
//! `evidence` extracts deterministic facts, while `pipeline` selects policy and builds the
//! application-facing result. Infrastructure code should enter through those two modules.

pub mod analyzers;
pub mod evidence;
pub mod pipeline;
pub mod queue_config;
pub mod realtime;
pub mod services;
pub mod thresholds;

pub use evidence::{extract_match_evidence, EvidenceQuality, MatchEvidence};
