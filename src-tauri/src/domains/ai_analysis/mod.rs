//! 本地 AI 分析领域（BYOK）
//!
//! 约束：
//! - 只接收已脱敏的聚合 Evidence，不上传完整 Timeline / PUUID / 召唤师名
//! - 模型输出必须是可 Serde 校验的结构化 JSON；失败最多重试一次
//! - AI **不参与**基础数值计算，只产出解读与建议

pub mod prompt;
pub mod types;

pub use prompt::{build_ai_prompt, compact_evidence_for_ai};
pub use types::{AiInsight, AiInsightFinding, AiInsightSuggestion, AiPromptBundle};
