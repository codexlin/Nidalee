/// 时间线数据解析器
///
/// 职责：
/// - 从 frames 数据中提取关键时间线指标
/// - 计算分阶段统计数据（对线期、中期、后期）
/// - 分析对手差距和相对表现
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

/// 阶段分析
#[derive(Debug, Clone)]
pub struct PhaseAnalysis {
    pub duration_minutes: f64,
    pub cs_per_minute: f64,
    pub gold_per_minute: f64,
    pub xp_per_minute: f64,
    pub cs_difference: f64,  // 相对对手的补刀差
    pub xp_difference: f64,  // 相对对手的经验差
    pub gold_difference: f64, // 相对对手的金币差
    pub level_difference: i32, // 相对对手的等级差
}

/// 关键事件
#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub timestamp: i64,
    pub event_type: String,
    pub description: String,
    pub impact_score: f64, // 事件影响分数
}

/// 对手比较
#[derive(Debug, Clone)]
pub struct OpponentComparison {
    pub lane_opponent_id: Option<i32>,
    pub cs_advantage: f64,      // 补刀优势
    pub xp_advantage: f64,      // 经验优势
    pub gold_advantage: f64,    // 金币优势
    pub level_advantage: i32,   // 等级优势
    pub kill_death_ratio: f64,  // 击杀死亡比
}

/// 解析时间线数据
pub fn parse_timeline_data(timeline_json: &Value) -> Option<TimelineAnalysis> {
    let frames = timeline_json.get("frames")?.as_array()?;

    if frames.is_empty() {
        return None;
    }

    // 解析所有帧
    let timeline_frames: Vec<TimelineFrame> = frames
        .iter()
        .filter_map(|frame| parse_frame(frame))
        .collect();

    if timeline_frames.is_empty() {
        return None;
    }

    // 分析时间线
    Some(analyze_timeline(&timeline_frames))
}

/// 解析单个帧
fn parse_frame(frame: &Value) -> Option<TimelineFrame> {
    let timestamp = frame.get("timestamp")?.as_i64()?;
    let events = parse_events(frame.get("events")?);
    let participant_frames = parse_participant_frames(frame.get("participantFrames")?)?;

    Some(TimelineFrame {
        timestamp,
        events,
        participant_frames,
    })
}

/// 解析事件列表
fn parse_events(events: &Value) -> Vec<GameEvent> {
    let empty_vec = vec![];
    let events_array = events.as_array().unwrap_or(&empty_vec);

    events_array.iter().filter_map(|event| {
        Some(GameEvent {
            event_type: event.get("type")?.as_str()?.to_string(),
            timestamp: event.get("timestamp")?.as_i64()?,
            killer_id: event.get("killerId")?.as_i64().map(|v| v as i32),
            victim_id: event.get("victimId")?.as_i64().map(|v| v as i32),
            assisting_participant_ids: event.get("assistingParticipantIds")?
                .as_array()?
                .iter()
                .filter_map(|id| id.as_i64().map(|v| v as i32))
                .collect(),
            position: parse_position(event.get("position")?),
            monster_type: event.get("monsterType")?.as_str().map(|s| s.to_string()),
            monster_sub_type: event.get("monsterSubType")?.as_str().map(|s| s.to_string()),
        })
    }).collect()
}

/// 解析参与者帧数据
fn parse_participant_frames(frames: &Value) -> Option<HashMap<String, ParticipantFrame>> {
    let frames_obj = frames.as_object()?;
    let mut result = HashMap::new();

    for (key, frame) in frames_obj {
        if let Some(participant_frame) = parse_single_participant_frame(frame) {
            result.insert(key.clone(), participant_frame);
        }
    }

    Some(result)
}

/// 解析单个参与者帧
fn parse_single_participant_frame(frame: &Value) -> Option<ParticipantFrame> {
    Some(ParticipantFrame {
        participant_id: frame.get("participantId")?.as_i64()? as i32,
        current_gold: frame.get("currentGold")?.as_i64()? as i32,
        total_gold: frame.get("totalGold")?.as_i64()? as i32,
        level: frame.get("level")?.as_i64()? as i32,
        xp: frame.get("xp")?.as_i64()? as i32,
        minions_killed: frame.get("minionsKilled")?.as_i64()? as i32,
        jungle_minions_killed: frame.get("jungleMinionsKilled")?.as_i64()? as i32,
        position: parse_position(frame.get("position")?)?,
    })
}

/// 解析位置信息
fn parse_position(position: &Value) -> Option<Position> {
    Some(Position {
        x: position.get("x")?.as_f64()?,
        y: position.get("y")?.as_f64()?,
    })
}

/// 分析时间线数据
fn analyze_timeline(frames: &[TimelineFrame]) -> TimelineAnalysis {
    // 按时间分组
    let (early_frames, mid_frames, late_frames) = group_frames_by_time(frames);

    // 分析各阶段
    let early_game = analyze_phase(&early_frames, "对线期");
    let mid_game = analyze_phase(&mid_frames, "中期");
    let late_game = analyze_phase(&late_frames, "后期");

    // 提取关键事件
    let key_events = extract_key_events(frames);

    // 分析对手比较（这里需要知道目标玩家ID，暂时用占位符）
    let opponent_comparison = analyze_opponent_comparison(frames);

    TimelineAnalysis {
        early_game,
        mid_game,
        late_game,
        key_events,
        opponent_comparison,
    }
}

