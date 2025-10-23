/// 数据解析层
///
/// 职责：
/// - 将 LCU API 的原始 JSON 数据解析为统一的内部数据结构
/// - 隔离 LCU 数据结构变化的影响
/// - 如果 LCU API 变更，只需要修改这个文件
use serde_json::Value;

/// 统一的对局数据结构（内部使用）
#[derive(Debug, Clone)]
pub struct ParsedGame {
    pub game_id: u64,
    pub queue_id: i64,
    pub game_duration: i32,
    pub game_creation: i64,
    pub player_data: ParsedPlayerData,
    pub team_data: ParsedTeamData,
}

/// 时间线数据（分阶段统计）
#[derive(Debug, Clone, Default)]
pub struct TimelineData {
    // 对线期 (0-10分钟)
    pub cs_per_min_0_10: Option<f64>,
    pub gold_per_min_0_10: Option<f64>,
    pub xp_per_min_0_10: Option<f64>,
    pub cs_diff_0_10: Option<f64>, // ⭐ 关键：补刀差
    pub xp_diff_0_10: Option<f64>, // ⭐ 关键：经验差
    pub damage_taken_per_min_0_10: Option<f64>,

    // 发育期 (10-20分钟)
    pub cs_per_min_10_20: Option<f64>,
    pub gold_per_min_10_20: Option<f64>,
    pub xp_per_min_10_20: Option<f64>,
    pub cs_diff_10_20: Option<f64>,
    pub xp_diff_10_20: Option<f64>,
    pub damage_taken_per_min_10_20: Option<f64>,

    // 后期 (20分钟+)
    pub cs_per_min_20_end: Option<f64>,
    pub gold_per_min_20_end: Option<f64>,
    pub cs_diff_20_end: Option<f64>,
}

/// 玩家数据（单场）
#[derive(Debug, Clone)]
pub struct ParsedPlayerData {
    pub win: bool,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub kda: f64,
    pub damage_to_champions: i64,
    pub damage_taken: i64,
    pub gold_earned: i64,
    pub vision_score: i32,
    pub wards_placed: i32,
    pub wards_killed: i32,
    pub cs: i32,
    pub damage_to_objectives: i64,
    pub damage_to_turrets: i64,
    pub time_cc_others: i32,
    pub champion_id: i32,
    pub role: String,
    pub lane: String,
    pub timeline_data: Option<TimelineData>, // ⭐ 新增：时间线数据
}

/// 队伍数据（单场）
#[derive(Debug, Clone)]
pub struct ParsedTeamData {
    pub team_total_kills: i32,
    pub team_total_damage: i64,
    pub team_total_damage_taken: i64,
    pub team_total_vision_score: i32,
}

/// 解析单场游戏数据
///
/// 如果 LCU API 变更，只需要修改这个函数
pub fn parse_game(game: &Value, puuid: &str) -> Option<ParsedGame> {
    // 1. 查找玩家的 participant_id
    let participant_id = game["participantIdentities"]
        .as_array()?
        .iter()
        .find(|id| id["player"]["puuid"].as_str() == Some(puuid))?["participantId"]
        .as_i64()?;

    // 2. 查找玩家的 participant 数据
    let participant = game["participants"]
        .as_array()?
        .iter()
        .find(|p| p["participantId"].as_i64() == Some(participant_id))?;

    // 3. 解析玩家数据
    let player_data = parse_player_data(participant)?;

    // 4. 解析队伍数据
    let team_id = participant["teamId"].as_i64()?;
    let team_data = parse_team_data(game, team_id)?;

    // 5. 解析游戏基础信息
    Some(ParsedGame {
        game_id: game["gameId"].as_u64().unwrap_or(0),
        queue_id: game["queueId"].as_i64().unwrap_or(0),
        game_duration: game["gameDuration"].as_i64().unwrap_or(0) as i32,
        game_creation: game["gameCreation"].as_i64().unwrap_or(0),
        player_data,
        team_data,
    })
}

