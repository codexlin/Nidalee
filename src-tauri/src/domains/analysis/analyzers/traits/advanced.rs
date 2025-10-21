use crate::shared::types::{MatchPerformance, PlayerMatchStats, SummonerTrait};
use crate::domains::analysis::thresholds;
use serde_json::Value;

/// 深度特征分析（基于相对数据和趋势）
///
/// 这些分析需要聚合队伍数据，能提供更深入的洞察
pub fn analyze_advanced_traits(
    stats: &PlayerMatchStats,
    games: &[Value],
    puuid: &str,
) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    // 1. 参团率分析
    if let Some(kp_trait) = analyze_kill_participation(games, puuid) {
        traits.push(kp_trait);
    }

    // 2. 伤害占比分析
    if let Some(damage_trait) = analyze_damage_share(games, puuid) {
        traits.push(damage_trait);
    }

    // 3. 承伤占比分析（坦克/前排）
    if let Some(tank_trait) = analyze_tank_share(games, puuid) {
        traits.push(tank_trait);
    }

    // 4. 稳定性分析
    if let Some(stability_trait) = analyze_stability(&stats.recent_performance) {
        traits.push(stability_trait);
    }

    // 5. 趋势分析
    if let Some(trend_trait) = analyze_performance_trend(&stats.recent_performance) {
        traits.push(trend_trait);
    }

    // 6. 视野控制分析
    traits.extend(analyze_vision_control(stats, games, puuid));

    // 7. 资源控制分析
    traits.extend(analyze_objective_control(games, puuid));

    // 8. 英雄熟练度分析
    traits.extend(analyze_champion_mastery(stats));

    traits
}

/// 1. 参团率分析 (Kill Participation %)
fn analyze_kill_participation(games: &[Value], puuid: &str) -> Option<SummonerTrait> {
    let mut total_kp = 0.0;
    let mut valid_games = 0;

    for game in games {
        if let Some(kp) = calculate_kp_for_game(game, puuid) {
            total_kp += kp;
            valid_games += 1;
        }
    }

    if valid_games == 0 {
        return None;
    }

    let avg_kp = total_kp / valid_games as f64;

    if avg_kp >= thresholds::kill_participation::HIGH {
        Some(SummonerTrait {
            name: "团战核心".to_string(),
            description: format!("参与了{:.0}%的击杀，团战参与度极高", avg_kp * 100.0),
            score: (avg_kp * 100.0) as i32,
            trait_type: "good".to_string(),
        })
    } else if avg_kp >= 0.60 {  // 中等参团率阈值
        Some(SummonerTrait {
            name: "积极参团".to_string(),
            description: format!("参团率{:.0}%，团战意识不错", avg_kp * 100.0),
            score: (avg_kp * 100.0) as i32,
            trait_type: "good".to_string(),
        })
    } else if avg_kp <= thresholds::kill_participation::LOW {
        Some(SummonerTrait {
            name: "游离".to_string(),
            description: format!("参团率仅{:.0}%，很少参与团战", avg_kp * 100.0),
            score: (avg_kp * 100.0) as i32,
            trait_type: "bad".to_string(),
        })
    } else {
        None
    }
}

fn calculate_kp_for_game(game: &Value, puuid: &str) -> Option<f64> {
    let participant_id = game["participantIdentities"]
        .as_array()?
        .iter()
        .find(|id| id["player"]["puuid"].as_str() == Some(puuid))?
        ["participantId"]
        .as_i64()?;

    let participant = game["participants"]
        .as_array()?
        .iter()
        .find(|p| p["participantId"].as_i64() == Some(participant_id))?;

    let team_id = participant["teamId"].as_i64()?;
    let player_kills = participant["stats"]["kills"].as_i64().unwrap_or(0);
    let player_assists = participant["stats"]["assists"].as_i64().unwrap_or(0);

    let team_total_kills: i64 = game["participants"]
        .as_array()?
        .iter()
        .filter(|p| p["teamId"].as_i64() == Some(team_id))
        .filter_map(|p| p["stats"]["kills"].as_i64())
        .sum();

    if team_total_kills == 0 {
        return Some(0.0);
    }

    let kp = (player_kills + player_assists) as f64 / team_total_kills as f64;
    // 确保参团率不超过100%
    Some(kp.min(1.0))
}

