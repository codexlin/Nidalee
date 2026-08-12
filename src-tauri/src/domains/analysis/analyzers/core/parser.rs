/// 数据解析器 - 将LCU API原始数据解析为统一格式
///
/// 职责：
/// - 解析LCU API的原始JSON数据
/// - 提取关键统计信息
/// - 计算衍生指标（KDA、CS等）
/// - 识别位置信息
/// - 集成时间线数据解析
use serde_json::Value;

/// 时间线数据（分阶段统计）
#[derive(Debug, Clone, Default)]
pub struct TimelineData {
    // 对线期 (0-10分钟)
    pub cs_per_min_0_10: Option<f64>,           // 每分钟补刀
    pub gold_per_min_0_10: Option<f64>,         // 每分钟金币
    pub xp_per_min_0_10: Option<f64>,           // 每分钟经验
    pub cs_diff_0_10: Option<f64>,              // ⭐ 补刀差（相对对手）
    pub xp_diff_0_10: Option<f64>,              // ⭐ 经验差（相对对手）
    pub damage_taken_per_min_0_10: Option<f64>, // 每分钟承伤

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

/// 解析后的玩家数据
#[derive(Debug, Clone)]
pub struct ParsedPlayerData {
    // 基础信息
    pub participant_id: i32,
    pub champion_id: i32,
    pub team_id: i32,

    // 游戏结果
    pub win: bool,

    // KDA数据
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub kda: f64,

    // 伤害数据
    pub damage_to_champions: i32,
    pub damage_taken: i32,

    // 经济数据
    pub gold_earned: i32,

    // 视野数据
    pub vision_score: i32,
    pub wards_placed: i32,
    pub wards_killed: i32,

    // 补刀数据
    pub cs: i32,
    pub jungle_cs: i32,

    // 位置信息
    pub role: String,
    pub lane: String,

