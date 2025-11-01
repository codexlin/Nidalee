/// 时间线分析器 - 基于frames数据的深度分析
///
/// 职责：
/// - 解析match_timeline_json中的frames数据
/// - 计算分阶段效率指标
/// - 分析对手差距
/// - 提取关键事件
/// - 生成时间线特征
use serde_json::Value;
use std::collections::HashMap;

/// 时间线帧数据结构
#[derive(Debug, Clone)]
pub struct TimelineFrame {
    pub timestamp: i64,  // 时间戳（毫秒）
    pub events: Vec<GameEvent>,
    pub participant_frames: HashMap<String, ParticipantFrame>,
}

/// 游戏事件
#[derive(Debug, Clone)]
pub struct GameEvent {
    pub event_type: String,
    pub timestamp: i64,
    pub killer_id: Option<i32>,
    pub victim_id: Option<i32>,
    pub assisting_participant_ids: Vec<i32>,
    pub position: Option<Position>,
    pub monster_type: Option<String>,
    pub monster_sub_type: Option<String>,
}

/// 参与者帧数据
#[derive(Debug, Clone)]
pub struct ParticipantFrame {
    pub participant_id: i32,
    pub current_gold: i32,
    pub total_gold: i32,
    pub level: i32,
    pub xp: i32,
    pub minions_killed: i32,
    pub jungle_minions_killed: i32,
    pub position: Position,
}

/// 位置信息
#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// 阶段分析结果
#[derive(Debug, Clone)]
pub struct PhaseAnalysis {
    pub cs_per_minute: f64,
    pub gold_per_minute: f64,
    pub xp_per_minute: f64,
    pub cs_difference: f64,
    pub xp_difference: f64,
    pub gold_difference: f64,
    pub level_difference: f64,
}

/// 关键事件
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub event_type: String,
    pub timestamp: i64,
    pub participant_id: i32,
    pub importance_score: f64,
    pub description: String,
}

/// 对手比较
#[derive(Debug, Clone)]
pub struct OpponentComparison {
    pub opponent_id: i32,
    pub cs_advantage: f64,
    pub xp_advantage: f64,
    pub gold_advantage: f64,
    pub level_advantage: f64,
    pub overall_advantage: f64,
}

/// 时间线分析结果
#[derive(Debug, Clone)]
pub struct TimelineAnalysis {
    // 对线期数据 (0-10分钟)
    pub early_game: PhaseAnalysis,

    // 中期数据 (10-20分钟)
    pub mid_game: PhaseAnalysis,

    // 后期数据 (20分钟+)
    pub late_game: PhaseAnalysis,

    // 关键事件
    pub key_events: Vec<KeyEvent>,

    // 对手分析
    pub opponent_comparison: OpponentComparison,
}

/// 解析时间线数据
pub fn parse_timeline_data(
    timeline_json: &Value,
    target_participant_id: i32,
    opponent_id: Option<i32>,
) -> Option<TimelineAnalysis> {
    let frames = timeline_json.get("frames")?.as_array()?;

    if frames.is_empty() {
        return None;
    }

    // 按时间分组
    let (early_frames, mid_frames, late_frames) = group_frames_by_time(frames);

    // 分析各个阶段
    let early_game = analyze_phase(&early_frames, target_participant_id, opponent_id)?;
    let mid_game = analyze_phase(&mid_frames, target_participant_id, opponent_id)?;
    let late_game = analyze_phase(&late_frames, target_participant_id, opponent_id)?;

    // 提取关键事件
    let key_events = extract_key_events(frames, target_participant_id);

    // 分析对手比较
    let opponent_comparison = analyze_opponent_comparison(
        frames,
        target_participant_id,
        opponent_id.unwrap_or(0),
    );

    Some(TimelineAnalysis {
        early_game,
        mid_game,
        late_game,
        key_events,
        opponent_comparison,
    })
}

