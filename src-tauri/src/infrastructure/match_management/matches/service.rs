use crate::domains::analysis::{
    analyze_advanced_traits, analyze_distribution_traits, analyze_player_stats, analyze_role_based_traits,
    analyze_timeline_traits, analyze_traits, analyze_win_loss_pattern, identify_main_role, identify_player_roles,
    identify_position_from_game, optimize_traits, parse_games, AnalysisContext, AnalysisStrategy,
    parse_timeline_data, TimelineAnalysis,
};
use crate::domains::tactical_advice::generate_advice;
use crate::shared::types::{
    AdvicePerspective, GameAdvice, GameDetail, MultiPositionAnalysis, ParticipantInfo, ParticipantStats,
    PlayerMatchStats, PositionStats, TeamInfo, TeamStats,
};
use crate::shared::utils::{lcu_get, lcu_request_json};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use reqwest::{Client, Method};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

// 以下内容为原 match_history.rs 全部内容，粘贴至此
// 其余内容保持不变，全部迁移

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiGameData {
    game_id: u64,
    game_duration: i32,
    game_creation: i64,
    game_mode: String,
    game_type: String,
    game_version: String,
    map_id: i32,
    queue_id: i32,
    teams: Vec<TeamInfo>,
    participants: Vec<ApiParticipant>,
    participant_identities: Vec<ApiParticipantIdentity>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiParticipant {
    participant_id: i32,
    #[serde(default)]
    team_id: Option<i32>,
    #[serde(default)]
    champion_id: Option<i32>,
    stats: ApiParticipantStats,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiParticipantStats {
    #[serde(default)]
    kills: Option<i32>,
    #[serde(default)]
    deaths: Option<i32>,
    #[serde(default)]
    assists: Option<i32>,
    #[serde(default)]
    champ_level: Option<i32>,
    #[serde(default)]
    gold_earned: Option<i32>,
    #[serde(default)]
    total_damage_dealt_to_champions: Option<i32>,
    #[serde(default)]
    total_damage_taken: Option<i32>,
    #[serde(default)]
    vision_score: Option<i32>,
    #[serde(default)]
    item0: Option<i32>,
    #[serde(default)]
    item1: Option<i32>,
    #[serde(default)]
    item2: Option<i32>,
    #[serde(default)]
    item3: Option<i32>,
    #[serde(default)]
    item4: Option<i32>,
    #[serde(default)]
    item5: Option<i32>,
    #[serde(default)]
    item6: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiParticipantIdentity {
    participant_id: i32,
    player: ApiPlayer,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiPlayer {
    #[serde(default)]
    summoner_name: Option<String>,
    #[serde(default)]
    game_name: Option<String>,
    #[serde(default)]
    tag_line: Option<String>,
    profile_icon: i64,
}

/// 获取当前玩家历史战绩统计（自动认证、统一请求、日志耗时）
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `end_count`: 获取对局数量
/// - `queue_id`: 可选的队列ID过滤（如 420=单排, 440=灵活排, 450=大乱斗）
pub async fn get_match_history(
    client: &Client,
    end_count: usize,
    queue_id: Option<i32>,
) -> Result<PlayerMatchStats, String> {
    println!("\n🔍 ===== 开始获取我的战绩 =====");
    if let Some(qid) = queue_id {
        println!("🎯 队列过滤: queueId={}", qid);
    }

    // 第1步：获取当前召唤师信息来得到PUUID
    println!("\n📍 第1步：获取当前召唤师信息");
    let summoner_data: Value = lcu_get(client, "/lol-summoner/v1/current-summoner").await?;
    let puuid = summoner_data
        .get("puuid")
        .and_then(|p| p.as_str())
        .ok_or_else(|| "未找到PUUID".to_string())?;
    println!("🆔 提取到的PUUID: {}", puuid);

    // 第2步：使用PUUID获取对局列表
    println!("\n📍 第2步：使用PUUID获取对局列表");
    let safe_end = if end_count == 0 { 20 } else { end_count.min(100) };
    // LCU API 的 endIndex 是包含的，所以需要减1
    let actual_end_index = if safe_end > 0 { safe_end - 1 } else { 0 };
    println!(
        "🔢 请求的对局数量: end_count={}, safe_end={}, actual_end_index={}",
        end_count, safe_end, actual_end_index
    );
    let match_list_url = format!(
        "/lol-match-history/v1/products/lol/{}/matches?begIndex=0&endIndex={}",
        puuid, actual_end_index
    );
    println!("🌐 请求URL: {}", match_list_url);
    let match_list_data: Value = lcu_get(client, &match_list_url).await?;

    // 第3步：直接分析对局列表数据
    println!("\n📍 第3步：分析对局列表数据");
    let statistics = analyze_match_list_data(match_list_data, puuid, queue_id)?;

    println!("\n✅ ===== 我的战绩查询完成 =====");
    println!("📊 最终统计结果:");
    println!("   - 总对局: {}", statistics.total_games);
    println!("   - 胜场: {}", statistics.wins);
    println!("   - 负场: {}", statistics.losses);
    println!("   - 胜率: {:.1}%", statistics.win_rate);
    println!("   - 平均KDA: {:.2}", statistics.avg_kda);
    println!("   - 最近对局数: {}", statistics.recent_performance.len());

    Ok(statistics)
}

pub async fn get_game_detail_logic(client: &Client, game_id: u64) -> Result<GameDetail, String> {
    let path = format!("/lol-match-history/v1/games/{}", game_id);
    let api_game_data: ApiGameData = lcu_request_json(client, Method::GET, &path, None)
        .await
        .map_err(|e| format!("获取游戏详细信息失败: {}", e))?;

    let mut blue_team_stats = TeamStats::default();
    let mut red_team_stats = TeamStats::default();
    let mut max_damage = 0;
    let mut best_player_champion_id = 0;
    let mut max_tank = 0;
    let mut max_tank_champion_id = 0;
    let max_streak = 0;
    let max_streak_champion_id = 0;

    let player_map: HashMap<i32, ApiPlayer> = api_game_data
        .participant_identities
        .into_iter()
        .map(|p| (p.participant_id, p.player))
        .collect();

    let mut participants: Vec<ParticipantInfo> = Vec::with_capacity(api_game_data.participants.len());
    for p in api_game_data.participants {
        let stats = p.stats; // No need to clone anymore
        let kills = stats.kills.unwrap_or(0);
        let deaths = stats.deaths.unwrap_or(0);
        let assists = stats.assists.unwrap_or(0);
        let damage = stats.total_damage_dealt_to_champions.unwrap_or(0);
        let damage_taken = stats.total_damage_taken.unwrap_or(0);
        let gold = stats.gold_earned.unwrap_or(0);
        let vision = stats.vision_score.unwrap_or(0);
        let champ_level = stats.champ_level.unwrap_or(0);
        let champion_id = p.champion_id.unwrap_or(0);
        let team_id = p.team_id.unwrap_or(0);

        if team_id == 100 {
            blue_team_stats.kills += kills;
            blue_team_stats.gold_earned += gold;
            blue_team_stats.total_damage_dealt_to_champions += damage;
            blue_team_stats.vision_score += vision;
        } else {
            red_team_stats.kills += kills;
            red_team_stats.gold_earned += gold;
            red_team_stats.total_damage_dealt_to_champions += damage;
            red_team_stats.vision_score += vision;
        }

        if damage > max_damage {
            max_damage = damage;
            best_player_champion_id = champion_id;
        }

        if damage_taken > max_tank {
            max_tank = damage_taken;
            max_tank_champion_id = champion_id;
        }

        let player_identity = player_map.get(&p.participant_id);
        let summoner_name = player_identity.map_or_else(String::new, |pi| {
            if let (Some(name), Some(tag)) = (&pi.game_name, &pi.tag_line) {
                if !name.is_empty() && !tag.is_empty() {
                    return format!("{}#{}", name, tag);
                }
            }
            if let Some(s_name) = &pi.summoner_name {
                return s_name.clone();
            }
            String::from("未知玩家")
        });
        let profile_icon_id = player_identity.map_or(0, |pi| pi.profile_icon);

        participants.push(ParticipantInfo {
            participant_id: p.participant_id,
            champion_id,
            summoner_name,
            profile_icon_id,
            team_id,
            rank_tier: None,
            score: None,
            stats: ParticipantStats {
                kills,
                deaths,
                assists,
                champ_level,
                gold_earned: gold,
                total_damage_dealt_to_champions: damage,
                total_damage_taken: damage_taken,
                vision_score: vision,
                item0: stats.item0,
                item1: stats.item1,
                item2: stats.item2,
                item3: stats.item3,
                item4: stats.item4,
                item5: stats.item5,
                item6: stats.item6,
            },
        });
    }

    Ok(GameDetail {
        game_id: api_game_data.game_id,
        game_duration: api_game_data.game_duration,
        game_creation: api_game_data.game_creation,
        game_mode: api_game_data.game_mode,
        game_type: api_game_data.game_type,
        game_version: api_game_data.game_version,
        map_id: api_game_data.map_id,
        queue_id: api_game_data.queue_id,
        teams: api_game_data.teams,
        participants,
        blue_team_stats,
        red_team_stats,
        best_player_champion_id,
        max_damage,
        max_tank_champion_id,
        max_tank,
        max_streak_champion_id,
        max_streak,
    })
}

/// 获取指定召唤师最近几场简单战绩
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `puuid`: 玩家 PUUID
/// - `count`: 获取对局数量
/// - `queue_id`: 可选的队列ID，用于过滤（如 420=单排, 440=灵活排）
/// - `perspective`: 建议视角（默认SelfImprovement）
/// - `target_name`: 目标玩家名称（用于建议措辞）
pub async fn get_recent_matches_by_puuid(
    client: &Client,
    puuid: &str,
    count: usize,
    queue_id: Option<i32>,
) -> Result<PlayerMatchStats, String> {
    get_recent_matches_by_puuid_with_perspective(
        client,
        puuid,
        count,
        queue_id,
        AdvicePerspective::SelfImprovement,
        None,
    )
    .await
}

/// 获取指定召唤师最近几场战绩（支持指定建议视角）
///
/// 用于 TeamAnalysis 时可以指定不同的视角
pub async fn get_recent_matches_by_puuid_with_perspective(
    client: &Client,
    puuid: &str,
    count: usize,
    queue_id: Option<i32>,
    perspective: AdvicePerspective,
    target_name: Option<String>,
) -> Result<PlayerMatchStats, String> {
    let url = format!(
        "/lol-match-history/v1/products/lol/{}/matches?begIndex=0&endIndex={}",
        puuid, count
    );
    let match_list_data: Value = lcu_get(client, &url).await?;
    // 第3步：直接分析对局列表数据，传入视角参数
    let statistics =
        analyze_match_list_data_with_perspective(match_list_data, puuid, queue_id, perspective, target_name)?;
    Ok(statistics)
}

fn analyze_match_list_data(
    match_list_data: Value,
    current_puuid: &str,
    queue_id: Option<i32>,
) -> Result<PlayerMatchStats, String> {
    // ⭐ v3.4: 使用多位置分组分析，但只返回总览数据（保持向后兼容）
    // TODO: 后续可以修改返回类型为 MultiPositionAnalysis
    let multi_position_result = super::position_analysis::analyze_with_analysis_mode(
        match_list_data.clone(),
        current_puuid,
        None, // 使用默认的MixedRanked模式
        None, // 使用默认的Deep深度
    )?;

    // 目前只返回overall_stats，保持API兼容性
    Ok(multi_position_result.overall_stats)
}

/// 分析对局列表数据（支持指定建议视角）⭐
///
/// 分析frames时间线数据
fn analyze_frames_timeline_data(games: &[Value], target_puuid: &str) -> Option<TimelineAnalysis> {
    // 查找包含match_timeline_json的游戏
    for game in games {
        if let Some(timeline_json) = game.get("match_timeline_json") {
            // 查找目标玩家的participant_id
            if let Some(participant_identities) = game.get("participantIdentities").and_then(|p| p.as_array()) {
                for identity in participant_identities {
                    if identity["player"]["puuid"].as_str() == Some(target_puuid) {
                        let participant_id = identity["participantId"].as_i64().unwrap_or(0) as i32;

                        // 识别对手（简化版，实际应该基于位置信息）
                        let opponent_id = identify_lane_opponent_simple(game, participant_id);

                        return parse_timeline_data(timeline_json, participant_id, opponent_id);
                    }
                }
            }
        }
    }
    None
}

/// 简化的对手识别（基于队伍ID）
fn identify_lane_opponent_simple(game: &Value, target_participant_id: i32) -> Option<i32> {
    let participants = game.get("participants")?.as_array()?;

    // 找到目标玩家
    let target_participant = participants.iter().find(|p| {
        p["participantId"].as_i64().unwrap_or(0) as i32 == target_participant_id
    })?;

    let target_team_id = target_participant["teamId"].as_i64().unwrap_or(0) as i32;

    // 找到敌方队伍中的第一个玩家作为对手
    for participant in participants {
        let team_id = participant["teamId"].as_i64().unwrap_or(0) as i32;
        let participant_id = participant["participantId"].as_i64().unwrap_or(0) as i32;

        if team_id != target_team_id {
            return Some(participant_id);
        }
    }

    None
}

/// 基于frames分析生成特征
fn generate_frames_based_traits(timeline_analysis: &TimelineAnalysis, main_role: &str) -> Vec<crate::shared::types::SummonerTrait> {
    let mut traits = Vec::new();

    // 对手分析特征
    if timeline_analysis.opponent_comparison.overall_advantage > 5.0 {
        traits.push(crate::shared::types::SummonerTrait {
            name: "对线优势".to_string(),
            description: "整体表现优于对手".to_string(),
            score: 8,
            trait_type: "good".to_string(),
        });
    } else if timeline_analysis.opponent_comparison.overall_advantage < -5.0 {
        traits.push(crate::shared::types::SummonerTrait {
            name: "对线劣势".to_string(),
            description: "整体表现不如对手".to_string(),
            score: 4,
            trait_type: "bad".to_string(),
        });
    }

    // 分阶段效率特征
    if timeline_analysis.early_game.cs_per_minute > 8.0 {
        traits.push(crate::shared::types::SummonerTrait {
            name: "对线强势".to_string(),
            description: "对线期补刀效率高".to_string(),
            score: 8,
            trait_type: "good".to_string(),
        });
    }

    if timeline_analysis.mid_game.gold_per_minute > timeline_analysis.early_game.gold_per_minute * 1.1 {
        traits.push(crate::shared::types::SummonerTrait {
            name: "中期爆发".to_string(),
            description: "中期金币获取效率提升".to_string(),
            score: 7,
            trait_type: "good".to_string(),
        });
    }

    // 关键事件特征
    let kill_events = timeline_analysis.key_events.iter()
        .filter(|e| e.event_type == "CHAMPION_KILL" && e.importance_score > 0.0)
        .count();

    if kill_events > 3 {
        traits.push(crate::shared::types::SummonerTrait {
            name: "击杀机器".to_string(),
            description: "关键击杀次数多".to_string(),
            score: 8,
            trait_type: "good".to_string(),
        });
    }

    traits
}

/// 生成基于时间线分析的增强建议
fn generate_timeline_enhanced_advice(timeline_analysis: &TimelineAnalysis, main_role: &str) -> Vec<crate::shared::types::GameAdvice> {
    let mut advice = Vec::new();

    // 对线期建议
    if timeline_analysis.early_game.cs_difference < -10.0 {
        advice.push(crate::shared::types::GameAdvice {
            title: "对线期补刀劣势".to_string(),
            problem: "对线期补刀被对手压制".to_string(),
            evidence: format!("前10分钟补刀差: {:.1}", timeline_analysis.early_game.cs_difference),
            suggestions: vec![
                "加强补刀练习，提高基本功".to_string(),
                "注意兵线控制，避免被压制".to_string(),
                "考虑换血时机，寻找击杀机会".to_string(),
            ],
            priority: 8,
            category: crate::shared::types::AdviceCategory::Laning,
            perspective: crate::shared::types::AdvicePerspective::SelfImprovement,
            affected_role: Some(main_role.to_string()),
            target_player: None,
        });
    } else if timeline_analysis.early_game.cs_difference > 10.0 {
        advice.push(crate::shared::types::GameAdvice {
            title: "对线期优势明显".to_string(),
            problem: "对线期补刀优势明显".to_string(),
            evidence: format!("前10分钟补刀差: +{:.1}", timeline_analysis.early_game.cs_difference),
            suggestions: vec![
                "继续保持压制，扩大优势".to_string(),
                "考虑游走支援其他路".to_string(),
                "注意视野控制，防止被抓".to_string(),
            ],
            priority: 6,
            category: crate::shared::types::AdviceCategory::Laning,
            perspective: crate::shared::types::AdvicePerspective::SelfImprovement,
            affected_role: Some(main_role.to_string()),
            target_player: None,
        });
    }

    // 发育期建议
    if timeline_analysis.mid_game.cs_per_minute < 6.0 {
        advice.push(crate::shared::types::GameAdvice {
            title: "中期发育缓慢".to_string(),
            problem: "中期补刀效率偏低".to_string(),
            evidence: format!("中期补刀/分: {:.1}", timeline_analysis.mid_game.cs_per_minute),
            suggestions: vec![
                "注意发育节奏，不要盲目参团".to_string(),
                "寻找安全的补刀机会".to_string(),
                "考虑换线发育".to_string(),
            ],
            priority: 7,
            category: crate::shared::types::AdviceCategory::Farming,
            perspective: crate::shared::types::AdvicePerspective::SelfImprovement,
            affected_role: Some(main_role.to_string()),
            target_player: None,
        });
    }

    // 对手比较建议
    if timeline_analysis.opponent_comparison.overall_advantage < -5.0 {
        advice.push(crate::shared::types::GameAdvice {
            title: "整体表现不如对手".to_string(),
            problem: "各项指标都不如对手".to_string(),
            evidence: format!("综合优势: {:.1}", timeline_analysis.opponent_comparison.overall_advantage),
            suggestions: vec![
                "需要提升对线技巧".to_string(),
                "加强基本功练习".to_string(),
                "学习对手的打法".to_string(),
            ],
            priority: 9,
            category: crate::shared::types::AdviceCategory::Laning,
            perspective: crate::shared::types::AdvicePerspective::SelfImprovement,
            affected_role: Some(main_role.to_string()),
            target_player: None,
        });
    }

    advice
}

/// 用于 TeamAnalysis 时可以为敌方生成 Targeting，为队友生成 Collaboration
fn analyze_match_list_data_with_perspective(
    match_list_data: Value,
    current_puuid: &str,
    queue_id: Option<i32>,
    advice_perspective: AdvicePerspective,
    target_player_name: Option<String>,
) -> Result<PlayerMatchStats, String> {
    println!("📊 开始分析对局列表数据 (使用优化架构: Parser + Strategy)");
    println!("👤 目标玩家PUUID: {}", current_puuid);

    let empty_games = vec![];
    let games = match_list_data
        .get("games")
        .and_then(|games_obj| games_obj.get("games"))
        .and_then(|g| g.as_array())
        .unwrap_or(&empty_games);

    println!("📊 找到 {} 场对局记录", games.len());

    // === 第1步: Parser 层 - 解析原始数据为统一格式 ===
    let parsed_games = parse_games(games, current_puuid);
    println!("✅ Parser: 解析了 {} 场对局数据", parsed_games.len());

    // === 第2步: Strategy 层 - 根据队列选择分析策略 ===
    let strategy = if let Some(qid) = queue_id {
        let strategy = AnalysisStrategy::from_queue_id(qid as i64);
        println!("🎯 Strategy: {} (queueId={})", strategy.description(), qid);
        strategy
    } else {
        let strategy = AnalysisStrategy::from_games(&parsed_games);
        println!("🎯 Strategy: {} (自动推断)", strategy.description());
        strategy
    };
    println!("📊 分析方案: {}", strategy.analysis_layers());

    // === 第3步: Analyzer 层 - 使用解析后的数据进行统计计算 ===
    let mut context = AnalysisContext::new();

    // 根据队列ID设置分析上下文（精确匹配）
    if let Some(qid) = queue_id {
        context = context.with_queue_id(qid);
    }

    let mut player_stats = analyze_player_stats(&parsed_games, current_puuid, context);

    // === 第4步: 多层次特征分析（根据策略决定分析深度）===
    let mut traits = Vec::new();

    // 第1层：基础特征（所有模式都执行）
    traits.extend(analyze_traits(&player_stats));

    // 第1.5层：队列感知特征（所有模式都执行，提供队列特定的评估）
    traits.extend(crate::domains::analysis::analyzers::traits::queue_aware::analyze_queue_aware_traits(
        &player_stats,
        queue_id,
    ));

    // 第2-5层：深度分析（仅排位模式）
    if strategy.enable_advanced_analysis() {
        // 第2层：深度特征（参团率、伤害占比、稳定性、趋势）
        traits.extend(analyze_advanced_traits(&player_stats, games, current_puuid));
    }

    if strategy.enable_role_analysis() {
        // 第3层：位置特征（基于位置的专项分析）
        let role_stats_map = identify_player_roles(games, current_puuid);
        traits.extend(analyze_role_based_traits(&player_stats, &role_stats_map));
    }

    if strategy.enable_distribution_analysis() {
        // 第4层：分布特征（高光时刻、崩盘场次、稳定性）
        traits.extend(analyze_distribution_traits(&player_stats.recent_performance));

        // ⭐ 第4.5层：时间线特征（对线压制、稳定发育、爆发成长）NEW
        let main_role = identify_main_role(&parsed_games);
        traits.extend(analyze_timeline_traits(&parsed_games, &main_role));
        println!("✅ 时间线分析：识别对线期和发育曲线特征");

        // ⭐ 第4.6层：基于frames的深度时间线分析（新增）
        if let Some(timeline_analysis) = analyze_frames_timeline_data(games, current_puuid) {
            traits.extend(generate_frames_based_traits(&timeline_analysis, &main_role));
            println!("✅ Frames时间线分析：深度分析对手差距和关键事件");
        }
    }

    // 第5层：胜负模式（所有模式都执行，但其他模式会简化）
    traits.extend(analyze_win_loss_pattern(&player_stats.recent_performance));

    // === 第5步: 智能去重优化 ===
    // 根据策略限制特征数量
    let max_traits = strategy.max_traits();
    let mut optimized_traits = optimize_traits(traits);
    optimized_traits.truncate(max_traits);
    player_stats.traits = optimized_traits;

    // === 第6步: 生成智能建议（仅排位模式）⭐ v3.0 ===
    if matches!(strategy, AnalysisStrategy::SoloRanked | AnalysisStrategy::FlexRanked | AnalysisStrategy::MixedRanked) {
        let main_role = identify_main_role(&parsed_games);
        player_stats.advice = generate_advice(
            &player_stats,
            &parsed_games,
            &main_role,
            advice_perspective, // 使用传入的视角 ⭐
            target_player_name, // 使用传入的目标名称 ⭐
            &strategy,
        );
        println!(
            "💡 建议生成：共 {} 条建议（视角：{:?}）",
            player_stats.advice.len(),
            advice_perspective
        );

        // ⭐ 第6.1步: 集成时间线分析建议（新增）
        if let Some(timeline_analysis) = analyze_frames_timeline_data(games, current_puuid) {
            let timeline_advice = generate_timeline_enhanced_advice(&timeline_analysis, &main_role);
            player_stats.advice.extend(timeline_advice);
            println!("✅ 时间线建议：生成基于frames数据的深度建议");
        }
    }

    println!("✅ 分析完成 ({:?}):", strategy);
    println!(
        "   总对局={}, 胜场={}, 胜率={:.1}%",
        player_stats.total_games, player_stats.wins, player_stats.win_rate
    );
    println!("   今日对局: {}/{}", player_stats.today_wins, player_stats.today_games);
    println!("   识别特征: {}个 (限制{}个)", player_stats.traits.len(), max_traits);
    println!("   智能建议: {}条", player_stats.advice.len());

    Ok(player_stats)
}

/// 获取指定玩家的战术建议
///
/// 用于针对敌人或协作队友生成建议
pub async fn get_player_tactical_advice(
    client: &Client,
    summoner_name: String,
    perspective: AdvicePerspective,
    target_role: Option<String>,
) -> Result<Vec<GameAdvice>, String> {
    println!("🎯 开始获取玩家战术建议");
    println!("   召唤师: {}", summoner_name);
    println!("   视角: {:?}", perspective);

    // 1. 根据召唤师名称获取PUUID
    let summoner_url = format!(
        "/lol-summoner/v1/summoners?name={}",
        utf8_percent_encode(&summoner_name, NON_ALPHANUMERIC)
    );

    let summoner_response: Value = lcu_request_json(&client, Method::GET, &summoner_url, None)
        .await
        .map_err(|e| format!("获取召唤师信息失败: {}", e))?;

    let puuid = summoner_response["puuid"]
        .as_str()
        .ok_or("无法获取玩家PUUID")?
        .to_string();

    println!("✅ 获取到PUUID: {}", puuid);

    // 2. 获取玩家战绩（最近20场排位）
    let match_history_url = format!(
        "/lol-match-history/v1/products/lol/{}/matches?begIndex=0&endIndex=20",
        puuid
    );

    let match_list_response: Value = lcu_request_json(&client, Method::GET, &match_history_url, None)
        .await
        .map_err(|e| format!("获取战绩失败: {}", e))?;

    let match_list_data = match_list_response["games"]["games"]
        .as_array()
        .ok_or("无法解析战绩数据")?;

    println!("✅ 获取到{}场对局", match_list_data.len());

    if match_list_data.is_empty() {
        return Ok(Vec::new());
    }

    // 3. 解析战绩数据
    let parsed_games = parse_games(match_list_data, &puuid);
    println!("✅ 解析完成：{} 场对局", parsed_games.len());

    // 4. 确定分析策略（默认使用排位策略，因为我们需要深度分析）
    let strategy = AnalysisStrategy::SoloRanked;

    // 5. 分析玩家数据
    let context = AnalysisContext::new().with_queue_id(420); // 排位模式
    let player_stats = analyze_player_stats(&parsed_games, &puuid, context);

    // 6. 识别主要位置
    let main_role = target_role.unwrap_or_else(|| identify_main_role(&parsed_games));
    println!("✅ 主要位置: {}", main_role);

    // 7. 生成建议（使用指定的视角）
    let advice = generate_advice(
        &player_stats,
        &parsed_games,
        &main_role,
        perspective,
        Some(summoner_name.clone()), // 传递目标玩家名称
        &strategy,
    );

    println!("💡 生成建议：共 {} 条", advice.len());

    Ok(advice)
}

        .and_then(|games_obj| games_obj.get("games"))
        .and_then(|g| g.as_array())
        .unwrap_or(&empty_games);

    println!("📊 找到 {} 场对局记录", games.len());

    // === 第1步: Parser 层 - 解析原始数据为统一格式 ===
    let parsed_games = parse_games(games, current_puuid);
    println!("✅ Parser: 解析了 {} 场对局数据", parsed_games.len());

    // === 第2步: Strategy 层 - 根据队列选择分析策略 ===
    let strategy = if let Some(qid) = queue_id {
        let strategy = AnalysisStrategy::from_queue_id(qid as i64);
        println!("🎯 Strategy: {} (queueId={})", strategy.description(), qid);
        strategy
    } else {
        let strategy = AnalysisStrategy::from_games(&parsed_games);
        println!("🎯 Strategy: {} (自动推断)", strategy.description());
        strategy
    };
    println!("📊 分析方案: {}", strategy.analysis_layers());

    // === 第3步: Analyzer 层 - 使用解析后的数据进行统计计算 ===
    let mut context = AnalysisContext::new();

    // 根据队列ID设置分析上下文（精确匹配）
    if let Some(qid) = queue_id {
        context = context.with_queue_id(qid);
    }

    let mut player_stats = analyze_player_stats(&parsed_games, current_puuid, context);

    // === 第4步: 多层次特征分析（根据策略决定分析深度）===
    let mut traits = Vec::new();

    // 第1层：基础特征（所有模式都执行）
    traits.extend(analyze_traits(&player_stats));

    // 第1.5层：队列感知特征（所有模式都执行，提供队列特定的评估）
    traits.extend(crate::domains::analysis::analyzers::traits::queue_aware::analyze_queue_aware_traits(
        &player_stats,
        queue_id,
    ));

    // 第2-5层：深度分析（仅排位模式）
    if strategy.enable_advanced_analysis() {
        // 第2层：深度特征（参团率、伤害占比、稳定性、趋势）
        traits.extend(analyze_advanced_traits(&player_stats, games, current_puuid));
    }

    if strategy.enable_role_analysis() {
        // 第3层：位置特征（基于位置的专项分析）
        let role_stats_map = identify_player_roles(games, current_puuid);
        traits.extend(analyze_role_based_traits(&player_stats, &role_stats_map));
    }

    if strategy.enable_distribution_analysis() {
        // 第4层：分布特征（高光时刻、崩盘场次、稳定性）
        traits.extend(analyze_distribution_traits(&player_stats.recent_performance));

        // ⭐ 第4.5层：时间线特征（对线压制、稳定发育、爆发成长）NEW
        let main_role = identify_main_role(&parsed_games);
        traits.extend(analyze_timeline_traits(&parsed_games, &main_role));
        println!("✅ 时间线分析：识别对线期和发育曲线特征");

        // ⭐ 第4.6层：基于frames的深度时间线分析（新增）
        if let Some(timeline_analysis) = analyze_frames_timeline_data(games, current_puuid) {
            traits.extend(generate_frames_based_traits(&timeline_analysis, &main_role));
            println!("✅ Frames时间线分析：深度分析对手差距和关键事件");
        }
    }

    // 第5层：胜负模式（所有模式都执行，但其他模式会简化）
    traits.extend(analyze_win_loss_pattern(&player_stats.recent_performance));

    // === 第5步: 智能去重优化 ===
    // 根据策略限制特征数量
    let max_traits = strategy.max_traits();
    let mut optimized_traits = optimize_traits(traits);
    optimized_traits.truncate(max_traits);
    player_stats.traits = optimized_traits;

    // === 第6步: 生成智能建议（仅排位模式）⭐ v3.0 ===
    if matches!(strategy, AnalysisStrategy::SoloRanked | AnalysisStrategy::FlexRanked | AnalysisStrategy::MixedRanked) {
        let main_role = identify_main_role(&parsed_games);
        player_stats.advice = generate_advice(
            &player_stats,
            &parsed_games,
            &main_role,
            advice_perspective, // 使用传入的视角 ⭐
            target_player_name, // 使用传入的目标名称 ⭐
            &strategy,
        );
        println!(
            "💡 建议生成：共 {} 条建议（视角：{:?}）",
            player_stats.advice.len(),
            advice_perspective
        );

        // ⭐ 第6.1步: 集成时间线分析建议（新增）
        if let Some(timeline_analysis) = analyze_frames_timeline_data(games, current_puuid) {
            let timeline_advice = generate_timeline_enhanced_advice(&timeline_analysis, &main_role);
            player_stats.advice.extend(timeline_advice);
            println!("✅ 时间线建议：生成基于frames数据的深度建议");
        }
    }

    println!("✅ 分析完成 ({:?}):", strategy);
    println!(
        "   总对局={}, 胜场={}, 胜率={:.1}%",
        player_stats.total_games, player_stats.wins, player_stats.win_rate
    );
    println!("   今日对局: {}/{}", player_stats.today_wins, player_stats.today_games);
    println!("   识别特征: {}个 (限制{}个)", player_stats.traits.len(), max_traits);
    println!("   智能建议: {}条", player_stats.advice.len());

    Ok(player_stats)
}

/// 获取指定玩家的战术建议
///
/// 用于针对敌人或协作队友生成建议
pub async fn get_player_tactical_advice(
    client: &Client,
    summoner_name: String,
    perspective: AdvicePerspective,
    target_role: Option<String>,
) -> Result<Vec<GameAdvice>, String> {
    println!("🎯 开始获取玩家战术建议");
    println!("   召唤师: {}", summoner_name);
    println!("   视角: {:?}", perspective);

    // 1. 根据召唤师名称获取PUUID
    let summoner_url = format!(
        "/lol-summoner/v1/summoners?name={}",
        utf8_percent_encode(&summoner_name, NON_ALPHANUMERIC)
    );

    let summoner_response: Value = lcu_request_json(&client, Method::GET, &summoner_url, None)
        .await
        .map_err(|e| format!("获取召唤师信息失败: {}", e))?;

    let puuid = summoner_response["puuid"]
        .as_str()
        .ok_or("无法获取玩家PUUID")?
        .to_string();

    println!("✅ 获取到PUUID: {}", puuid);

    // 2. 获取玩家战绩（最近20场排位）
    let match_history_url = format!(
        "/lol-match-history/v1/products/lol/{}/matches?begIndex=0&endIndex=20",
        puuid
    );

    let match_list_response: Value = lcu_request_json(&client, Method::GET, &match_history_url, None)
        .await
        .map_err(|e| format!("获取战绩失败: {}", e))?;

    let match_list_data = match_list_response["games"]["games"]
        .as_array()
        .ok_or("无法解析战绩数据")?;

    println!("✅ 获取到{}场对局", match_list_data.len());

    if match_list_data.is_empty() {
        return Ok(Vec::new());
    }

    // 3. 解析战绩数据
    let parsed_games = parse_games(match_list_data, &puuid);
    println!("✅ 解析完成：{} 场对局", parsed_games.len());

    // 4. 确定分析策略（默认使用排位策略，因为我们需要深度分析）
    let strategy = AnalysisStrategy::SoloRanked;

    // 5. 分析玩家数据
    let context = AnalysisContext::new().with_queue_id(420); // 排位模式
    let player_stats = analyze_player_stats(&parsed_games, &puuid, context);

    // 6. 识别主要位置
    let main_role = target_role.unwrap_or_else(|| identify_main_role(&parsed_games));
    println!("✅ 主要位置: {}", main_role);

    // 7. 生成建议（使用指定的视角）
    let advice = generate_advice(
        &player_stats,
        &parsed_games,
        &main_role,
        perspective,
        Some(summoner_name.clone()), // 传递目标玩家名称
        &strategy,
    );

    println!("💡 生成建议：共 {} 条", advice.len());

    Ok(advice)
}
