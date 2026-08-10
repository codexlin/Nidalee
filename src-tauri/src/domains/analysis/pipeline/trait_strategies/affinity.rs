//! 模式身份特征：海克斯常驻 / 乱斗选手 / 娱乐为主 / 排位为主

use super::{TraitAnalysisContext, TraitStrategy};
use crate::domains::analysis::analyzers::core::parser::ParsedGame;
use crate::domains::analysis::evidence::EvidenceConfidence;
use crate::domains::analysis::pipeline::types::{is_ranked_queue, DeterministicTrait, TraitSentiment};
use crate::domains::analysis::queue_config::QueueType;

/// 占比阈值（含）且至少这么多场才打身份标签
const AFFINITY_RATIO: f64 = 0.6;
const AFFINITY_MIN_GAMES: u32 = 3;

pub struct ModeAffinityTraitStrategy;

impl TraitStrategy for ModeAffinityTraitStrategy {
    fn id(&self) -> &'static str {
        "mode_affinity"
    }

    fn analyze(&self, ctx: &TraitAnalysisContext<'_>) -> Vec<DeterministicTrait> {
        let games = ctx.display_games;
        let total = games.len() as u32;
        if total < AFFINITY_MIN_GAMES {
            return Vec::new();
        }

        let hextech = count_queue(games, 2400);
        let aram = count_queue(games, 450);
        let ranked = games.iter().filter(|g| is_ranked_queue(g.queue_id)).count() as u32;
        let fun = total.saturating_sub(ranked);

        let mut out = Vec::new();

        if let Some(trait_) = affinity_trait(
            "mode_affinity_hextech",
            "海克斯常驻",
            hextech,
            total,
            games,
            |g| g.queue_id == 2400,
            |count, total, pct| format!("最近 {count}/{total} 场都在海克斯大乱斗（{pct:.0}%），已经玩出肌肉记忆了。"),
        ) {
            out.push(trait_);
        } else if let Some(trait_) = affinity_trait(
            "mode_affinity_aram",
            "乱斗选手",
            aram,
            total,
            games,
            |g| g.queue_id == 450,
            |count, total, pct| format!("最近 {count}/{total} 场泡在极地大乱斗（{pct:.0}%），乱斗魂拉满。"),
        ) {
            out.push(trait_);
        } else if fun as f64 / total as f64 >= AFFINITY_RATIO && fun >= AFFINITY_MIN_GAMES {
            if let Some(trait_) = affinity_trait(
                "mode_affinity_fun",
                "娱乐为主",
                fun,
                total,
                games,
                |g| !is_ranked_queue(g.queue_id),
                |count, total, pct| format!("最近 {count}/{total} 场都是娱乐局（{pct:.0}%），排位可以先放一放。"),
            ) {
                out.push(trait_);
            }
        } else if ranked as f64 / total as f64 >= AFFINITY_RATIO && ranked >= AFFINITY_MIN_GAMES {
            if let Some(trait_) = affinity_trait(
                "mode_affinity_ranked",
                "排位为主",
                ranked,
                total,
                games,
                |g| is_ranked_queue(g.queue_id),
                |count, total, pct| format!("最近 {count}/{total} 场在打排位（{pct:.0}%），认真上分的状态。"),
            ) {
                out.push(trait_);
            }
        }

        if out.is_empty() {
            if let Some(dominant) = dominant_named_queue(games, total) {
                out.push(dominant);
            }
        }

        out
    }
}

fn count_queue(games: &[ParsedGame], queue_id: i64) -> u32 {
    games.iter().filter(|g| g.queue_id == queue_id).count() as u32
}

fn affinity_trait(
    key: &str,
    name: &str,
    count: u32,
    total: u32,
    games: &[ParsedGame],
    pred: impl Fn(&ParsedGame) -> bool,
    describe: impl Fn(u32, u32, f64) -> String,
) -> Option<DeterministicTrait> {
    if count < AFFINITY_MIN_GAMES || (count as f64 / total as f64) < AFFINITY_RATIO {
        return None;
    }
    let ratio = count as f64 / total as f64;
    let mut ids: Vec<u64> = games.iter().filter(|g| pred(g)).map(|g| g.game_id).collect();
    ids.sort_unstable();
    Some(DeterministicTrait {
        key: key.to_string(),
        name: name.to_string(),
        description: describe(count, total, ratio * 100.0),
        sentiment: TraitSentiment::Neutral,
        sample_count: count,
        frequency: (ratio * 100.0).round() / 100.0,
        confidence: EvidenceConfidence::from_sample_size(count),
        supports_conclusion: true,
        evidence_game_ids: ids,
        position: None,
    })
}

fn dominant_named_queue(games: &[ParsedGame], total: u32) -> Option<DeterministicTrait> {
    let mut best: Option<(i64, u32)> = None;
    for g in games {
        let n = count_queue(games, g.queue_id);
        if best.map(|(_, c)| n > c).unwrap_or(true) {
            best = Some((g.queue_id, n));
        }
    }
    let (queue_id, count) = best?;
    if count < AFFINITY_MIN_GAMES || (count as f64 / total as f64) < AFFINITY_RATIO {
        return None;
    }
    if matches!(queue_id, 420 | 440 | 450 | 2400) {
        return None;
    }
    let label = QueueType::from_queue_id(queue_id as i32).name();
    affinity_trait(
        &format!("mode_affinity_queue_{queue_id}"),
        &format!("{label}爱好者"),
        count,
        total,
        games,
        |g| g.queue_id == queue_id,
        move |count, total, pct| format!("最近 {count}/{total} 场都在玩{label}（{pct:.0}%），算半个常驻了。"),
    )
}
