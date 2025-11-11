/// 多位置分组分析模块
///
/// 职责：
/// - 按位置分组对局数据
/// - 分别分析每个位置的表现
/// - 生成多位置统计结果

use crate::domains::analysis::{
    analyze_player_stats, identify_position_from_game, parse_games,
    AnalysisContext, AnalysisStrategy, ParsedGame,
};
use crate::domains::tactical_advice::generate_advice;
use crate::shared::types::{
    AdvicePerspective, MultiPositionAnalysis, PositionStats, ChampionStat, TrendPoint,
};
use crate::infrastructure::data_services::champion_data::service::get_champion_info;
use serde_json::Value;
use std::collections::HashMap;

/// 按分析模式进行多位置分组分析
///
/// 参数:
/// - match_list_data: LCU API返回的对局列表
/// - current_puuid: 当前玩家PUUID
/// - analysis_mode: 前端选择的分析模式
/// - analysis_depth: 前端选择的分析深度
pub fn analyze_with_analysis_mode(
    match_list_data: Value,
    current_puuid: &str,
    analysis_mode: Option<crate::domains::analysis::analyzers::core::strategy::AnalysisMode>,
    analysis_depth: Option<crate::domains::analysis::analyzers::core::strategy::AnalysisDepth>,
) -> Result<MultiPositionAnalysis, String> {
    println!("\n📊 开始多位置分组分析（前端驱动模式）");

    // 1. 解析所有对局
    let empty_games = vec![];
    let games = match_list_data
        .get("games")
        .and_then(|games_obj| games_obj.get("games"))
        .and_then(|g| g.as_array())
        .unwrap_or(&empty_games);

    println!("📊 找到 {} 场对局记录", games.len());

    let parsed_games = parse_games(games, current_puuid);
    println!("✅ Parser: 解析了 {} 场对局数据", parsed_games.len());

    // 2. 根据分析模式过滤对局
    let analysis_mode = analysis_mode.unwrap_or(crate::domains::analysis::analyzers::core::strategy::AnalysisMode::MixedRanked);
    let analysis_depth = analysis_depth.unwrap_or(crate::domains::analysis::analyzers::core::strategy::AnalysisDepth::Deep);
    println!("🎯 分析模式: {} ({:?})", analysis_mode.description(), analysis_mode);
    println!("🎯 分析深度: {:?}", analysis_depth);

    let filtered_games: Vec<_> = {
        let queue_ids = analysis_mode.queue_ids();
        if queue_ids.is_empty() {
            // 全部模式：不过滤
            println!("🎯 全部模式：分析所有对局");
            parsed_games
        } else {
            // 特定模式：按队列ID过滤
            println!("🎯 过滤队列ID: {:?}", queue_ids);
            parsed_games
                .into_iter()
                .filter(|g| queue_ids.contains(&g.queue_id))
                .collect()
        }
    };

    println!("🔍 过滤后剩余 {} 场对局", filtered_games.len());

    if filtered_games.is_empty() {
        return Err(format!("没有符合条件的对局数据（模式：{}）", analysis_mode.description()));
    }

    // 3. 确定分析策略
    let strategy = analysis_mode.to_analysis_strategy(analysis_depth);
    println!("🎯 Strategy: {}", strategy.description());

    // 4. 按位置分组
    let mut position_groups: HashMap<String, Vec<ParsedGame>> = HashMap::new();

    for game in &filtered_games {
        let position = identify_position_from_game(
            &game.player_data.role,
            &game.player_data.lane,
            game.queue_id,
        );

        position_groups
            .entry(position)
            .or_insert_with(Vec::new)
            .push(game.clone());
    }

    println!("📍 识别到 {} 个不同位置", position_groups.len());

    // 5. 分析每个位置
    let mut position_stats_list = Vec::new();

    for (position, games) in position_groups.iter() {
        let game_count = games.len();

        // 跳过场次太少的位置（非排位赛）或者"未知"位置
        // 排位赛：至少1场就分析
        // 非排位赛："灵活"位置也分析
        let should_analyze = match strategy {
            AnalysisStrategy::SoloRanked | AnalysisStrategy::FlexRanked | AnalysisStrategy::MixedRanked => position != "未知",
            AnalysisStrategy::Other => true,
        };

        if !should_analyze {
            println!("  ⏭️  跳过位置 '{}' ({}场) - 未知位置", position, game_count);
            continue;
        }

        println!("  📊 分析位置 '{}' ({}场)", position, game_count);

        // 分析该位置的数据
        let context = AnalysisContext::new();
        let mut stats = analyze_player_stats(games, current_puuid, context);

        // 为排位赛生成建议
        if matches!(strategy, AnalysisStrategy::SoloRanked | AnalysisStrategy::FlexRanked | AnalysisStrategy::MixedRanked) {
            let advice = generate_advice(
                &stats,
                games,
                position,  // ⭐ 使用实际位置
                AdvicePerspective::SelfImprovement,
                None,
                &strategy,
            );
            stats.advice = advice;
        }

        // 计算该位置的胜场和胜率
        let wins = games.iter().filter(|g| g.player_data.win).count() as u32;
        let win_rate = if game_count > 0 {
            (wins as f64 / game_count as f64) * 100.0
        } else {
            0.0
        };

        // 生成该位置的英雄池数据
        let champion_pool = generate_champion_pool(&games);

        // 生成该位置的胜率趋势数据
        let win_rate_trend = generate_win_rate_trend(&games);

        position_stats_list.push(PositionStats {
            position: position.clone(),
            games: game_count as u32,
            wins,
            win_rate,
            stats,
            champion_pool: Some(champion_pool),
            win_rate_trend: Some(win_rate_trend),
        });
    }

    // 6. 按场次排序（主位置排第一）
    position_stats_list.sort_by(|a, b| b.games.cmp(&a.games));

    let main_position = position_stats_list
        .first()
        .map(|p| p.position.clone())
        .unwrap_or_else(|| "未知".to_string());

    println!("🎯 主要位置: {} ({}场)",
        main_position,
        position_stats_list.first().map(|p| p.games).unwrap_or(0)
    );

    // 7. 计算总览数据（所有对局合计）
    let context = AnalysisContext::new();
    let overall_stats = analyze_player_stats(&filtered_games, current_puuid, context);

    println!("✅ 多位置分析完成:");
    println!("   - 总对局: {}", overall_stats.total_games);
    println!("   - 位置数: {}", position_stats_list.len());
    for pos_stat in &position_stats_list {
        println!("     • {}: {}场 (胜率{:.1}%)",
            pos_stat.position,
            pos_stat.games,
            pos_stat.win_rate
        );
    }

    Ok(MultiPositionAnalysis {
        position_stats: position_stats_list,
        main_position,
        overall_stats,
    })
}

