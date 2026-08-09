//! 排位 Evidence 指标特征 + 由特征生成建议
//!
//! 全局特征入口已迁至 [`super::trait_strategies`]；本模块保留：
//! - `build_deterministic_traits`：排位 Evidence 口径（供 RankedEvidenceTraitStrategy）
//! - `build_deterministic_advice`：仅对已知排位负向 key 产出建议
//!
//! 设计约束：
//! - **不碰原始 JSON**：输入只有已经提取好的 [`MatchEvidence`]
//! - **每条结论都能回指对局**
//! - 样本量不足时不下结论、不产建议
//!
//! 阈值集中在本文件顶部，改口径只需要改一处。

use crate::domains::analysis::evidence::{EvidenceConfidence, GamePhase, MatchEvidence, MIN_SAMPLE_FOR_CONCLUSION};
use crate::shared::types::{AdviceCategory, AdvicePerspective};

use super::types::{DeterministicAdvice, DeterministicTrait, TraitSentiment};

/// 对线期归一化优势的判定带（无量纲份额差，见 `evidence::types`）
const LANING_GOOD_ADVANTAGE: f64 = 0.05;
const LANING_BAD_ADVANTAGE: f64 = -0.05;

/// 对线期 total CS/min 判定带
const FARMING_GOOD_CS_PER_MIN: f64 = 7.0;
const FARMING_BAD_CS_PER_MIN: f64 = 5.0;

/// 场均死亡判定带（越低越好）
const DEATHS_GOOD_PER_GAME: f64 = 4.0;
const DEATHS_BAD_PER_GAME: f64 = 7.0;

/// 场均资源参与（小龙 / 先锋 / 大龙 / 虚空幼虫）判定带
const OBJECTIVE_GOOD_PER_GAME: f64 = 2.5;
const OBJECTIVE_BAD_PER_GAME: f64 = 0.8;

/// 胜率判定带（百分比）
const WIN_RATE_GOOD: f64 = 60.0;
const WIN_RATE_BAD: f64 = 40.0;

/// 指标方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MetricDirection {
    HigherIsBetter,
    LowerIsBetter,
}

/// 单个指标的口径定义
struct MetricSpec {
    key: &'static str,
    good_name: &'static str,
    bad_name: &'static str,
    neutral_name: &'static str,
    direction: MetricDirection,
    good_threshold: f64,
    bad_threshold: f64,
    /// 把均值渲染成人类可读的事实描述
    describe: fn(f64) -> String,
}

impl MetricSpec {
    fn sentiment_of(&self, value: f64) -> TraitSentiment {
        match self.direction {
            MetricDirection::HigherIsBetter => {
                if value >= self.good_threshold {
                    TraitSentiment::Good
                } else if value <= self.bad_threshold {
                    TraitSentiment::Bad
                } else {
                    TraitSentiment::Neutral
                }
            }
            MetricDirection::LowerIsBetter => {
                if value <= self.good_threshold {
                    TraitSentiment::Good
                } else if value >= self.bad_threshold {
                    TraitSentiment::Bad
                } else {
                    TraitSentiment::Neutral
                }
            }
        }
    }

    fn name_of(&self, sentiment: TraitSentiment) -> &'static str {
        match sentiment {
            TraitSentiment::Good => self.good_name,
            TraitSentiment::Bad => self.bad_name,
            TraitSentiment::Neutral => self.neutral_name,
        }
    }
}

/// 一组 (对局, 指标值) 样本
#[derive(Debug, Default)]
struct MetricSamples {
    values: Vec<(u64, f64)>,
}

impl MetricSamples {
    fn len(&self) -> u32 {
        self.values.len() as u32
    }

    fn average(&self) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }
        Some(self.values.iter().map(|(_, value)| value).sum::<f64>() / self.values.len() as f64)
    }

    fn game_ids(&self) -> Vec<u64> {
        let mut ids: Vec<u64> = self.values.iter().map(|(game_id, _)| *game_id).collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// 样本中落在指定倾向上的占比
    fn share_matching(&self, spec: &MetricSpec, sentiment: TraitSentiment) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        let hits = self
            .values
            .iter()
            .filter(|(_, value)| spec.sentiment_of(*value) == sentiment)
            .count();
        hits as f64 / self.values.len() as f64
    }
}