/// 按时间分组帧
fn group_frames_by_time(frames: &[TimelineFrame]) -> (Vec<&TimelineFrame>, Vec<&TimelineFrame>, Vec<&TimelineFrame>) {
    let mut early_frames = Vec::new();
    let mut mid_frames = Vec::new();
    let mut late_frames = Vec::new();

    for frame in frames {
        let minutes = frame.timestamp / 60000; // 转换为分钟

        match minutes {
            0..=10 => early_frames.push(frame),
            11..=20 => mid_frames.push(frame),
            _ => late_frames.push(frame),
        }
    }

    (early_frames, mid_frames, late_frames)
}

/// 分析单个阶段
fn analyze_phase(frames: &[&TimelineFrame], phase_name: &str) -> PhaseAnalysis {
    if frames.is_empty() {
        return PhaseAnalysis {
            duration_minutes: 0.0,
            cs_per_minute: 0.0,
            gold_per_minute: 0.0,
            xp_per_minute: 0.0,
            cs_difference: 0.0,
            xp_difference: 0.0,
            gold_difference: 0.0,
            level_difference: 0,
        };
    }

    let first_frame = frames.first().unwrap();
    let last_frame = frames.last().unwrap();
    let duration_minutes = (last_frame.timestamp - first_frame.timestamp) as f64 / 60000.0;

    // 计算平均值（这里需要指定目标玩家ID，暂时用占位符）
    let cs_per_minute = calculate_average_cs_per_minute(frames);
    let gold_per_minute = calculate_average_gold_per_minute(frames);
    let xp_per_minute = calculate_average_xp_per_minute(frames);

    PhaseAnalysis {
        duration_minutes,
        cs_per_minute,
        gold_per_minute,
        xp_per_minute,
        cs_difference: 0.0, // TODO: 需要实现对手比较逻辑
        xp_difference: 0.0,
        gold_difference: 0.0,
        level_difference: 0,
    }
}

/// 计算平均每分钟补刀
fn calculate_average_cs_per_minute(frames: &[&TimelineFrame]) -> f64 {
    if frames.len() < 2 {
        return 0.0;
    }

    let first_frame = frames.first().unwrap();
    let last_frame = frames.last().unwrap();
    let duration_minutes = (last_frame.timestamp - first_frame.timestamp) as f64 / 60000.0;

    if duration_minutes <= 0.0 {
        return 0.0;
    }

    // 这里需要指定目标玩家ID，暂时返回0
    // TODO: 实现具体的补刀计算逻辑
    0.0
}

/// 计算平均每分钟金币
fn calculate_average_gold_per_minute(frames: &[&TimelineFrame]) -> f64 {
    if frames.len() < 2 {
        return 0.0;
    }

    let first_frame = frames.first().unwrap();
    let last_frame = frames.last().unwrap();
    let duration_minutes = (last_frame.timestamp - first_frame.timestamp) as f64 / 60000.0;

    if duration_minutes <= 0.0 {
        return 0.0;
    }

    // TODO: 实现具体的金币计算逻辑
    0.0
}

/// 计算平均每分钟经验
fn calculate_average_xp_per_minute(frames: &[&TimelineFrame]) -> f64 {
    if frames.len() < 2 {
        return 0.0;
    }

    let first_frame = frames.first().unwrap();
    let last_frame = frames.last().unwrap();
    let duration_minutes = (last_frame.timestamp - first_frame.timestamp) as f64 / 60000.0;

    if duration_minutes <= 0.0 {
        return 0.0;
    }

    // TODO: 实现具体的经验计算逻辑
    0.0
}

/// 提取关键事件
fn extract_key_events(frames: &[TimelineFrame]) -> Vec<KeyEvent> {
    let mut key_events = Vec::new();

    for frame in frames {
        for event in &frame.events {
            if let Some(key_event) = analyze_event(event) {
                key_events.push(key_event);
            }
        }
    }

    // 按影响分数排序
    key_events.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());

    key_events
}

/// 分析单个事件
fn analyze_event(event: &GameEvent) -> Option<KeyEvent> {
    let (description, impact_score) = match event.event_type.as_str() {
        "CHAMPION_KILL" => {
            if let (Some(killer), Some(victim)) = (event.killer_id, event.victim_id) {
                (format!("玩家{}击杀玩家{}", killer, victim), 5.0)
            } else {
                return None;
            }
        },
        "ELITE_MONSTER_KILL" => {
            let default_monster = "未知怪物".to_string();
            let monster_name = event.monster_sub_type.as_ref()
                .or(event.monster_type.as_ref())
                .unwrap_or(&default_monster);
            (format!("击杀{}", monster_name), 4.0)
        },
        "BUILDING_KILL" => {
            ("推塔".to_string(), 3.0)
        },
        _ => return None,
    };

    Some(KeyEvent {
        timestamp: event.timestamp,
        event_type: event.event_type.clone(),
        description,
        impact_score,
    })
}

/// 分析对手比较
fn analyze_opponent_comparison(frames: &[TimelineFrame]) -> OpponentComparison {
    // TODO: 实现对手比较逻辑
    // 需要知道目标玩家ID和对线对手ID
    OpponentComparison {
        lane_opponent_id: None,
        cs_advantage: 0.0,
        xp_advantage: 0.0,
        gold_advantage: 0.0,
        level_advantage: 0,
        kill_death_ratio: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_timeline_data() {
        let timeline_json = json!({
            "frames": [
                {
                    "timestamp": 0,
                    "events": [],
                    "participantFrames": {
                        "1": {
                            "participantId": 1,
                            "currentGold": 500,
                            "totalGold": 500,
                            "level": 1,
                            "xp": 0,
                            "minionsKilled": 0,
                            "jungleMinionsKilled": 0,
                            "position": { "x": 554.0, "y": 581.0 }
                        }
                    }
                }
            ]
        });

        let result = parse_timeline_data(&timeline_json);
        assert!(result.is_some());
    }
}
