use crate::domains::analysis::thresholds;
use crate::shared::types::{PlayerMatchStats, SummonerTrait};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RoleStats {
    pub games: u32,
    pub wins: u32,
    pub win_rate: f64,
}

/// 识别玩家各位置的表现
pub fn identify_player_roles(games: &[Value], puuid: &str) -> HashMap<String, RoleStats> {
    let mut role_data: HashMap<String, (u32, u32)> = HashMap::new();

    for game in games {
        if let Some((role, win)) = extract_role_info(game, puuid) {
            let entry = role_data.entry(role).or_insert((0, 0));
            entry.0 += 1;
            if win {
                entry.1 += 1;
            }
        }
    }

    role_data
        .into_iter()
        .map(|(role, (games, wins))| {
            let win_rate = if games > 0 {
                wins as f64 / games as f64 * 100.0
            } else {
                0.0
            };
            (role, RoleStats { games, wins, win_rate })
        })
        .collect()
}

fn extract_role_info(game: &Value, puuid: &str) -> Option<(String, bool)> {
    let participant_id = game["participantIdentities"]
        .as_array()?
        .iter()
        .find(|id| id["player"]["puuid"].as_str() == Some(puuid))?["participantId"]
        .as_i64()?;

    let participant = game["participants"]
        .as_array()?
        .iter()
        .find(|p| p["participantId"].as_i64() == Some(participant_id))?;

    let timeline = participant.get("timeline")?;
    let role = timeline["role"].as_str().unwrap_or("NONE");
    let lane = timeline["lane"].as_str().unwrap_or("NONE");

    let role_name = match (role, lane) {
        ("DUO_CARRY", _) => "ADC",
        ("DUO_SUPPORT", _) => "辅助",
        ("SOLO", "TOP") => "上单",
        ("SOLO", "MIDDLE") | ("SOLO", "MID") => "中单",
        ("NONE", "JUNGLE") | ("JUNGLE", _) => "打野",
        _ => return None,
    };

    let win = participant["stats"]["win"].as_bool().unwrap_or(false);
    Some((role_name.to_string(), win))
}

/// 基于位置的特征分析
pub fn analyze_role_based_traits(
    stats: &PlayerMatchStats,
    role_stats_map: &HashMap<String, RoleStats>,
) -> Vec<SummonerTrait> {
    let mut traits = Vec::new();

    let main_role = role_stats_map
        .iter()
        .filter(|(_, stat)| stat.games >= 5)
        .max_by_key(|(_, stat)| stat.games);

    if let Some((role, role_stats)) = main_role {
        if role_stats.win_rate >= thresholds::win_rate::EXCELLENT_OTHER {
            traits.push(SummonerTrait {
                name: format!("{}专精", role),
                description: format!("打{}胜率{:.0}%，非常擅长该位置", role, role_stats.win_rate),
                score: role_stats.win_rate as i32,
                trait_type: "good".to_string(),
            });
        }
    }

    let role_count = role_stats_map.iter().filter(|(_, s)| s.games >= 5).count();
    if role_count >= 3 && stats.win_rate >= thresholds::win_rate::GOOD {
        traits.push(SummonerTrait {
            name: "全能选手".to_string(),
            description: format!("能胜任{}个位置且胜率不错（{:.0}%）", role_count, stats.win_rate),
            score: role_count as i32,
            trait_type: "good".to_string(),
        });
    }

    traits
}