/// 2. 伤害占比分析
fn analyze_damage_share(games: &[Value], puuid: &str) -> Option<SummonerTrait> {
    let mut total_damage_share = 0.0;
    let mut valid_games = 0;

    for game in games {
        if let Some(share) = calculate_damage_share_for_game(game, puuid) {
            total_damage_share += share;
            valid_games += 1;
        }
    }

    if valid_games == 0 {
        return None;
    }

    let avg_damage_share = total_damage_share / valid_games as f64;

    if avg_damage_share >= 0.30 {
        Some(SummonerTrait {
            name: "输出核心".to_string(),
            description: format!("承担{:.0}%的队伍伤害，是主要输出", avg_damage_share * 100.0),
            score: (avg_damage_share * 100.0) as i32,
            trait_type: "good".to_string(),
        })
    } else if avg_damage_share >= 0.25 {
        Some(SummonerTrait {
            name: "主要输出".to_string(),
            description: format!("伤害占比{:.0}%，输出能力不错", avg_damage_share * 100.0),
            score: (avg_damage_share * 100.0) as i32,
            trait_type: "good".to_string(),
        })
    } else if avg_damage_share <= 0.12 {
        Some(SummonerTrait {
            name: "输出不足".to_string(),
            description: format!("伤害占比仅{:.0}%，输出贡献低", avg_damage_share * 100.0),
            score: (avg_damage_share * 100.0) as i32,
            trait_type: "bad".to_string(),
        })
    } else {
        None
    }
}

fn calculate_damage_share_for_game(game: &Value, puuid: &str) -> Option<f64> {
    let participant_id = game["participantIdentities"]
        .as_array()?
        .iter()
        .find(|id| id["player"]["puuid"].as_str() == Some(puuid))?
        ["participantId"]
        .as_i64()?;

    let participant = game["participants"]
        .as_array()?
        .iter()
        .find(|p| p["participantId"].as_i64() == Some(participant_id))?;

    let team_id = participant["teamId"].as_i64()?;
    let player_damage = participant["stats"]["totalDamageDealtToChampions"]
        .as_i64()
        .unwrap_or(0);

    let team_total_damage: i64 = game["participants"]
        .as_array()?
        .iter()
        .filter(|p| p["teamId"].as_i64() == Some(team_id))
        .filter_map(|p| p["stats"]["totalDamageDealtToChampions"].as_i64())
        .sum();

    if team_total_damage == 0 {
        return Some(0.0);
    }

    Some(player_damage as f64 / team_total_damage as f64)
}

/// 3. 承伤占比分析
fn analyze_tank_share(games: &[Value], puuid: &str) -> Option<SummonerTrait> {
    let mut total_tank_share = 0.0;
    let mut valid_games = 0;

    for game in games {
        if let Some(share) = calculate_tank_share_for_game(game, puuid) {
            total_tank_share += share;
            valid_games += 1;
        }
    }

    if valid_games == 0 {
        return None;
    }

    let avg_tank_share = total_tank_share / valid_games as f64;

    if avg_tank_share >= 0.28 {
        Some(SummonerTrait {
            name: "前排坦克".to_string(),
            description: format!("承担{:.0}%的队伍承伤，是主要坦克", avg_tank_share * 100.0),
            score: (avg_tank_share * 100.0) as i32,
            trait_type: "good".to_string(),
        })
    } else if avg_tank_share >= 0.22 {
        Some(SummonerTrait {
            name: "半肉".to_string(),
            description: format!("承伤占比{:.0}%，有一定坦度", avg_tank_share * 100.0),
            score: (avg_tank_share * 100.0) as i32,
            trait_type: "good".to_string(),
        })
    } else {
        None
    }
}

fn calculate_tank_share_for_game(game: &Value, puuid: &str) -> Option<f64> {
    let participant_id = game["participantIdentities"]
        .as_array()?
        .iter()
        .find(|id| id["player"]["puuid"].as_str() == Some(puuid))?
        ["participantId"]
        .as_i64()?;

    let participant = game["participants"]
        .as_array()?
        .iter()
        .find(|p| p["participantId"].as_i64() == Some(participant_id))?;

    let team_id = participant["teamId"].as_i64()?;
    let player_tank = participant["stats"]["totalDamageTaken"]
        .as_i64()
        .unwrap_or(0);

    let team_total_tank: i64 = game["participants"]
        .as_array()?
        .iter()
        .filter(|p| p["teamId"].as_i64() == Some(team_id))
        .filter_map(|p| p["stats"]["totalDamageTaken"].as_i64())
        .sum();

    if team_total_tank == 0 {
        return Some(0.0);
    }

    Some(player_tank as f64 / team_total_tank as f64)
}

