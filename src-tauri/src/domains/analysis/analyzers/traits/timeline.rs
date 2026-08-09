/// 时间线特征分析器 - 基于时间线数据生成玩家特征
///
/// 职责：
/// - 分析时间线数据生成特征标签
/// - 识别对线期表现模式
/// - 分析发育期效率
/// - 评估后期表现
/// - 生成时间线相关的建议
use crate::domains::analysis::analyzers::core::timeline_analyzer::TimelineAnalysis;
use crate::shared::types::SummonerTrait;

/// 分析时间线特征
pub fn analyze_timeline_traits(
    games: &[crate::domains::analysis::analyzers::core::parser::ParsedGame],
    main_role: &str,
) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    // 分析每场游戏的时间线数据
    for game in games {
        if let Some(ref timeline_data) = game.player_data.timeline_data {
            traits.extend(analyze_single_game_timeline(timeline_data, main_role));
        }
    }

    // 聚合多场游戏的特征
    aggregate_timeline_traits(&traits)
}

/// 分析单场游戏的时间线特征
fn analyze_single_game_timeline(
    timeline_data: &crate::domains::analysis::analyzers::core::parser::TimelineData,
    _main_role: &str,
) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    // 对线期分析 (0-10分钟)
    if let Some(cs_diff) = timeline_data.cs_diff_0_10 {
        if cs_diff > 15.0 {
            traits.push(SummonerTrait {
                name: "对线压制".to_string(),
                description: "对线期补刀优势明显".to_string(),
                score: 8,
                trait_type: "good".to_string(),
            });
        } else if cs_diff < -20.0 {
            traits.push(SummonerTrait {
                name: "对线劣势".to_string(),
                description: "对线期补刀被压制".to_string(),
                score: 6,
                trait_type: "bad".to_string(),
            });
        } else if cs_diff > -5.0 && cs_diff < 5.0 {
            traits.push(SummonerTrait {
                name: "稳健对线".to_string(),
                description: "对线期表现稳定".to_string(),
                score: 7,
                trait_type: "good".to_string(),
            });
        }
    }

    // 发育期分析 (10-20分钟)
    if let Some(cs_per_min) = timeline_data.cs_per_min_10_20 {
        if cs_per_min > 8.0 {
            traits.push(SummonerTrait {
                name: "高效发育".to_string(),
                description: "中期补刀效率高".to_string(),
                score: 8,
                trait_type: "good".to_string(),
            });
        } else if cs_per_min < 5.0 {
            traits.push(SummonerTrait {
                name: "发育缓慢".to_string(),
                description: "中期补刀效率偏低".to_string(),
                score: 5,
                trait_type: "bad".to_string(),
            });
        }
    }

    // 金币效率分析
    if let Some(gold_0_10) = timeline_data.gold_per_min_0_10 {
        if let Some(gold_10_20) = timeline_data.gold_per_min_10_20 {
            if gold_10_20 > gold_0_10 * 1.2 {
                traits.push(SummonerTrait {
                    name: "爆发成长".to_string(),
                    description: "中期金币获取效率提升".to_string(),
                    score: 7,
                    trait_type: "good".to_string(),
                });
            }
        }
    }

    // 后期分析 (20分钟+)
    if let Some(cs_20_end) = timeline_data.cs_per_min_20_end {
        if cs_20_end < 6.0 {
            traits.push(SummonerTrait {
                name: "后期乏力".to_string(),
                description: "后期补刀效率下降".to_string(),
                score: 4,
                trait_type: "bad".to_string(),
            });
        }
    }

    // 承伤分析
    if let Some(damage_taken) = timeline_data.damage_taken_per_min_0_10 {
        if damage_taken > 800.0 {
            traits.push(SummonerTrait {
                name: "激进打法".to_string(),
                description: "对线期承伤较高".to_string(),
                score: 6,
                trait_type: "bad".to_string(),
            });
        } else if damage_taken < 400.0 {
            traits.push(SummonerTrait {
                name: "保守打法".to_string(),
                description: "对线期承伤较低".to_string(),
                score: 7,
                trait_type: "good".to_string(),
            });
        }
    }

    traits
}

/// 聚合多场游戏的时间线特征
fn aggregate_timeline_traits(traits: &[SummonerTrait]) -> Vec<SummonerTrait> {
    let mut trait_counts: std::collections::HashMap<String, (SummonerTrait, usize)> = std::collections::HashMap::new();

    for trait_item in traits {
        let key = trait_item.name.clone();
        let entry = trait_counts.entry(key).or_insert_with(|| (trait_item.clone(), 0));
        entry.1 += 1;
    }

    // 转换为Vec并按频率排序
    let mut result: Vec<SummonerTrait> = trait_counts.into_values().map(|(trait_item, _)| trait_item).collect();

    result.sort_by(|a, b| b.score.cmp(&a.score));
    result
}

/// 基于时间线分析生成建议
pub fn generate_timeline_advice(timeline_analysis: &TimelineAnalysis, _main_role: &str) -> Vec<String> {
    let mut advice = Vec::new();

    // 对线期建议
    if timeline_analysis.early_game.cs_difference < -10.0 {
        advice.push("对线期补刀被压制，建议加强补刀练习".to_string());
    } else if timeline_analysis.early_game.cs_difference > 10.0 {
        advice.push("对线期补刀优势明显，继续保持压制".to_string());
    }

    // 发育期建议
    if timeline_analysis.mid_game.cs_per_minute < 6.0 {
        advice.push("中期补刀效率偏低，注意发育节奏".to_string());
    }

    // 后期建议
    if timeline_analysis.late_game.cs_per_minute < timeline_analysis.mid_game.cs_per_minute {
        advice.push("后期补刀效率下降，注意保持发育".to_string());
    }

    // 对手比较建议
    if timeline_analysis.opponent_comparison.overall_advantage < -5.0 {
        advice.push("整体表现不如对手，需要提升对线技巧".to_string());
    } else if timeline_analysis.opponent_comparison.overall_advantage > 5.0 {
        advice.push("整体表现优于对手，保持当前状态".to_string());
    }

    advice
}