impl FromIterator<(u64, f64)> for MetricSamples {
    fn from_iter<I: IntoIterator<Item = (u64, f64)>>(iter: I) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// === 指标口径 ===

static LANING: MetricSpec = MetricSpec {
    key: "laning_advantage",
    good_name: "对线压制",
    bad_name: "对线劣势",
    neutral_name: "对线均势",
    direction: MetricDirection::HigherIsBetter,
    good_threshold: LANING_GOOD_ADVANTAGE,
    bad_threshold: LANING_BAD_ADVANTAGE,
    describe: |value| {
        let pct = round1(value * 100.0);
        if pct >= 0.0 {
            format!("对线期整体大约领先对手 {pct:.1}%。")
        } else {
            format!("对线期整体大约落后对手 {:.1}%。", pct.abs())
        }
    },
};

static FARMING: MetricSpec = MetricSpec {
    key: "farming_efficiency",
    good_name: "补刀高效",
    bad_name: "补刀偏低",
    neutral_name: "补刀中规中矩",
    direction: MetricDirection::HigherIsBetter,
    good_threshold: FARMING_GOOD_CS_PER_MIN,
    bad_threshold: FARMING_BAD_CS_PER_MIN,
    describe: |value| format!("对线期补刀节奏大约 {:.1} 刀/分钟。", round1(value)),
};

static DEATHS: MetricSpec = MetricSpec {
    key: "death_control",
    good_name: "阵亡克制",
    bad_name: "阵亡偏多",
    neutral_name: "阵亡正常",
    direction: MetricDirection::LowerIsBetter,
    good_threshold: DEATHS_GOOD_PER_GAME,
    bad_threshold: DEATHS_BAD_PER_GAME,
    describe: |value| format!("排位里场均大约倒 {:.1} 次。", round1(value)),
};

static OBJECTIVES: MetricSpec = MetricSpec {
    key: "objective_participation",
    good_name: "资源参与积极",
    bad_name: "资源参与不足",
    neutral_name: "资源参与一般",
    direction: MetricDirection::HigherIsBetter,
    good_threshold: OBJECTIVE_GOOD_PER_GAME,
    bad_threshold: OBJECTIVE_BAD_PER_GAME,
    describe: |value| {
        format!(
            "场均大约参与 {:.1} 次龙/峡谷这类大型资源。",
            round1(value)
        )
    },
};

static WIN_RATE: MetricSpec = MetricSpec {
    key: "win_rate",
    good_name: "近期状态好",
    bad_name: "近期状态低迷",
    neutral_name: "近期状态平稳",
    direction: MetricDirection::HigherIsBetter,
    good_threshold: WIN_RATE_GOOD,
    bad_threshold: WIN_RATE_BAD,
    describe: |value| format!("近期排位胜率大约 {:.0}%。", round1(value)),
};

// === 样本提取 ===

fn laning_samples(matches: &[MatchEvidence]) -> MetricSamples {
    matches
        .iter()
        .filter_map(|evidence| {
            evidence
                .phase(GamePhase::Early)
                .and_then(|phase| phase.opponent_diff.as_ref())
                .map(|diff| (evidence.game_id, diff.overall_advantage))
        })
        .collect()
}

fn farming_samples(matches: &[MatchEvidence]) -> MetricSamples {
    matches
        .iter()
        .filter_map(|evidence| {
            evidence
                .phase(GamePhase::Early)
                .and_then(|phase| phase.cs_per_min)
                .map(|value| (evidence.game_id, value))
        })
        .collect()
}

fn death_samples(matches: &[MatchEvidence]) -> MetricSamples {
    matches
        .iter()
        .filter_map(|evidence| {
            evidence
                .events
                .as_ref()
                .map(|events| (evidence.game_id, events.deaths as f64))
        })
        .collect()
}

fn objective_samples(matches: &[MatchEvidence]) -> MetricSamples {
    matches
        .iter()
        .filter_map(|evidence| {
            evidence.events.as_ref().map(|events| {
                let total =
                    events.dragon_takedowns + events.herald_takedowns + events.baron_takedowns + events.horde_takedowns;
                (evidence.game_id, total as f64)
            })
        })
        .collect()
}

/// 胜率是「整组样本一个值」，因此每局记成 0 / 100，均值即胜率
fn win_rate_samples(matches: &[MatchEvidence]) -> MetricSamples {
    matches
        .iter()
        .map(|evidence| (evidence.game_id, if evidence.win { 100.0 } else { 0.0 }))
        .collect()
}

// === 对外入口 ===

/// 位置敏感指标：只在位置维度产出，且辅助不做补刀评判
fn farming_applies(position: Option<&str>) -> bool {
    matches!(position, Some("TOP") | Some("MID") | Some("ADC") | Some("JUNGLE"))
}

fn objectives_applies(position: Option<&str>) -> bool {
    // 资源参与只在位置维度评判，避免全局把辅助/打野混在一起
    position.is_some()
}

/// 由证据产出确定性特征
///
/// `position` 为 `None` 表示全局特征，否则是该位置维度的特征。
/// 全局不产出 FARMING / OBJECTIVES（位置敏感）；SUPPORT 不产出补刀特征。
pub fn build_deterministic_traits(matches: &[MatchEvidence], position: Option<&str>) -> Vec<DeterministicTrait> {
    let mut specs: Vec<(&MetricSpec, MetricSamples)> = vec![
        (&LANING, laning_samples(matches)),
        (&DEATHS, death_samples(matches)),
        (&WIN_RATE, win_rate_samples(matches)),
    ];

    if farming_applies(position) {
        specs.push((&FARMING, farming_samples(matches)));
    }
    if objectives_applies(position) {
        specs.push((&OBJECTIVES, objective_samples(matches)));
    }

    specs
        .into_iter()
        .filter_map(|(spec, samples)| build_trait(spec, &samples, position))
        .collect()
}

fn build_trait(spec: &MetricSpec, samples: &MetricSamples, position: Option<&str>) -> Option<DeterministicTrait> {
    let average = samples.average()?;
    let sample_count = samples.len();
    let confidence = EvidenceConfidence::from_sample_size(sample_count);
    let supports_conclusion = confidence.supports_conclusion();

    // 样本不足时只描述事实，不给好/坏定性
    let sentiment = if supports_conclusion {
        spec.sentiment_of(average)
    } else {
        TraitSentiment::Neutral
    };

    let description = if supports_conclusion {
        (spec.describe)(average)
    } else {
        format!(
            "{}（最近只有 {} 场，多打几把再看）",
            (spec.describe)(average),
            sample_count
        )
    };

    Some(DeterministicTrait {
        key: spec.key.to_string(),
        name: spec.name_of(sentiment).to_string(),
        description,
        sentiment,
        sample_count,
        frequency: round2(samples.share_matching(spec, sentiment)),
        confidence,
        supports_conclusion,
        evidence_game_ids: samples.game_ids(),
        position: position.map(str::to_string),
    })
}

/// 由特征产出确定性建议
///
/// 只有「负向且样本量足够」的特征才会变成建议：这是「不许拿一场比赛教育用户」的执行点。
pub fn build_deterministic_advice(
    traits: &[DeterministicTrait],
    perspective: AdvicePerspective,
    target_player: Option<&str>,
) -> Vec<DeterministicAdvice> {
    let mut advice: Vec<DeterministicAdvice> = traits
        .iter()
        .filter(|item| item.supports_conclusion && item.sentiment == TraitSentiment::Bad)
        .filter_map(|item| build_advice(item, perspective, target_player))
        .collect();

    advice.sort_by(|a, b| b.priority.cmp(&a.priority).then_with(|| a.key.cmp(&b.key)));
    advice
}

fn build_advice(
    item: &DeterministicTrait,
    perspective: AdvicePerspective,
    target_player: Option<&str>,
) -> Option<DeterministicAdvice> {
    let (title, problem, priority, category, suggestions) = match item.key.as_str() {
        "laning_advantage" => (
            "对线期被压制",
            "对线期综合数据落后于对位",
            9,
            AdviceCategory::Laning,
            vec![
                "开局先确认对位英雄的强势期，避开对方等级/装备节点的换血",
                "被压时优先保证经验，宁可少补几刀也不要送掉传送/闪现",
                "让打野在对线期给一次反蹲，把线权抢回来",
            ],
        ),
        "farming_efficiency" => {
            // 辅助绝不产出补刀建议（双重保险，traits 层已过滤）
            if item.position.as_deref() == Some("SUPPORT") {
                return None;
            }
            (
                "补刀效率偏低",
                "对线期每分钟补刀低于健康水平",
                8,
                AdviceCategory::Farming,
                vec![
                    "把「回城前清完一波兵」当成固定动作，减少漏兵",
                    "游走或参团后先回线补一波再做下一件事",
                    "训练模式练 10 分钟补刀，先把基本功拉到稳定水平",
                ],
            )
        }
        "death_control" => (
            "阵亡次数偏多",
            "场均阵亡明显偏高，等于持续给对面送节奏",
            9,
            AdviceCategory::Positioning,
            vec![
                "开团前先确认对方关键技能是否交出",
                "视野缺失时默认对方打野在自己这半区，收缩到塔下发育",
                "劣势期把目标从「找机会」改成「不送」，等队友起来",
            ],
        ),
        "objective_participation" => (
            "资源参与不足",
            "小龙 / 先锋 / 大龙的参与率偏低",
            7,
            AdviceCategory::Decision,
            vec![
                "资源刷新前 30 秒开始清线并向该区域靠拢",
                "把兵线处理和资源计时绑定，避免「刚推完线资源就没了」",
                "无法到场时至少在对侧制造压力，换取资源交换",
            ],
        ),
        _ => return None,
    };

    Some(DeterministicAdvice {
        key: item.key.clone(),
        title: title.to_string(),
        problem: problem.to_string(),
        evidence: format!(
            "{}；其中 {:.0}% 的对局符合该判定",
            item.description,
            item.frequency * 100.0
        ),
        suggestions: suggestions.into_iter().map(str::to_string).collect(),
        priority,
        category,
        perspective,
        sample_count: item.sample_count,
        confidence: item.confidence,
        evidence_game_ids: item.evidence_game_ids.clone(),
        position: item.position.clone(),
        target_player: target_player.map(str::to_string),
    })
}
