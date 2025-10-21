use crate::domains::analysis::thresholds;
use crate::shared::types::{MatchPerformance, SummonerTrait};

/// 表现分布特征分析
pub fn analyze_distribution_traits(games: &[MatchPerformance]) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    if games.len() < 5 {
        return traits;
    }

    let mut s_count = 0;
    let mut a_count = 0;
    let mut d_count = 0;
    let mut c_count = 0;

    for game in games {
        if game.kda >= thresholds::kda::S_GRADE {
            s_count += 1;
        } else if game.kda >= thresholds::kda::A_GRADE {
            a_count += 1;
        } else if game.kda < thresholds::kda::D_GRADE {
            d_count += 1;
        } else if game.kda >= thresholds::kda::D_GRADE && game.kda < thresholds::kda::B_GRADE {
            c_count += 1;
        }
    }

    let total = games.len();
    let excellent_rate = (s_count + a_count) as f64 / total as f64;
    let poor_rate = d_count as f64 / total as f64;

    if s_count >= 3 {
        traits.push(SummonerTrait {
            name: "高光时刻".to_string(),
            description: format!(
                "{}场中有{}场超神表现（KDA>{:.1}）",
                total,
                s_count,
                thresholds::kda::S_GRADE
            ),
            score: s_count,
            trait_type: "good".to_string(),
        });
    }

    if excellent_rate >= 0.50 {
        traits.push(SummonerTrait {
            name: "稳定优秀".to_string(),
            description: format!(
                "{:.0}%的对局达到优秀水平（KDA>{:.1}）",
                excellent_rate * 100.0,
                thresholds::kda::A_GRADE
            ),
            score: (excellent_rate * 100.0) as i32,
            trait_type: "good".to_string(),
        });
    }

    if d_count >= 3 && poor_rate >= 0.15 {
        traits.push(SummonerTrait {
            name: "偶尔崩盘".to_string(),
            description: format!(
                "{}场中有{}场崩盘表现（KDA<{:.1}）",
                total,
                d_count,
                thresholds::kda::D_GRADE
            ),
            score: d_count,
            trait_type: "bad".to_string(),
        });
    }

    if s_count >= 3 && d_count >= 3 {
        traits.push(SummonerTrait {
            name: "两极分化".to_string(),
            description: format!("既有{}场超神，也有{}场崩盘，表现不稳定", s_count, d_count),
            score: s_count + d_count,
            trait_type: "bad".to_string(),
        });
    }

    traits
}

/// 胜负模式分析
pub fn analyze_win_loss_pattern(games: &[MatchPerformance]) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    if games.len() < 10 {
        return traits;
    }

    let recent_5 = &games[..games.len().min(5)];
    let recent_10 = &games[..games.len().min(10)];

    let wins_5 = recent_5.iter().filter(|g| g.win).count();
    let wins_10 = recent_10.iter().filter(|g| g.win).count();

    if wins_5 >= 4 {
        traits.push(SummonerTrait {
            name: "近期火热".to_string(),
            description: format!("最近5场{}胜，状态极佳", wins_5),
            score: wins_5 as i32,
            trait_type: "good".to_string(),
        });
    }

    if wins_5 <= 1 {
        traits.push(SummonerTrait {
            name: "近期低迷".to_string(),
            description: format!("最近5场仅{}胜，状态不佳", wins_5),
            score: (5 - wins_5) as i32,
            trait_type: "bad".to_string(),
        });
    }

    if wins_10 >= 8 {
        traits.push(SummonerTrait {
            name: "近期强势".to_string(),
            description: format!("最近10场{}胜，保持强势", wins_10),
            score: wins_10 as i32,
            trait_type: "good".to_string(),
        });
    } else if wins_10 <= 3 {
        traits.push(SummonerTrait {
            name: "近期挣扎".to_string(),
            description: format!("最近10场仅{}胜，陷入低谷", wins_10),
            score: (10 - wins_10) as i32,
            trait_type: "bad".to_string(),
        });
    }

    traits
}