/// 4. 稳定性分析（KDA 波动性）
fn analyze_stability(recent_games: &[MatchPerformance]) -> Option<SummonerTrait> {
    if recent_games.len() < 5 {
        return None;
    }

    let kdas: Vec<f64> = recent_games.iter().map(|g| g.kda).collect();
    let mean_kda = kdas.iter().sum::<f64>() / kdas.len() as f64;

    if mean_kda < 1.0 {
        return None;
    }

    let variance = kdas
        .iter()
        .map(|kda| (kda - mean_kda).powi(2))
        .sum::<f64>()
        / kdas.len() as f64;
    let std_dev = variance.sqrt();
    let cv = std_dev / mean_kda;

    if cv < 0.4 && mean_kda >= 3.0 {
        Some(SummonerTrait {
            name: "稳如老狗".to_string(),
            description: format!("KDA波动极小，发挥稳定（CV={:.2}）", cv),
            score: (mean_kda * 10.0) as i32,
            trait_type: "good".to_string(),
        })
    } else if cv > 1.2 {
        Some(SummonerTrait {
            name: "神经刀".to_string(),
            description: format!("KDA波动大，发挥不稳定（CV={:.2}）", cv),
            score: (cv * 10.0) as i32,
            trait_type: "bad".to_string(),
        })
    } else {
        None
    }
}

/// 5. 趋势分析（状态上升/下滑）
fn analyze_performance_trend(recent_games: &[MatchPerformance]) -> Option<SummonerTrait> {
    if recent_games.len() < 10 {
        return None;
    }

    let mid = recent_games.len() / 2;
    let recent_half = &recent_games[..mid];
    let older_half = &recent_games[mid..];

    let recent_kda: f64 = recent_half.iter().map(|g| g.kda).sum::<f64>() / recent_half.len() as f64;
    let older_kda: f64 = older_half.iter().map(|g| g.kda).sum::<f64>() / older_half.len() as f64;

    let recent_wr =
        recent_half.iter().filter(|g| g.win).count() as f64 / recent_half.len() as f64;
    let older_wr = older_half.iter().filter(|g| g.win).count() as f64 / older_half.len() as f64;

    let kda_change_rate = if older_kda > 0.0 {
        (recent_kda - older_kda) / older_kda
    } else {
        0.0
    };
    let wr_change = recent_wr - older_wr;

    if kda_change_rate > 0.3 && wr_change > 0.15 {
        Some(SummonerTrait {
            name: "状态上升".to_string(),
            description: format!(
                "最近表现明显提升（KDA↑{:.0}%, 胜率↑{:.0}%）",
                (kda_change_rate * 100.0).min(200.0), // 限制最大200%
                wr_change * 100.0
            ),
            score: (kda_change_rate * 100.0).min(200.0) as i32,
            trait_type: "good".to_string(),
        })
    } else if kda_change_rate < -0.3 && wr_change < -0.15 {
        Some(SummonerTrait {
            name: "状态下滑".to_string(),
            description: format!(
                "最近表现下降（KDA↓{:.0}%, 胜率↓{:.0}%）",
                (kda_change_rate.abs() * 100.0).min(200.0), // 限制最大200%
                wr_change.abs() * 100.0
            ),
            score: (kda_change_rate.abs() * 100.0).min(200.0) as i32,
            trait_type: "bad".to_string(),
        })
    } else {
        None
    }
}

/// 6. 视野控制分析
fn analyze_vision_control(
    stats: &PlayerMatchStats,
    games: &[Value],
    puuid: &str,
) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    if stats.vspm >= 2.0 {
        traits.push(SummonerTrait {
            name: "视野大师".to_string(),
            description: format!("每分钟视野得分{:.1}，视野控制能力强", stats.vspm),
            score: (stats.vspm * 10.0) as i32,
            trait_type: "good".to_string(),
        });
    }

    if let Some(avg_wards_killed) = calculate_avg_wards_killed(games, puuid) {
        if avg_wards_killed >= 10.0 {
            traits.push(SummonerTrait {
                name: "排眼狂魔".to_string(),
                description: format!("场均排眼{:.0}个，压制对方视野", avg_wards_killed),
                score: avg_wards_killed as i32,
                trait_type: "good".to_string(),
            });
        }
    }

    traits
}

