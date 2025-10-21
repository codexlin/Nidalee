use crate::shared::types::{AnalysisChampionStats, MatchPerformance, PlayerMatchStats};
use crate::infrastructure::data_services::champion_data::service::get_champion_info;
use super::parser::ParsedGame;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 精度格式化函数
fn format_precision(value: f64, decimals: usize) -> f64 {
    let multiplier = 10_f64.powi(decimals as i32);
    (value * multiplier).round() / multiplier
}

/// 分析上下文
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// 当前队列ID（用于过滤相关对局）
    pub current_queue_id: Option<i32>,
    /// 是否只分析排位赛（420=单双排，440=灵活组排）
    pub ranked_only: bool,
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self {
            current_queue_id: None,
            ranked_only: false,
        }
    }
}

impl AnalysisContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_queue_id(mut self, queue_id: i32) -> Self {
        self.current_queue_id = Some(queue_id);
        self
    }

    #[allow(dead_code)]
    pub fn ranked_only(mut self) -> Self {
        self.ranked_only = true;
        self
    }
}

/// 通用玩家战绩分析器
///
/// 输入：解析后的对局数据 (ParsedGame)
/// 输出：完整的 PlayerMatchStats（包含所有计算好的字段）
pub fn analyze_player_stats(games: &[ParsedGame], _puuid: &str, context: AnalysisContext) -> PlayerMatchStats {
    // 获取今天零点的时间戳（毫秒）
    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();
    let current_ms = since_epoch.as_millis() as i64;
    // 计算今天零点（UTC时区）
    let today_start_ms = (current_ms / 86400000) * 86400000;

    // 过滤相关对局
    let relevant_games: Vec<&ParsedGame> = games
        .iter()
        .filter(|game| {
            let queue_id = game.queue_id as i32;

            // 根据上下文过滤
            if let Some(current_queue) = context.current_queue_id {
                // 精确匹配队列ID
                queue_id == current_queue
            } else if context.ranked_only {
                // 只统计排位赛（无具体队列时使用）
                queue_id == 420 || queue_id == 440
            } else {
                true // 无限制，显示所有对局
            }
        })
        .collect();

    let total_games = relevant_games.len() as u32;
    if total_games == 0 {
        return PlayerMatchStats::default();
    }

    // 初始化统计变量
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

    // 遍历所有对局 (ParsedGame 已经包含解析好的玩家数据)
    for game in &relevant_games {
        let player = &game.player_data;
        let win = player.win;
        let kills = player.kills as f64;
        let deaths = player.deaths as f64;
        let assists = player.assists as f64;
        let game_duration = game.game_duration as f64;

        // 基础统计
        if win {
            wins += 1;
        }
        total_kills += kills;
        total_deaths += deaths;
        total_assists += assists;
        total_duration_secs += game_duration;

        // 衍生指标数据
        total_damage_to_champs += player.damage_to_champions as f64;
        total_vision_score += player.vision_score as f64;
        total_cs += player.cs as f64;

        // 今日统计
        if game.game_creation >= today_start_ms {
            today_games += 1;
            if win {
                today_wins += 1;
            }
        }

        // 常用英雄统计
        let champion_id = player.champion_id;
        let entry = favorite_champions_map.entry(champion_id).or_insert((0, 0));
        entry.0 += 1;
        if win {
            entry.1 += 1;
        }

        // 最近战绩
        // ⭐ v3.1: 添加位置信息
        let position = role_to_position(&player.role, &player.lane);

        // 获取英雄名称
        let champion_name = get_champion_info(player.champion_id)
            .map(|info| info.name)
            .unwrap_or_else(|| format!("未知英雄({})", player.champion_id));

        recent_performance.push(MatchPerformance {
            game_id: Some(game.game_id),
            win,
            champion_id: player.champion_id,
            champion_name,
            kills: kills as i32,
            deaths: deaths as i32,
            assists: assists as i32,
            kda: player.kda,
            game_duration: Some(game_duration as i32),
            game_creation: Some(game.game_creation),
            queue_id: Some(game.queue_id),
            game_mode: None, // ParsedGame 没有 game_mode，可以后续添加
            role: player.role.clone(),
            lane: player.lane.clone(),
            position,
        });
    }

    // 计算平均值和衍生指标
    let total_duration_mins = if total_duration_secs > 0.0 {
        total_duration_secs / 60.0
    } else {
        1.0 // 避免除零
    };

    let avg_kills = if total_games > 0 {
        total_kills / total_games as f64
    } else {
        0.0
    };
    let avg_deaths = if total_games > 0 {
        total_deaths / total_games as f64
    } else {
        0.0
    };
    let avg_assists = if total_games > 0 {
        total_assists / total_games as f64
    } else {
        0.0
    };
    let avg_kda = if total_deaths > 0.0 {
        (total_kills + total_assists) / total_deaths
    } else {
        total_kills + total_assists
    };

    let dpm = total_damage_to_champs / total_duration_mins;
    let cspm = total_cs / total_duration_mins;
    let vspm = total_vision_score / total_duration_mins;

    // 处理常用英雄
    let mut favorite_champions: Vec<AnalysisChampionStats> = favorite_champions_map
        .into_iter()
        .map(|(champion_id, (games, wins))| {
            // 获取英雄名称
            let champion_name = get_champion_info(champion_id)
                .map(|info| info.name)
                .unwrap_or_else(|| format!("未知英雄({})", champion_id));

            AnalysisChampionStats {
                champion_id,
                champion_name,
                games,
                wins,
                win_rate: if games > 0 {
                    format_precision((wins as f64 / games as f64) * 100.0, 1)
                } else {
                    0.0
                },
            }
        })
        .collect();

    // 按游戏场次排序
    favorite_champions.sort_by(|a, b| b.games.cmp(&a.games));

    // 构建结果（注意：traits 将由 traits_analyzer 填充）
    PlayerMatchStats {
        total_games,
        wins,
        losses: total_games - wins,
        win_rate: if total_games > 0 {
            format_precision((wins as f64 / total_games as f64) * 100.0, 1)
        } else {
            0.0
        },
        avg_kills: format_precision(avg_kills, 2),
        avg_deaths: format_precision(avg_deaths, 2),
        avg_assists: format_precision(avg_assists, 2),
        avg_kda: format_precision(avg_kda, 2),
        today_games,
        today_wins,
        dpm: format_precision(dpm, 1),
        cspm: format_precision(cspm, 2),
        vspm: format_precision(vspm, 2),
        traits: Vec::new(), // 由 traits_analyzer 填充
        favorite_champions,
        recent_performance,
        advice: Vec::new(), // ⭐ v3.0: 由 advice 模块填充
    }
}

/// ⭐ v3.1: 将 role/lane 转换为中文位置名称
fn role_to_position(role: &str, lane: &str) -> String {
    let position = match (role, lane) {
        // 标准位置匹配
        ("DUO_CARRY", _) => "ADC",
        ("DUO_SUPPORT", _) => "辅助",
        ("SOLO", "TOP") => "上单",
        ("SOLO", "MIDDLE") | ("SOLO", "MID") => "中单",
        ("NONE", "JUNGLE") | ("JUNGLE", _) => "打野",

        // 仅根据 lane 判断（当 role 是 NONE 但 lane 有值时）
        ("NONE", "TOP") => "上单",
        ("NONE", "MIDDLE") | ("NONE", "MID") => "中单",
        ("NONE", "BOTTOM") => "下路", // 无法区分 ADC/辅助

        // 没有 timeline 数据（大乱斗、自定义等）
        ("NONE", "NONE") => {
            // 这种情况通常出现在：大乱斗、自定义游戏、或旧对局数据
            "灵活"
        }

        // 其他未知情况，记录日志以便调试
        _ => {
            println!("⚠️ 未识别的位置组合: role={}, lane={}", role, lane);
            "未知"
        }
    };

    position.to_string()
}