/// 按时间分组frames
fn group_frames_by_time(frames: &[Value]) -> (Vec<&Value>, Vec<&Value>, Vec<&Value>) {
    let mut early_frames = Vec::new();
    let mut mid_frames = Vec::new();
    let mut late_frames = Vec::new();

    for frame in frames {
        let timestamp = frame["timestamp"].as_i64().unwrap_or(0);
        let minutes = timestamp / 60000; // 转换为分钟

        if minutes < 10 {
            early_frames.push(frame);
        } else if minutes < 20 {
            mid_frames.push(frame);
        } else {
            late_frames.push(frame);
        }
    }

    (early_frames, mid_frames, late_frames)
}

/// 分析单个阶段
fn analyze_phase(
    frames: &[&Value],
    target_participant_id: i32,
    opponent_id: Option<i32>,
) -> Option<PhaseAnalysis> {
    if frames.is_empty() {
        return None;
    }

    let first_frame = frames.first()?;
    let last_frame = frames.last()?;

    // 获取目标玩家的数据
    let target_key = format!("{}", target_participant_id);
    let first_target = first_frame["participantFrames"][&target_key].as_object()?;
    let last_target = last_frame["participantFrames"][&target_key].as_object()?;

    // 计算效率指标
    let cs_per_minute = calculate_average_cs_per_minute(&Value::Object(first_target.clone()), &Value::Object(last_target.clone()), frames.len());
    let gold_per_minute = calculate_average_gold_per_minute(&Value::Object(first_target.clone()), &Value::Object(last_target.clone()), frames.len());
    let xp_per_minute = calculate_average_xp_per_minute(&Value::Object(first_target.clone()), &Value::Object(last_target.clone()), frames.len());

    // 计算对手差距
    let (cs_difference, xp_difference, gold_difference, level_difference) =
        if let Some(opp_id) = opponent_id {
            calculate_opponent_differences(&Value::Object(last_target.clone()), &format!("{}", opp_id), last_frame)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

    Some(PhaseAnalysis {
        cs_per_minute,
        gold_per_minute,
        xp_per_minute,
        cs_difference,
        xp_difference,
        gold_difference,
        level_difference,
    })
}

/// 计算平均每分钟补刀
fn calculate_average_cs_per_minute(first: &Value, last: &Value, frame_count: usize) -> f64 {
    let first_cs = first["minionsKilled"].as_i64().unwrap_or(0);
    let last_cs = last["minionsKilled"].as_i64().unwrap_or(0);

    if frame_count > 0 {
        (last_cs - first_cs) as f64 / frame_count as f64
    } else {
        0.0
    }
}

/// 计算平均每分钟金币
fn calculate_average_gold_per_minute(first: &Value, last: &Value, frame_count: usize) -> f64 {
    let first_gold = first["totalGold"].as_i64().unwrap_or(0);
    let last_gold = last["totalGold"].as_i64().unwrap_or(0);

    if frame_count > 0 {
        (last_gold - first_gold) as f64 / frame_count as f64
    } else {
        0.0
    }
}

/// 计算平均每分钟经验
fn calculate_average_xp_per_minute(first: &Value, last: &Value, frame_count: usize) -> f64 {
    let first_xp = first["xp"].as_i64().unwrap_or(0);
    let last_xp = last["xp"].as_i64().unwrap_or(0);

    if frame_count > 0 {
        (last_xp - first_xp) as f64 / frame_count as f64
    } else {
        0.0
    }
}

/// 计算对手差距
fn calculate_opponent_differences(
    target: &Value,
    opponent_key: &str,
    frame: &Value,
) -> (f64, f64, f64, f64) {
    let opponent = frame["participantFrames"][opponent_key].as_object();

    if let Some(opp) = opponent {
        let cs_diff = target["minionsKilled"].as_i64().unwrap_or(0) -
                     opp["minionsKilled"].as_i64().unwrap_or(0);
        let xp_diff = target["xp"].as_i64().unwrap_or(0) -
                     opp["xp"].as_i64().unwrap_or(0);
        let gold_diff = target["totalGold"].as_i64().unwrap_or(0) -
                       opp["totalGold"].as_i64().unwrap_or(0);
        let level_diff = target["level"].as_i64().unwrap_or(0) -
                        opp["level"].as_i64().unwrap_or(0);

        (cs_diff as f64, xp_diff as f64, gold_diff as f64, level_diff as f64)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    }
}

/// 提取关键事件
fn extract_key_events(frames: &[Value], target_participant_id: i32) -> Vec<KeyEvent> {
    let mut events = Vec::new();

    for frame in frames {
        if let Some(frame_events) = frame.get("events").and_then(|e| e.as_array()) {
            for event in frame_events {
                if let Some(key_event) = analyze_event(event, target_participant_id) {
                    events.push(key_event);
                }
            }
        }
    }

    events
}

/// 分析单个事件
fn analyze_event(event: &Value, target_participant_id: i32) -> Option<KeyEvent> {
    let event_type = event["type"].as_str()?.to_string();
    let timestamp = event["timestamp"].as_i64().unwrap_or(0);

    let mut importance_score = 0.0;
    let mut description = String::new();
    let mut participant_id = target_participant_id;

    match event_type.as_str() {
        "CHAMPION_KILL" => {
            if event["killerId"].as_i64().unwrap_or(0) as i32 == target_participant_id {
                importance_score = 10.0;
                description = "击杀敌方英雄".to_string();
                participant_id = event["killerId"].as_i64().unwrap_or(0) as i32;
            } else if event["victimId"].as_i64().unwrap_or(0) as i32 == target_participant_id {
                importance_score = -8.0;
                description = "被敌方击杀".to_string();
                participant_id = event["victimId"].as_i64().unwrap_or(0) as i32;
            }
        },
        "ELITE_MONSTER_KILL" => {
            if event["killerId"].as_i64().unwrap_or(0) as i32 == target_participant_id {
                importance_score = 7.0;
                description = "击杀大型野怪".to_string();
                participant_id = event["killerId"].as_i64().unwrap_or(0) as i32;
            }
        },
        "BUILDING_KILL" => {
            if event["killerId"].as_i64().unwrap_or(0) as i32 == target_participant_id {
                importance_score = 5.0;
                description = "摧毁建筑".to_string();
                participant_id = event["killerId"].as_i64().unwrap_or(0) as i32;
            }
        },
        _ => return None,
    }

    if importance_score != 0.0 {
        Some(KeyEvent {
            event_type,
            timestamp,
            participant_id,
            importance_score,
            description,
        })
    } else {
        None
    }
}

/// 分析对手比较
fn analyze_opponent_comparison(
    frames: &[Value],
    target_participant_id: i32,
    opponent_id: i32,
) -> OpponentComparison {
    if frames.is_empty() || opponent_id == 0 {
        return OpponentComparison {
            opponent_id: 0,
            cs_advantage: 0.0,
            xp_advantage: 0.0,
            gold_advantage: 0.0,
            level_advantage: 0.0,
            overall_advantage: 0.0,
        };
    }

    let last_frame = frames.last().unwrap();
    let target_key = format!("{}", target_participant_id);
    let opponent_key = format!("{}", opponent_id);

    let target = &last_frame["participantFrames"][&target_key];
    let opponent = &last_frame["participantFrames"][&opponent_key];

    let cs_advantage = target["minionsKilled"].as_i64().unwrap_or(0) -
                      opponent["minionsKilled"].as_i64().unwrap_or(0);
    let xp_advantage = target["xp"].as_i64().unwrap_or(0) -
                      opponent["xp"].as_i64().unwrap_or(0);
    let gold_advantage = target["totalGold"].as_i64().unwrap_or(0) -
                       opponent["totalGold"].as_i64().unwrap_or(0);
    let level_advantage = target["level"].as_i64().unwrap_or(0) -
                         opponent["level"].as_i64().unwrap_or(0);

    let overall_advantage = (cs_advantage as f64 * 0.3) +
                          (xp_advantage as f64 * 0.3) +
                          (gold_advantage as f64 * 0.2) +
                          (level_advantage as f64 * 0.2);

    OpponentComparison {
        opponent_id,
        cs_advantage: cs_advantage as f64,
        xp_advantage: xp_advantage as f64,
        gold_advantage: gold_advantage as f64,
        level_advantage: level_advantage as f64,
        overall_advantage,
    }
}
