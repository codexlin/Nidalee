use super::parser::ParsedGame;
use crate::domains::analysis::evidence::position_from_role_lane;
use crate::shared::types::{AnalysisChampionStats, MatchPerformance, PlayerMatchStats};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 精度格式化函数
fn format_precision(value: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).round() / multiplier
}

/// 分析上下文
#[derive(Debug, Clone, Default)]
pub struct AnalysisContext {
    /// 当前队列ID（用于过滤相关对局）
    pub current_queue_id: Option<i32>,
    /// 是否只分析排位赛（420=单双排，440=灵活组排）
    pub ranked_only: bool,
}

impl AnalysisContext {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn ranked_only(mut self) -> Self {
        self.ranked_only = true;
        self
    }
}

/// 英雄名解析端口（领域层不认识静态数据来源）
pub type ChampionNameResolver<'a> = &'a dyn Fn(i32) -> Option<String>;

/// 将一场已解析对局转换为前端展示模型。
pub fn match_performance_from_game(
    game: &ParsedGame,
    champion_name: Option<ChampionNameResolver<'_>>,
) -> MatchPerformance {
    let player = &game.player_data;
    MatchPerformance {
        game_id: Some(game.game_id),
        win: player.win,
        champion_id: player.champion_id,
        champion_name: resolve_champion_name(champion_name, player.champion_id),
        kills: player.kills,
        deaths: player.deaths,
        assists: player.assists,
        kda: player.kda,
        grade: crate::domains::analysis::thresholds::kda::grade_from_kda(player.kda).to_string(),
        game_duration: Some(game.game_duration),
        game_creation: Some(game.game_creation),
        queue_id: Some(game.queue_id),
        game_mode: None,
        role: player.role.clone(),
        lane: player.lane.clone(),
        position: position_from_role_lane(&player.role, &player.lane, game.queue_id)
            .as_str()
            .to_string(),
    }
}

/// 通用玩家战绩分析器（可注入英雄名解析）
///
/// 输入：解析后的对局数据 (ParsedGame)
/// 输出：完整的 PlayerMatchStats（包含所有计算好的字段）
///
/// `position` 字段统一为 ASCII 位置码（TOP/JUNGLE/MID/ADC/SUPPORT/ARAM/FLEX/UNKNOWN），
/// 中文展示由前端负责。
pub fn analyze_player_stats_with_resolver(
    games: &[ParsedGame],
    _puuid: &str,
    context: AnalysisContext,
    champion_name: Option<ChampionNameResolver<'_>>,
) -> PlayerMatchStats {
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let current_ms = since_epoch.as_millis() as i64;
    let today_start_ms = (current_ms / 86400000) * 86400000;

    let relevant_games: Vec<&ParsedGame> = games
        .iter()
        .filter(|game| {
            let queue_id = game.queue_id as i32;
            if let Some(current_queue) = context.current_queue_id {
                queue_id == current_queue
            } else if context.ranked_only {
                queue_id == 420 || queue_id == 440
            } else {
                true
            }
        })
        .collect();

    let total_games = relevant_games.len() as u32;
    if total_games == 0 {
        return PlayerMatchStats::default();
    }

    let mut wins = 0u32;
    let mut today_games = 0u32;
    let mut today_wins = 0u32;
    let mut total_kills = 0.0;
    let mut total_deaths = 0.0;
    let mut total_assists = 0.0;
    let mut total_duration_secs = 0.0;
    let mut total_damage_to_champs = 0.0;
    let mut total_vision_score = 0.0;
    let mut total_cs = 0.0;
    let mut favorite_champions_map: HashMap<i32, (u32, u32)> = HashMap::new();
    let mut recent_performance = Vec::new();

    for game in &relevant_games {
        let player = &game.player_data;
        let win = player.win;
        let kills = player.kills as f64;
        let deaths = player.deaths as f64;
        let assists = player.assists as f64;
        let game_duration = game.game_duration as f64;

        if win {
            wins += 1;
        }
        total_kills += kills;
        total_deaths += deaths;
        total_assists += assists;
        total_duration_secs += game_duration;
        total_damage_to_champs += player.damage_to_champions as f64;
        total_vision_score += player.vision_score as f64;
        // 总补刀 = 线兵 + 野怪；打野若只算 totalMinionsKilled 会落到辅助同级假 CS
        total_cs += (player.cs + player.jungle_cs) as f64;

        if game.game_creation >= today_start_ms {
            today_games += 1;
            if win {
                today_wins += 1;
            }
        }

        let champion_id = player.champion_id;
        let entry = favorite_champions_map.entry(champion_id).or_insert((0, 0));
        entry.0 += 1;
        if win {
            entry.1 += 1;
        }

        recent_performance.push(match_performance_from_game(game, champion_name));
    }

    let total_duration_mins = if total_duration_secs > 0.0 {
        total_duration_secs / 60.0
    } else {
        1.0
    };

    let avg_kills = total_kills / total_games as f64;
    let avg_deaths = total_deaths / total_games as f64;
    let avg_assists = total_assists / total_games as f64;
    let avg_kda = if total_deaths > 0.0 {
        (total_kills + total_assists) / total_deaths
    } else {
        total_kills + total_assists
    };

    let dpm = total_damage_to_champs / total_duration_mins;
    let cspm = total_cs / total_duration_mins;
    let vspm = total_vision_score / total_duration_mins;

    let mut favorite_champions: Vec<AnalysisChampionStats> = favorite_champions_map
        .into_iter()
        .map(|(champion_id, (games, wins))| AnalysisChampionStats {
            champion_id,
            champion_name: resolve_champion_name(champion_name, champion_id),
            games,
            wins,
            win_rate: if games > 0 {
                format_precision((wins as f64 / games as f64) * 100.0, 1)
            } else {
                0.0
            },
        })
        .collect();

    favorite_champions.sort_by_key(|champion| std::cmp::Reverse(champion.games));

    PlayerMatchStats {
        total_games,
        wins,
        losses: total_games - wins,
        win_rate: format_precision((wins as f64 / total_games as f64) * 100.0, 1),
        avg_kills: format_precision(avg_kills, 2),
        avg_deaths: format_precision(avg_deaths, 2),
        avg_assists: format_precision(avg_assists, 2),
        avg_kda: format_precision(avg_kda, 2),
        today_games,
        today_wins,
        dpm: format_precision(dpm, 1),
        cspm: format_precision(cspm, 2),
        vspm: format_precision(vspm, 2),
        traits: Vec::new(),
        favorite_champions,
        recent_performance,
        advice: Vec::new(),
    }
}

fn resolve_champion_name(resolver: Option<ChampionNameResolver<'_>>, champion_id: i32) -> Option<String> {
    resolver.and_then(|resolve| resolve(champion_id))
}

#[cfg(test)]
mod tests {
    use super::resolve_champion_name;

    #[test]
    fn champion_name_should_remain_absent_without_a_resolver() {
        assert_eq!(resolve_champion_name(None, 67), None);
    }

    #[test]
    fn champion_name_should_remain_absent_when_resolver_misses() {
        let resolver = |_champion_id| None;

        assert_eq!(resolve_champion_name(Some(&resolver), 67), None);
    }

    #[test]
    fn champion_name_should_use_resolved_catalog_value() {
        let resolver = |champion_id| (champion_id == 67).then(|| "暗夜猎手".to_string());

        assert_eq!(resolve_champion_name(Some(&resolver), 67), Some("暗夜猎手".to_string()));
    }
}
