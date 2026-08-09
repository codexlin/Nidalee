//! 召唤师特征策略（Strategy）
//!
//! 同一抽象、多种算法：排位吃 Evidence，娱乐/身份吃展示局 ParsedGame。
//! 静态组合三种策略，不做运行时插件注册表。

mod affinity;
mod fun;
mod ranked;

use crate::domains::analysis::analyzers::core::parser::ParsedGame;
use crate::domains::analysis::evidence::MatchEvidence;
use crate::domains::analysis::pipeline::types::is_ranked_queue;

use super::types::DeterministicTrait;

pub use affinity::ModeAffinityTraitStrategy;
pub use fun::FunModeTraitStrategy;
pub use ranked::RankedEvidenceTraitStrategy;

/// 全局特征最多展示条数
const MAX_GLOBAL_TRAITS: usize = 6;

/// 策略输入上下文
#[derive(Debug, Clone, Copy)]
pub struct TraitAnalysisContext<'a> {
    pub display_games: &'a [ParsedGame],
    pub evidence_matches: &'a [MatchEvidence],
    /// `None` = 全局；`Some` = 位置维度（仅排位策略）
    pub position: Option<&'a str>,
}

/// 特征策略抽象
pub trait TraitStrategy {
    fn id(&self) -> &'static str;
    fn analyze(&self, ctx: &TraitAnalysisContext<'_>) -> Vec<DeterministicTrait>;
}

/// 按样本选择并合并策略
pub fn analyze_traits(ctx: &TraitAnalysisContext<'_>) -> Vec<DeterministicTrait> {
    if ctx.position.is_some() {
        return RankedEvidenceTraitStrategy.analyze(ctx);
    }

    let mut parts: Vec<Vec<DeterministicTrait>> = Vec::new();
    parts.push(ModeAffinityTraitStrategy.analyze(ctx));

    if !ctx.evidence_matches.is_empty() {
        parts.push(RankedEvidenceTraitStrategy.analyze(ctx));
    }

    if ctx.display_games.iter().any(|g| !is_ranked_queue(g.queue_id)) {
        parts.push(FunModeTraitStrategy.analyze(ctx));
    }

    merge_traits(parts, MAX_GLOBAL_TRAITS)
}

/// 同 key 去重（先到优先），截断上限
fn merge_traits(parts: Vec<Vec<DeterministicTrait>>, cap: usize) -> Vec<DeterministicTrait> {
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for part in parts {
        for item in part {
            if !seen.insert(item.key.clone()) {
                continue;
            }
            out.push(item);
            if out.len() >= cap {
                return out;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::analysis::evidence::EvidenceConfidence;
    use crate::domains::analysis::pipeline::types::TraitSentiment;

    fn dummy(key: &str) -> DeterministicTrait {
        DeterministicTrait {
            key: key.to_string(),
            name: key.to_string(),
            description: String::new(),
            sentiment: TraitSentiment::Neutral,
            sample_count: 3,
            frequency: 1.0,
            confidence: EvidenceConfidence::Low,
            supports_conclusion: true,
            evidence_game_ids: vec![1],
            position: None,
        }
    }

    #[test]
    fn merge_keeps_first_key_and_caps() {
        let merged = merge_traits(
            vec![vec![dummy("a"), dummy("b")], vec![dummy("a"), dummy("c"), dummy("d")]],
            3,
        );
        assert_eq!(
            merged.iter().map(|t| t.key.as_str()).collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
    }
}