fn parse_player_data(participant: &Value) -> Option<ParsedPlayerData> {
    let stats = &participant["stats"];
    let timeline = participant.get("timeline");

    let kills = stats["kills"].as_i64().unwrap_or(0) as i32;
    let deaths = stats["deaths"].as_i64().unwrap_or(0) as i32;
    let assists = stats["assists"].as_i64().unwrap_or(0) as i32;

    let kda = if deaths > 0 {
        (kills + assists) as f64 / deaths as f64
    } else {
        (kills + assists) as f64
    };

    // 解析位置信息和时间线数据
    let (role, lane, timeline_data) = if let Some(timeline) = timeline {
        let role = timeline["role"].as_str().unwrap_or("NONE").to_string();
        let lane = timeline["lane"].as_str().unwrap_or("NONE").to_string();
        let timeline_data = parse_timeline_data(timeline); // ⭐ 解析时间线数据
        (role, lane, timeline_data)
    } else {
        ("NONE".to_string(), "NONE".to_string(), None)
    };

    Some(ParsedPlayerData {
        win: stats["win"].as_bool().unwrap_or(false),
        kills,
        deaths,
        assists,
        kda,
        damage_to_champions: stats["totalDamageDealtToChampions"].as_i64().unwrap_or(0),
        damage_taken: stats["totalDamageTaken"].as_i64().unwrap_or(0),
        gold_earned: stats["goldEarned"].as_i64().unwrap_or(0),
        vision_score: stats["visionScore"].as_i64().unwrap_or(0) as i32,
        wards_placed: stats["wardsPlaced"].as_i64().unwrap_or(0) as i32,
        wards_killed: stats["wardsKilled"].as_i64().unwrap_or(0) as i32,
        cs: (stats["totalMinionsKilled"].as_i64().unwrap_or(0) + stats["neutralMinionsKilled"].as_i64().unwrap_or(0))
            as i32,
        damage_to_objectives: stats["damageDealtToObjectives"].as_i64().unwrap_or(0),
        damage_to_turrets: stats["damageDealtToTurrets"].as_i64().unwrap_or(0),
        time_cc_others: stats["timeCCingOthers"].as_i64().unwrap_or(0) as i32,
        champion_id: participant["championId"].as_i64().unwrap_or(0) as i32,
        role,
        lane,
        timeline_data, // ⭐ 新增：时间线数据
    })
}

fn parse_team_data(game: &Value, team_id: i64) -> Option<ParsedTeamData> {
    let participants = game["participants"].as_array()?;

    // 聚合队伍数据
    let mut team_total_kills = 0;
    let mut team_total_damage = 0i64;
    let mut team_total_damage_taken = 0i64;
    let mut team_total_vision_score = 0;

    for p in participants {
        if p["teamId"].as_i64() == Some(team_id) {
            let stats = &p["stats"];
            team_total_kills += stats["kills"].as_i64().unwrap_or(0) as i32;
            team_total_damage += stats["totalDamageDealtToChampions"].as_i64().unwrap_or(0);
            team_total_damage_taken += stats["totalDamageTaken"].as_i64().unwrap_or(0);
            team_total_vision_score += stats["visionScore"].as_i64().unwrap_or(0) as i32;
        }
    }

    Some(ParsedTeamData {
        team_total_kills,
        team_total_damage,
        team_total_damage_taken,
        team_total_vision_score,
    })
}