fn calculate_avg_wards_killed(games: &[Value], puuid: &str) -> Option<f64> {
    let mut total_wards_killed = 0.0;
    let mut valid_games = 0;

    for game in games {
        if let Some(participant) = find_participant(game, puuid) {
            let wards = participant["stats"]["wardsKilled"].as_i64().unwrap_or(0);
            total_wards_killed += wards as f64;
            valid_games += 1;
        }
    }

    if valid_games == 0 {
        return None;
    }

    Some(total_wards_killed / valid_games as f64)
}

/// 7. 资源控制分析
fn analyze_objective_control(games: &[Value], puuid: &str) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    let mut total_obj_damage = 0.0;
    let mut total_turret_damage = 0.0;
    let mut valid_games = 0;

    for game in games {
        if let Some(participant) = find_participant(game, puuid) {
            total_obj_damage += participant["stats"]["damageDealtToObjectives"]
                .as_i64()
                .unwrap_or(0) as f64;
            total_turret_damage += participant["stats"]["damageDealtToTurrets"]
                .as_i64()
                .unwrap_or(0) as f64;
            valid_games += 1;
        }
    }

    if valid_games == 0 {
        return traits;
    }

    let avg_obj_damage = total_obj_damage / valid_games as f64;
    let avg_turret_damage = total_turret_damage / valid_games as f64;

    if avg_obj_damage >= 8000.0 {
        traits.push(SummonerTrait {
            name: "控龙高手".to_string(),
            description: format!("场均对大龙/小龙造成{:.0}伤害", avg_obj_damage),
            score: (avg_obj_damage / 100.0) as i32,
            trait_type: "good".to_string(),
        });
    }

    if avg_turret_damage >= 5000.0 {
        traits.push(SummonerTrait {
            name: "推塔狂魔".to_string(),
            description: format!("场均推塔伤害{:.0}", avg_turret_damage),
            score: (avg_turret_damage / 100.0) as i32,
            trait_type: "good".to_string(),
        });
    }

    traits
}

/// 8. 英雄熟练度分析
fn analyze_champion_mastery(stats: &PlayerMatchStats) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    if stats.favorite_champions.is_empty() {
        return traits;
    }

    let top_champion = &stats.favorite_champions[0];
    let total_games = stats.total_games;
    let specialization = top_champion.games as f64 / total_games as f64;

    if specialization >= 0.5 && top_champion.win_rate >= 60.0 {
        traits.push(SummonerTrait {
            name: "绝活哥".to_string(),
            description: format!(
                "专精某英雄（{:.0}%场次），胜率{:.0}%",
                specialization * 100.0,
                top_champion.win_rate
            ),
            score: top_champion.win_rate as i32,
            trait_type: "good".to_string(),
        });
    } else if specialization >= 0.7 && top_champion.win_rate < 50.0 {
        traits.push(SummonerTrait {
            name: "单一依赖".to_string(),
            description: format!(
                "英雄池窄（{:.0}%打同一英雄）且胜率不高",
                specialization * 100.0
            ),
            score: (specialization * 100.0) as i32,
            trait_type: "bad".to_string(),
        });
    }

    let champion_pool_size = stats.favorite_champions.len();
    if champion_pool_size >= 10 && stats.win_rate >= 55.0 {
        traits.push(SummonerTrait {
            name: "英雄池深".to_string(),
            description: format!("能熟练使用{}个英雄", champion_pool_size),
            score: champion_pool_size as i32,
            trait_type: "good".to_string(),
        });
    } else if champion_pool_size <= 3 && stats.total_games >= 20 {
        traits.push(SummonerTrait {
            name: "英雄池浅".to_string(),
            description: format!("只使用{}个英雄，容易被针对", champion_pool_size),
            score: champion_pool_size as i32,
            trait_type: "bad".to_string(),
        });
    }

    traits
}

fn find_participant<'a>(game: &'a Value, puuid: &str) -> Option<&'a Value> {
    let participant_id = game["participantIdentities"]
        .as_array()?
        .iter()
        .find(|id| id["player"]["puuid"].as_str() == Some(puuid))?
        ["participantId"]
        .as_i64()?;

    game["participants"]
        .as_array()?
        .iter()
        .find(|p| p["participantId"].as_i64() == Some(participant_id))
}