    // 时间线数据（新增）
    pub timeline_data: Option<TimelineData>,
}

/// 解析后的队伍数据
#[derive(Debug, Clone)]
pub struct ParsedTeamData {
    pub team_id: i32,
    pub total_kills: i32,
    pub total_deaths: i32,
    pub total_assists: i32,
    pub total_damage_to_champions: i32,
    pub total_damage_taken: i32,
    pub total_gold_earned: i32,
    pub total_vision_score: i32,
    pub total_cs: i32,
}

/// 解析后的游戏数据
#[derive(Debug, Clone)]
pub struct ParsedGame {
    pub game_id: u64,
    pub queue_id: i64,
    pub game_duration: i32,
    pub game_creation: i64,
    pub player_data: ParsedPlayerData,
    pub team_data: ParsedTeamData,
}

/// 解析时间线数据（分阶段统计）
fn parse_timeline_data(timeline: &Value) -> Option<TimelineData> {
    let mut data = TimelineData::default();

    // 解析 creepsPerMinDeltas
    if let Some(cs_deltas) = timeline.get("creepsPerMinDeltas") {
        data.cs_per_min_0_10 = parse_delta_value(cs_deltas, "0-10");
        data.cs_per_min_10_20 = parse_delta_value(cs_deltas, "10-20");
        data.cs_per_min_20_end =
            parse_delta_value(cs_deltas, "20-30").or_else(|| parse_delta_value(cs_deltas, "20-end"));
    }

    // 解析 goldPerMinDeltas
    if let Some(gold_deltas) = timeline.get("goldPerMinDeltas") {
        data.gold_per_min_0_10 = parse_delta_value(gold_deltas, "0-10");
        data.gold_per_min_10_20 = parse_delta_value(gold_deltas, "10-20");
        data.gold_per_min_20_end =
            parse_delta_value(gold_deltas, "20-30").or_else(|| parse_delta_value(gold_deltas, "20-end"));
    }

    // 解析 xpPerMinDeltas
    if let Some(xp_deltas) = timeline.get("xpPerMinDeltas") {
        data.xp_per_min_0_10 = parse_delta_value(xp_deltas, "0-10");
        data.xp_per_min_10_20 = parse_delta_value(xp_deltas, "10-20");
    }

    // 解析 csDiffPerMinDeltas ⭐ 关键
    if let Some(cs_diff) = timeline.get("csDiffPerMinDeltas") {
        data.cs_diff_0_10 = parse_delta_value(cs_diff, "0-10");
        data.cs_diff_10_20 = parse_delta_value(cs_diff, "10-20");
        data.cs_diff_20_end = parse_delta_value(cs_diff, "20-30").or_else(|| parse_delta_value(cs_diff, "20-end"));
    }

    // 解析 xpDiffPerMinDeltas ⭐ 关键
    if let Some(xp_diff) = timeline.get("xpDiffPerMinDeltas") {
        data.xp_diff_0_10 = parse_delta_value(xp_diff, "0-10");
        data.xp_diff_10_20 = parse_delta_value(xp_diff, "10-20");
    }

    // 解析 damageTakenPerMinDeltas
    if let Some(damage_deltas) = timeline.get("damageTakenPerMinDeltas") {
        data.damage_taken_per_min_0_10 = parse_delta_value(damage_deltas, "0-10");
        data.damage_taken_per_min_10_20 = parse_delta_value(damage_deltas, "10-20");
    }

    Some(data)
}

/// 解析 delta 对象中的单个值
fn parse_delta_value(deltas: &Value, key: &str) -> Option<f64> {
    deltas.get(key)?.as_f64()
}

/// 解析玩家数据
fn parse_player_data(participant: &Value) -> Option<ParsedPlayerData> {
    let stats = participant.get("stats")?;
    let timeline = participant.get("timeline")?;

    let participant_id = participant["participantId"].as_i64().unwrap_or(0) as i32;
    let champion_id = participant["championId"].as_i64().unwrap_or(0) as i32;
    let team_id = participant["teamId"].as_i64().unwrap_or(0) as i32;

    let win = stats["win"].as_bool().unwrap_or(false);

    let kills = stats["kills"].as_i64().unwrap_or(0) as i32;
    let deaths = stats["deaths"].as_i64().unwrap_or(0) as i32;
    let assists = stats["assists"].as_i64().unwrap_or(0) as i32;

    let kda = if deaths > 0 {
        (kills + assists) as f64 / deaths as f64
    } else {
        (kills + assists) as f64
    };

    let damage_to_champions = stats["totalDamageDealtToChampions"].as_i64().unwrap_or(0) as i32;
    let damage_taken = stats["totalDamageTaken"].as_i64().unwrap_or(0) as i32;
    let gold_earned = stats["goldEarned"].as_i64().unwrap_or(0) as i32;

    let vision_score = stats["visionScore"].as_i64().unwrap_or(0) as i32;
    let wards_placed = stats["wardsPlaced"].as_i64().unwrap_or(0) as i32;
    let wards_killed = stats["wardsKilled"].as_i64().unwrap_or(0) as i32;

    let cs = stats["totalMinionsKilled"].as_i64().unwrap_or(0) as i32;
    let jungle_cs = stats["neutralMinionsKilled"].as_i64().unwrap_or(0) as i32;

    let role = timeline["role"].as_str().unwrap_or("NONE").to_string();
    let lane = timeline["lane"].as_str().unwrap_or("NONE").to_string();

    // 解析时间线数据
    let timeline_data = parse_timeline_data(timeline);

    Some(ParsedPlayerData {
        participant_id,
        champion_id,
        team_id,
        win,
        kills,
        deaths,
        assists,
        kda,
        damage_to_champions,
        damage_taken,
        gold_earned,
        vision_score,
        wards_placed,
        wards_killed,
        cs,
        jungle_cs,
        role,
        lane,
        timeline_data,
    })
}

/// 解析队伍数据
fn parse_team_data(game: &Value, team_id: i32) -> Option<ParsedTeamData> {
    let participants = game.get("participants")?.as_array()?;

    let mut total_kills = 0;
    let mut total_deaths = 0;
    let mut total_assists = 0;
    let mut total_damage_to_champions = 0;
    let mut total_damage_taken = 0;
    let mut total_gold_earned = 0;
    let mut total_vision_score = 0;
    let mut total_cs = 0;

    for participant in participants {
        if participant["teamId"].as_i64().unwrap_or(0) as i32 == team_id {
            if let Some(stats) = participant.get("stats") {
                total_kills += stats["kills"].as_i64().unwrap_or(0) as i32;
                total_deaths += stats["deaths"].as_i64().unwrap_or(0) as i32;
                total_assists += stats["assists"].as_i64().unwrap_or(0) as i32;
                total_damage_to_champions += stats["totalDamageDealtToChampions"].as_i64().unwrap_or(0) as i32;
                total_damage_taken += stats["totalDamageTaken"].as_i64().unwrap_or(0) as i32;
                total_gold_earned += stats["goldEarned"].as_i64().unwrap_or(0) as i32;
                total_vision_score += stats["visionScore"].as_i64().unwrap_or(0) as i32;
                total_cs += stats["totalMinionsKilled"].as_i64().unwrap_or(0) as i32;
            }
        }
    }

    Some(ParsedTeamData {
        team_id,
        total_kills,
        total_deaths,
        total_assists,
        total_damage_to_champions,
        total_damage_taken,
        total_gold_earned,
        total_vision_score,
        total_cs,
    })
}

/// 解析游戏数据
fn parse_game(game: &Value, target_puuid: &str) -> Option<ParsedGame> {
    // 查找目标玩家的participant_id
    let participant_identities = game.get("participantIdentities")?.as_array()?;
    let target_participant_id = participant_identities
        .iter()
        .find(|p| p["player"]["puuid"].as_str() == Some(target_puuid))?["participantId"]
        .as_i64()
        .unwrap_or(0) as i32;

    // 查找目标玩家的数据
    let participants = game.get("participants")?.as_array()?;
    let target_participant = participants
        .iter()
        .find(|p| p["participantId"].as_i64().unwrap_or(0) as i32 == target_participant_id)?;

    let player_data = parse_player_data(target_participant)?;
    let team_data = parse_team_data(game, player_data.team_id)?;

    Some(ParsedGame {
        game_id: game["gameId"].as_u64().unwrap_or(0),
        queue_id: game["queueId"].as_i64().unwrap_or(0),
        game_duration: game["gameDuration"].as_i64().unwrap_or(0) as i32,
        game_creation: game["gameCreation"].as_i64().unwrap_or(0),
        player_data,
        team_data,
    })
}

/// 解析游戏列表
pub fn parse_games(games: &[Value], target_puuid: &str) -> Vec<ParsedGame> {
    games.iter().filter_map(|game| parse_game(game, target_puuid)).collect()
}
