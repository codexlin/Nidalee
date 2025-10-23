/// 多位置分组分析模块
///
/// 职责：
/// - 按位置分组对局数据
/// - 分别分析每个位置的表现
/// - 生成多位置统计结果

use crate::domains::analysis::{
    analyze_player_stats, identify_main_role, identify_position_from_game, parse_games,
    AnalysisContext, AnalysisStrategy, ParsedGame,
};
use crate::domains::tactical_advice::generate_advice;
use crate::shared::types::{
    AdvicePerspective, MultiPositionAnalysis, PlayerMatchStats, PositionStats,
};
use serde_json::Value;
use std::collections::HashMap;

/// 按位置分组分析对局数据
///
/// 参数:
/// - match_list_data: LCU API返回的对局列表
/// - current_puuid: 当前玩家PUUID
/// - queue_id: 可选的队列ID过滤
pub fn analyze_with_position_grouping(
    match_list_data: Value,
    current_puuid: &str,
    queue_id: Option<i32>,
) -> Result<MultiPositionAnalysis, String> {
    println!("\n📊 开始多位置分组分析");

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

    // 2. 按 queue_id 过滤（如果指定了）
    let filtered_games: Vec<_> = if let Some(qid) = queue_id {
        parsed_games
            .into_iter()
            .filter(|g| g.queue_id == qid as i64)
            .collect()
    } else {
        parsed_games
    };

    println!("🔍 过滤后剩余 {} 场对局", filtered_games.len());

    if filtered_games.is_empty() {
        return Err("没有符合条件的对局数据".to_string());
    }

    // 3. 确定分析策略
    let strategy = if let Some(qid) = queue_id {
        AnalysisStrategy::from_queue_id(qid as i64)
    } else {
        AnalysisStrategy::from_games(&filtered_games)
    };

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
            AnalysisStrategy::Ranked => position != "未知",
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
        if matches!(strategy, AnalysisStrategy::Ranked) {
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

        position_stats_list.push(PositionStats {
            position: position.clone(),
            games: game_count as u32,
            wins,
            win_rate,
            stats,
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

