//! 排位证据特征策略：委托现有 Evidence 指标口径

use super::{TraitAnalysisContext, TraitStrategy};
use crate::domains::analysis::pipeline::insights::build_deterministic_traits;
use crate::domains::analysis::pipeline::types::DeterministicTrait;

pub struct RankedEvidenceTraitStrategy;

impl TraitStrategy for RankedEvidenceTraitStrategy {
    fn analyze(&self, ctx: &TraitAnalysisContext<'_>) -> Vec<DeterministicTrait> {
        if ctx.evidence_matches.is_empty() {
            return Vec::new();
        }
        build_deterministic_traits(ctx.evidence_matches, ctx.position)
    }
}
