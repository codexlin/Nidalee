//! 娱乐模式表现特征（无对线/补刀口径）

use super::{TraitAnalysisContext, TraitStrategy};
use crate::domains::analysis::analyzers::core::parser::ParsedGame;
use crate::domains::analysis::evidence::{EvidenceConfidence, MIN_SAMPLE_FOR_CONCLUSION};
use crate::domains::analysis::pipeline::types::{is_ranked_queue, DeterministicTrait, TraitSentiment};

const DEATHS_GOOD: f64 = 5.0;
const DEATHS_BAD: f64 = 9.0;
const KDA_GOOD: f64 = 3.5;
const KDA_BAD: f64 = 1.8;
const DAMAGE_SHARE_GOOD: f64 = 0.22;
const DAMAGE_SHARE_BAD: f64 = 0.12;

pub struct FunModeTraitStrategy;

impl TraitStrategy for FunModeTraitStrategy {
    fn analyze(&self, ctx: &TraitAnalysisContext<'_>) -> Vec<DeterministicTrait> {
        let fun_games: Vec<&ParsedGame> = ctx
            .display_games
            .iter()
            .filter(|g| !is_ranked_queue(g.queue_id))
            .collect();
        if fun_games.len() < MIN_SAMPLE_FOR_CONCLUSION as usize {
            return Vec::new();
        }

        let mut out = Vec::new();
        if let Some(t) = death_trait(&fun_games) {
            out.push(t);
        }
        if let Some(t) = kda_trait(&fun_games) {
            out.push(t);
        }
        if let Some(t) = damage_share_trait(&fun_games) {
            out.push(t);
        }
        out
    }
}

fn death_trait(games: &[&ParsedGame]) -> Option<DeterministicTrait> {
    let values: Vec<(u64, f64)> = games.iter().map(|g| (g.game_id, g.player_data.deaths as f64)).collect();
    let avg = mean(&values)?;
    let sentiment = if avg <= DEATHS_GOOD {
        TraitSentiment::Good
    } else if avg >= DEATHS_BAD {
        TraitSentiment::Bad
    } else {
        TraitSentiment::Neutral
    };
    let (name, key, blurb) = match sentiment {
        TraitSentiment::Good => (
            "乱斗稳如泰山",
            "fun_death_control",
            format!("娱乐局里站得很稳，场均大约只倒 {:.1} 次。", round1(avg)),
        ),
        TraitSentiment::Bad => (
            "乱斗送货员",
            "fun_death_control",
            format!("娱乐局里倒得有点勤，场均大约 {:.1} 次阵亡，注意别白送。", round1(avg)),
        ),
        TraitSentiment::Neutral => (
            "乱斗阵亡正常",
            "fun_death_control",
            format!("娱乐局场均阵亡大约 {:.1} 次，中规中矩。", round1(avg)),
        ),
    };
    Some(make_trait(key, name, blurb, sentiment, &values))
}

fn kda_trait(games: &[&ParsedGame]) -> Option<DeterministicTrait> {
    let values: Vec<(u64, f64)> = games.iter().map(|g| (g.game_id, g.player_data.kda)).collect();
    let avg = mean(&values)?;
    let sentiment = if avg >= KDA_GOOD {
        TraitSentiment::Good
    } else if avg <= KDA_BAD {
        TraitSentiment::Bad
    } else {
        TraitSentiment::Neutral
    };
    let (name, key, blurb) = match sentiment {
        TraitSentiment::Good => (
            "乱斗高光选手",
            "fun_kda",
            format!("娱乐局数据很亮眼，场均 KDA 大约 {:.1}。", round1(avg)),
        ),
        TraitSentiment::Bad => (
            "乱斗发挥低迷",
            "fun_kda",
            format!("娱乐局数据偏闷，场均 KDA 大约 {:.1}，可以再凶一点。", round1(avg)),
        ),
        TraitSentiment::Neutral => (
            "乱斗发挥平稳",
            "fun_kda",
            format!("娱乐局发挥平稳，场均 KDA 大约 {:.1}。", round1(avg)),
        ),
    };
    Some(make_trait(key, name, blurb, sentiment, &values))
}

fn damage_share_trait(games: &[&ParsedGame]) -> Option<DeterministicTrait> {
    let values: Vec<(u64, f64)> = games
        .iter()
        .filter_map(|g| {
            let team = g.team_data.total_damage_to_champions;
            if team <= 0 {
                return None;
            }
            Some((g.game_id, g.player_data.damage_to_champions as f64 / team as f64))
        })
        .collect();
    if values.len() < MIN_SAMPLE_FOR_CONCLUSION as usize {
        return None;
    }
    let avg = mean(&values)?;
    let sentiment = if avg >= DAMAGE_SHARE_GOOD {
        TraitSentiment::Good
    } else if avg <= DAMAGE_SHARE_BAD {
        TraitSentiment::Bad
    } else {
        TraitSentiment::Neutral
    };
    let (name, key, blurb) = match sentiment {
        TraitSentiment::Good => (
            "乱斗输出机器",
            "fun_damage_share",
            format!("娱乐局里你很能打，队内伤害大约占 {:.0}%。", round1(avg * 100.0)),
        ),
        TraitSentiment::Bad => (
            "乱斗输出偏少",
            "fun_damage_share",
            format!("娱乐局里输出份额偏低，大约只占队内 {:.0}%。", round1(avg * 100.0)),
        ),
        TraitSentiment::Neutral => (
            "乱斗输出中规中矩",
            "fun_damage_share",
            format!("娱乐局输出份额中规中矩，大约占队内 {:.0}%。", round1(avg * 100.0)),
        ),
    };
    Some(make_trait(key, name, blurb, sentiment, &values))
}

fn make_trait(
    key: &str,
    name: &str,
    fact: String,
    sentiment: TraitSentiment,
    values: &[(u64, f64)],
) -> DeterministicTrait {
    let sample_count = values.len() as u32;
    let confidence = EvidenceConfidence::from_sample_size(sample_count);
    let supports_conclusion = confidence.supports_conclusion();
    let sentiment = if supports_conclusion {
        sentiment
    } else {
        TraitSentiment::Neutral
    };
    let description = if supports_conclusion {
        fact
    } else {
        format!("{}（娱乐局还只有 {} 场，多玩几把再下结论）", fact, sample_count)
    };
    let mut ids: Vec<u64> = values.iter().map(|(id, _)| *id).collect();
    ids.sort_unstable();
    DeterministicTrait {
        key: key.to_string(),
        name: name.to_string(),
        description,
        sentiment,
        sample_count,
        frequency: 1.0,
        confidence,
        supports_conclusion,
        evidence_game_ids: ids,
        position: None,
    }
}

fn mean(values: &[(u64, f64)]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().map(|(_, v)| *v).sum::<f64>() / values.len() as f64)
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