/// 生成该位置的英雄池数据（Top 5）
fn generate_champion_pool(games: &[ParsedGame]) -> Vec<ChampionStat> {
    let mut champion_stats: HashMap<i32, (u32, u32, f64)> = HashMap::new(); // (games, wins, total_kda)

    // 统计每个英雄的使用情况
    for game in games {
        let champion_id = game.player_data.champion_id;
        let win = game.player_data.win;
        let kills = game.player_data.kills as f64;
        let deaths = game.player_data.deaths as f64;
        let assists = game.player_data.assists as f64;

        // 计算KDA
        let kda = if deaths > 0.0 {
            (kills + assists) / deaths
        } else {
            kills + assists // 死亡为0时，KDA = K+A
        };

        let entry = champion_stats.entry(champion_id).or_insert((0, 0, 0.0));
        entry.0 += 1; // 总场次
        if win {
            entry.1 += 1; // 胜场
        }
        entry.2 += kda; // 累计KDA
    }

    // 转换为ChampionStat并排序
    let mut champion_pool: Vec<ChampionStat> = champion_stats
        .into_iter()
        .map(|(champion_id, (games, wins, total_kda))| {
            let win_rate = if games > 0 {
                (wins as f64 / games as f64) * 100.0
            } else {
                0.0
            };

            let avg_kda = if games > 0 {
                total_kda / games as f64
            } else {
                0.0
            };

            // 获取英雄名称
            let champion_name = get_champion_info(champion_id)
                .map(|info| info.name)
                .unwrap_or_else(|| format!("未知英雄({})", champion_id));

            ChampionStat {
                champion_id,
                champion_name: Some(champion_name),
                games,
                wins,
                win_rate,
                avg_kda,
            }
        })
        .collect();

    // 按场次排序，取前5个
    champion_pool.sort_by(|a, b| b.games.cmp(&a.games));
    champion_pool.truncate(5);

    champion_pool
}

/// 生成该位置的胜率趋势数据
fn generate_win_rate_trend(games: &[ParsedGame]) -> Vec<TrendPoint> {
    let mut trend_points = Vec::new();

    // 按时间排序（从早到晚）
    let mut sorted_games = games.to_vec();
    sorted_games.sort_by(|a, b| a.game_creation.cmp(&b.game_creation));

    let mut cumulative_wins = 0;
    let mut cumulative_games = 0;

    for (index, game) in sorted_games.iter().enumerate() {
        cumulative_games += 1;
        if game.player_data.win {
            cumulative_wins += 1;
        }

        let cumulative_win_rate = if cumulative_games > 0 {
            (cumulative_wins as f64 / cumulative_games as f64) * 100.0
        } else {
            0.0
        };

        // 计算移动平均胜率（最近5场）
        let moving_avg_win_rate = if cumulative_games >= 5 {
            let start_idx = if index >= 4 { index - 4 } else { 0 };
            let recent_games = &sorted_games[start_idx..=index];
            let recent_wins = recent_games.iter().filter(|g| g.player_data.win).count();
            (recent_wins as f64 / recent_games.len() as f64) * 100.0
        } else {
            cumulative_win_rate
        };

        trend_points.push(TrendPoint {
            index: index as u32,
            win: game.player_data.win,
            cumulative_win_rate,
            moving_avg_win_rate,
        });
    }

    trend_points
}