/// 解析时间线数据（分阶段统计）
fn parse_timeline_data(timeline: &Value) -> Option<TimelineData> {
    let mut data = TimelineData::default();

    // 解析 creepsPerMinDeltas（每分钟补刀）
    if let Some(cs_deltas) = timeline.get("creepsPerMinDeltas") {
        data.cs_per_min_0_10 = parse_delta_value(cs_deltas, "0-10");
        data.cs_per_min_10_20 = parse_delta_value(cs_deltas, "10-20");
        data.cs_per_min_20_end =
            parse_delta_value(cs_deltas, "20-30").or_else(|| parse_delta_value(cs_deltas, "20-end"));
    }

    // 解析 goldPerMinDeltas（每分钟金币）
    if let Some(gold_deltas) = timeline.get("goldPerMinDeltas") {
        data.gold_per_min_0_10 = parse_delta_value(gold_deltas, "0-10");
        data.gold_per_min_10_20 = parse_delta_value(gold_deltas, "10-20");
        data.gold_per_min_20_end =
            parse_delta_value(gold_deltas, "20-30").or_else(|| parse_delta_value(gold_deltas, "20-end"));
    }

    // 解析 xpPerMinDeltas（每分钟经验）
    if let Some(xp_deltas) = timeline.get("xpPerMinDeltas") {
        data.xp_per_min_0_10 = parse_delta_value(xp_deltas, "0-10");
        data.xp_per_min_10_20 = parse_delta_value(xp_deltas, "10-20");
    }

    // ⭐ 关键：解析 csDiffPerMinDeltas（补刀差，相对对手）
    if let Some(cs_diff) = timeline.get("csDiffPerMinDeltas") {
        data.cs_diff_0_10 = parse_delta_value(cs_diff, "0-10");
        data.cs_diff_10_20 = parse_delta_value(cs_diff, "10-20");
        data.cs_diff_20_end = parse_delta_value(cs_diff, "20-30").or_else(|| parse_delta_value(cs_diff, "20-end"));
    }

    // ⭐ 关键：解析 xpDiffPerMinDeltas（经验差，相对对手）
    if let Some(xp_diff) = timeline.get("xpDiffPerMinDeltas") {
        data.xp_diff_0_10 = parse_delta_value(xp_diff, "0-10");
        data.xp_diff_10_20 = parse_delta_value(xp_diff, "10-20");
    }

    // 解析 damageTakenPerMinDeltas（每分钟承伤）
    if let Some(dmg_taken) = timeline.get("damageTakenPerMinDeltas") {
        data.damage_taken_per_min_0_10 = parse_delta_value(dmg_taken, "0-10");
        data.damage_taken_per_min_10_20 = parse_delta_value(dmg_taken, "10-20");
    }

    Some(data)
}

/// 解析 delta 对象中的单个值
fn parse_delta_value(deltas: &Value, key: &str) -> Option<f64> {
    deltas.get(key)?.as_f64()
}

/// 批量解析对局数据
pub fn parse_games(games: &[Value], puuid: &str) -> Vec<ParsedGame> {
    games.iter().filter_map(|game| parse_game(game, puuid)).collect()
}

/// 识别玩家的主要位置
/// ⭐ v3.3: 增强版 - 区分排位和非排位，使用统一的位置识别逻辑
pub fn identify_main_role(parsed_games: &[ParsedGame]) -> String {
    if parsed_games.is_empty() {
        return "未知".to_string();
    }

    let mut role_counts: std::collections::HashMap<String, u32> = std::collections::HashMap::new();

    for game in parsed_games {
        let queue_id = game.queue_id;

        // ⭐ 使用统一的位置识别逻辑
        let role = identify_position_from_game(
            &game.player_data.role,
            &game.player_data.lane,
            queue_id
        );

        *role_counts.entry(role).or_insert(0) += 1;
    }

    // 返回出现次数最多的位置
    role_counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .map(|(role, _)| role)
        .unwrap_or_else(|| "未知".to_string())
}

/// 从role/lane/queue_id识别位置（与stats.rs中的role_to_position保持一致）
/// ⭐ 公开此函数以供service层使用
pub fn identify_position_from_game(role: &str, lane: &str, queue_id: i64) -> String {
    // ⭐ 非排位赛直接返回"灵活"
    if queue_id != 420 && queue_id != 440 {
        return "灵活".to_string();
    }

    // ✅ 排位赛才进行位置识别
    match (role, lane) {
        // 标准位置匹配
        ("DUO_CARRY", _) | ("CARRY", _) => "ADC".to_string(),
        ("DUO_SUPPORT", _) | ("SUPPORT", _) => "辅助".to_string(),
        ("SOLO", "TOP") => "上单".to_string(),
        ("SOLO", "MIDDLE") | ("SOLO", "MID") => "中单".to_string(),
        ("NONE", "JUNGLE") | ("JUNGLE", _) => "打野".to_string(),
        ("DUO", "BOTTOM") => "下路".to_string(),

        // 仅根据 lane 判断
        (_, "TOP") => "上单".to_string(),
        (_, "JUNGLE") => "打野".to_string(),
        (_, "MIDDLE") | (_, "MID") => "中单".to_string(),
        ("SUPPORT", "BOTTOM") => "辅助".to_string(),
        (_, "BOTTOM") => "ADC".to_string(),

        // 排位赛中位置数据缺失
        ("SOLO", "NONE") | ("NONE", "NONE") => {
            // 不在这里打印日志，因为identify_main_role会被多次调用
            "未知".to_string()
        }

        // 其他情况
        _ => "未知".to_string(),
    }
}
