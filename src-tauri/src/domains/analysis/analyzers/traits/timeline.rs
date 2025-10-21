use crate::domains::analysis::analyzers::core::parser::{ParsedGame, TimelineData};
use crate::domains::analysis::thresholds;
/// 时间线特征分析器
///
/// 职责：
/// - 基于分阶段数据分析对线期、发育期、后期表现
/// - 识别对线压制力、发育稳定性、成长曲线
/// - 生成时间线相关的特征标签
///
/// 核心指标：
/// - CS差（补刀差）：判断对线压制力
/// - 经验差：判断等级优势
/// - 金币/分钟：判断发育效率
/// - 发育曲线：判断打法风格（对线型/游走型）
use crate::shared::types::SummonerTrait;

/// 时间线特征分析
///
/// 输入：解析后的对局数据（含时间线）
/// 输出：时间线相关的特征标签
pub fn analyze_timeline_traits(games: &[ParsedGame], role: &str) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    // 数据量不足，不分析
    if games.len() < 5 {
        return traits;
    }

    // 1. 对线期分析（0-10分钟）⭐ 核心
    traits.extend(analyze_laning_phase(games, role));

    // 2. 发育曲线分析（对比各阶段）
    traits.extend(analyze_growth_curve(games));

    // 3. 等级优势分析
    traits.extend(analyze_level_advantage(games));

    traits
}

/// 对线期分析（0-10分钟）
///
/// 核心指标：CS差（补刀差）
/// - CS差 > +15：对线压制
/// - CS差 -5~+5：均势对线
/// - CS差 < -15：对线弱势
fn analyze_laning_phase(games: &[ParsedGame], role: &str) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    let mut total_cs_diff = 0.0;
    let mut valid_games = 0;

    // 统计对线期CS差
    for game in games {
        if let Some(timeline) = &game.player_data.timeline_data {
            if let Some(cs_diff) = timeline.cs_diff_0_10 {
                total_cs_diff += cs_diff;
                valid_games += 1;
            }
        }
    }

    // 数据不足，不分析
    if valid_games < 5 {
        return traits;
    }

    let avg_cs_diff = total_cs_diff / valid_games as f64;

    // 使用 thresholds 判断对线表现 ⭐
    if avg_cs_diff >= thresholds::laning_phase::CS_DIFF_DOMINATE {
        // 对线压制
        traits.push(SummonerTrait {
            name: "对线压制".to_string(),
            description: format!("前10分钟平均领先{:.1}刀，对线压制力强", avg_cs_diff),
            score: avg_cs_diff as i32,
            trait_type: "good".to_string(),
        });
    } else if avg_cs_diff >= thresholds::laning_phase::CS_DIFF_ADVANTAGE {
        // 对线优势
        traits.push(SummonerTrait {
            name: "对线优势".to_string(),
            description: format!("前10分钟平均领先{:.1}刀，对线有优势", avg_cs_diff),
            score: avg_cs_diff as i32,
            trait_type: "good".to_string(),
        });
    } else if avg_cs_diff >= thresholds::laning_phase::CS_DIFF_NEUTRAL_LOW
        && avg_cs_diff <= thresholds::laning_phase::CS_DIFF_NEUTRAL_HIGH
    {
        // 均势对线（一般不显示，除非特别稳定）
        if valid_games >= 10 {
            traits.push(SummonerTrait {
                name: "稳健对线".to_string(),
                description: format!("前10分钟补刀均势（{:+.1}刀），对线稳健", avg_cs_diff),
                score: 50,
                trait_type: "good".to_string(),
            });
        }
    } else if avg_cs_diff <= thresholds::laning_phase::CS_DIFF_SUPPRESSED {
        // 对线弱势
        traits.push(SummonerTrait {
            name: "对线弱势".to_string(),
            description: format!("前10分钟平均落后{:.1}刀，对线承压", -avg_cs_diff),
            score: (-avg_cs_diff) as i32,
            trait_type: "bad".to_string(),
        });
    } else if avg_cs_diff <= thresholds::laning_phase::CS_DIFF_DISADVANTAGE {
        // 对线劣势
        traits.push(SummonerTrait {
            name: "对线劣势".to_string(),
            description: format!("前10分钟平均落后{:.1}刀，对线有压力", -avg_cs_diff),
            score: (-avg_cs_diff) as i32,
            trait_type: "bad".to_string(),
        });
    }

    traits
}

