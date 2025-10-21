use crate::domains::analysis::thresholds;
use crate::shared::types::{MatchPerformance, PlayerMatchStats, SummonerTrait};

/// 分析召唤师特征标签
///
/// 根据战绩统计数据，生成定性分析标签
pub fn analyze_traits(stats: &PlayerMatchStats) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    // 如果数据量太少，不进行分析
    if stats.total_games < 3 {
        return traits;
    }

    // 1. 胜率特征
    analyze_win_rate_traits(stats, &mut traits);

    // 2. KDA特征
    analyze_kda_traits(stats, &mut traits);

    // 3. 击杀/助攻特征
    analyze_kills_assists_traits(stats, &mut traits);

    // 4. 连胜/连败特征
    analyze_streak_traits(stats, &mut traits);

    // 5. 综合评分特征
    analyze_overall_traits(stats, &mut traits);

    // 按得分排序（从高到低）
    traits.sort_by(|a, b| b.score.cmp(&a.score));

    traits
}

/// 胜率特征分析
fn analyze_win_rate_traits(stats: &PlayerMatchStats, traits: &mut Vec<SummonerTrait>) {
    // 使用排位标准作为"大神"基准，其他模式标准作为"稳定"基准
    if stats.win_rate >= thresholds::win_rate::EXCELLENT_RANKED {
        traits.push(SummonerTrait {
            name: "大神".to_string(),
            description: format!("胜率超高的实力玩家（{}%）", stats.win_rate as i32),
            score: stats.win_rate as i32,
            trait_type: "good".to_string(),
        });
    } else if stats.win_rate >= thresholds::win_rate::GOOD {
        traits.push(SummonerTrait {
            name: "稳定".to_string(),
            description: format!("胜率稳定的可靠队友（{}%）", stats.win_rate as i32),
            score: stats.win_rate as i32,
            trait_type: "good".to_string(),
        });
    } else if stats.win_rate <= thresholds::win_rate::POOR && stats.total_games >= 10 {
        traits.push(SummonerTrait {
            name: "坑货".to_string(),
            description: format!("胜率偏低的玩家（{}%）", stats.win_rate as i32),
            score: stats.win_rate as i32,
            trait_type: "bad".to_string(),
        });
    }
}

/// KDA特征分析
fn analyze_kda_traits(stats: &PlayerMatchStats, traits: &mut Vec<SummonerTrait>) {
    if stats.avg_kda >= thresholds::kda::EXCELLENT_RANKED {
        traits.push(SummonerTrait {
            name: "大爹".to_string(),
            description: format!("KDA超高的carry玩家（{:.1}）", stats.avg_kda),
            score: (stats.avg_kda * 10.0) as i32,
            trait_type: "good".to_string(),
        });
    } else if stats.avg_kda <= thresholds::kda::POOR && stats.avg_deaths >= 7.0 {
        traits.push(SummonerTrait {
            name: "送分".to_string(),
            description: format!("KDA偏低的玩家（{:.1}）", stats.avg_kda),
            score: stats.avg_deaths as i32,
            trait_type: "bad".to_string(),
        });
    }
}

/// 击杀/助攻特征分析
fn analyze_kills_assists_traits(stats: &PlayerMatchStats, traits: &mut Vec<SummonerTrait>) {
    if stats.avg_kills >= 8.0 {
        traits.push(SummonerTrait {
            name: "人头狗".to_string(),
            description: "击杀能力超强的玩家".to_string(),
            score: stats.avg_kills as i32,
            trait_type: "good".to_string(),
        });
    }

    if stats.avg_assists >= 10.0 {
        traits.push(SummonerTrait {
            name: "辅助王".to_string(),
            description: "助攻能力超强的团队玩家".to_string(),
            score: stats.avg_assists as i32,
            trait_type: "good".to_string(),
        });
    }
}

/// 连胜/连败特征分析
fn analyze_streak_traits(stats: &PlayerMatchStats, traits: &mut Vec<SummonerTrait>) {
    if stats.recent_performance.is_empty() {
        return;
    }

    let win_streak = calculate_win_streak(&stats.recent_performance);

    if win_streak >= thresholds::streak::WIN_STREAK_GOOD {
        traits.push(SummonerTrait {
            name: "连胜王".to_string(),
            description: format!("正在{}连胜的火热玩家", win_streak),
            score: win_streak,
            trait_type: "good".to_string(),
        });
    } else if win_streak <= thresholds::streak::LOSS_STREAK_BAD {
        traits.push(SummonerTrait {
            name: "连败".to_string(),
            description: format!("近期{}连败状态不佳", win_streak.abs()),
            score: win_streak.abs(),
            trait_type: "bad".to_string(),
        });
    }
}

/// 综合评分特征分析
fn analyze_overall_traits(stats: &PlayerMatchStats, traits: &mut Vec<SummonerTrait>) {
    let overall_score = calculate_overall_score(stats);

    if overall_score >= 80 {
        traits.push(SummonerTrait {
            name: "全能王".to_string(),
            description: "综合能力极强的玩家".to_string(),
            score: overall_score,
            trait_type: "good".to_string(),
        });
    }
}

/// 计算连胜/连败
///
/// 返回值：
/// - 正数表示连胜
/// - 负数表示连败
fn calculate_win_streak(recent_games: &[MatchPerformance]) -> i32 {
    let mut streak = 0;

    for game in recent_games {
        if game.win {
            if streak >= 0 {
                streak += 1;
            } else {
                break;
            }
        } else {
            if streak <= 0 {
                streak -= 1;
            } else {
                break;
            }
        }
    }

    streak
}

/// 计算综合评分
///
/// 综合考虑：
/// - 胜率（权重 60%）
/// - KDA（权重 20%）
/// - 游戏场次（权重 20%）
fn calculate_overall_score(stats: &PlayerMatchStats) -> i32 {
    let win_rate_score = (stats.win_rate * 0.6).min(60.0);
    let kda_score = (stats.avg_kda * 5.0).min(20.0);
    let games_score = (stats.total_games as f64 * 0.2).min(20.0);

    (win_rate_score + kda_score + games_score) as i32
}