/// 发育曲线分析
///
/// 对比对线期和中期的经济效率
/// - 中期 > 对线期 * 1.15：爆发成长（游走型打法）
/// - 各阶段稳定 > 400：稳定发育
/// - 中期下降：发育节奏问题
fn analyze_growth_curve(games: &[ParsedGame]) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    let mut early_gold_sum = 0.0;
    let mut mid_gold_sum = 0.0;
    let mut late_gold_sum = 0.0;
    let mut valid_games = 0;

    // 统计各阶段金币效率
    for game in games {
        if let Some(timeline) = &game.player_data.timeline_data {
            if let (Some(early), Some(mid)) = (timeline.gold_per_min_0_10, timeline.gold_per_min_10_20) {
                early_gold_sum += early;
                mid_gold_sum += mid;
                if let Some(late) = timeline.gold_per_min_20_end {
                    late_gold_sum += late;
                }
                valid_games += 1;
            }
        }
    }

    if valid_games < 5 {
        return traits;
    }

    let avg_early_gold = early_gold_sum / valid_games as f64;
    let avg_mid_gold = mid_gold_sum / valid_games as f64;

    // 使用 thresholds 判断发育曲线 ⭐
    // 爆发成长（中期经济提升显著）
    if avg_mid_gold > avg_early_gold * thresholds::growth::MID_GAME_BOOST {
        let growth_rate = (avg_mid_gold / avg_early_gold - 1.0) * 100.0;
        traits.push(SummonerTrait {
            name: "爆发成长".to_string(),
            description: format!(
                "中期经济效率提升{:.0}%（{}→{}），游走支援能力强",
                growth_rate, avg_early_gold as i32, avg_mid_gold as i32
            ),
            score: 70,
            trait_type: "good".to_string(),
        });
    }
    // 稳定发育（各阶段经济都不错）
    else if avg_early_gold >= thresholds::growth::STABLE_GOLD_EARLY
        && avg_mid_gold >= thresholds::growth::STABLE_GOLD_MID
    {
        traits.push(SummonerTrait {
            name: "稳定发育".to_string(),
            description: format!(
                "各阶段经济稳定（{:.0}/{:.0}），发育能力强",
                avg_early_gold, avg_mid_gold
            ),
            score: 65,
            trait_type: "good".to_string(),
        });
    }
    // 中期乏力（经济下降）
    else if avg_mid_gold < avg_early_gold * thresholds::growth::MID_GAME_DECLINE {
        let decline_rate = (1.0 - avg_mid_gold / avg_early_gold) * 100.0;
        traits.push(SummonerTrait {
            name: "中期乏力".to_string(),
            description: format!(
                "中期经济效率下降{:.0}%（{}→{}），发育节奏需优化",
                decline_rate, avg_early_gold as i32, avg_mid_gold as i32
            ),
            score: decline_rate as i32,
            trait_type: "bad".to_string(),
        });
    }

    traits
}

/// 等级优势分析
///
/// 基于经验差判断抢等级能力
fn analyze_level_advantage(games: &[ParsedGame]) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    let mut total_xp_diff = 0.0;
    let mut valid_games = 0;

    // 统计对线期经验差
    for game in games {
        if let Some(timeline) = &game.player_data.timeline_data {
            if let Some(xp_diff) = timeline.xp_diff_0_10 {
                total_xp_diff += xp_diff;
                valid_games += 1;
            }
        }
    }

    if valid_games < 5 {
        return traits;
    }

    let avg_xp_diff = total_xp_diff / valid_games as f64;

    // 使用 thresholds 判断等级差 ⭐
    // 等级优势
    if avg_xp_diff >= thresholds::laning_phase::XP_DIFF_ADVANTAGE {
        traits.push(SummonerTrait {
            name: "等级优势".to_string(),
            description: format!("前10分钟平均经验领先{:.0}，抢等级能力强", avg_xp_diff),
            score: 60,
            trait_type: "good".to_string(),
        });
    }
    // 等级劣势
    else if avg_xp_diff <= thresholds::laning_phase::XP_DIFF_DISADVANTAGE {
        traits.push(SummonerTrait {
            name: "等级劣势".to_string(),
            description: format!("前10分钟平均经验落后{:.0}，对线期被压制", -avg_xp_diff),
            score: 50,
            trait_type: "bad".to_string(),
        });
    }

    traits
}

/// 计算平均时间线指标（辅助函数）
#[allow(dead_code)]
fn calculate_avg_timeline_metric<F>(games: &[ParsedGame], extractor: F) -> Option<f64>
where
    F: Fn(&TimelineData) -> Option<f64>,
{
    let mut sum = 0.0;
    let mut count = 0;

    for game in games {
        if let Some(timeline) = &game.player_data.timeline_data {
            if let Some(value) = extractor(timeline) {
                sum += value;
                count += 1;
            }
        }
    }

    if count == 0 {
        None
    } else {
        Some(sum / count as f64)
    }
}
