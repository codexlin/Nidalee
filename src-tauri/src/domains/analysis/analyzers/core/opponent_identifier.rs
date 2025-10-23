/// 对手识别器
///
/// 职责：
/// - 基于位置信息识别对线对手
/// - 计算对手相对优势
use crate::domains::analysis::analyzers::core::timeline_parser::{TimelineFrame, ParticipantFrame};
use std::collections::HashMap;

/// 对手识别结果
#[derive(Debug, Clone)]
pub struct OpponentMatch {
    pub player_id: i32,
    pub opponent_id: i32,
    pub confidence: f64,  // 置信度 0-1
    pub lane: String,     // 对线路
}

/// 位置信息
#[derive(Debug, Clone)]
struct PositionInfo {
    pub x: f64,
    pub y: f64,
    pub timestamp: i64,
}

/// 对手识别器
pub struct OpponentIdentifier;

impl OpponentIdentifier {
    /// 识别对线对手
    pub fn identify_opponent(
        &self,
        player_id: i32,
        frames: &[TimelineFrame],
    ) -> Option<OpponentMatch> {
        // 1. 提取对线期（前10分钟）的位置数据
        let laning_frames: Vec<_> = frames.iter()
            .filter(|f| f.timestamp < 600000) // 10分钟
            .collect();

        if laning_frames.is_empty() {
            return None;
        }

        // 2. 获取玩家的位置信息
        let player_positions = self.extract_positions(player_id, &laning_frames);
        if player_positions.is_empty() {
            return None;
        }

        // 3. 确定玩家所在的队伍
        let player_team = if player_id <= 5 { 1 } else { 2 };

        // 4. 对每个敌方玩家计算接近程度
        let mut proximity_scores: HashMap<i32, f64> = HashMap::new();

        for opponent_id in self.get_enemy_team(player_team) {
            let opponent_positions = self.extract_positions(opponent_id, &laning_frames);
            if opponent_positions.is_empty() {
                continue;
            }

            // 计算平均距离
            let avg_distance = self.calculate_average_distance(&player_positions, &opponent_positions);
            proximity_scores.insert(opponent_id, avg_distance);
        }

        // 5. 找到最接近的对手（距离最小）
        proximity_scores.iter()
            .min_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(&opponent_id, &distance)| {
                // 计算置信度（距离越小，置信度越高）
                let confidence = self.calculate_confidence(distance);
                let lane = self.identify_lane(&player_positions);

                OpponentMatch {
                    player_id,
                    opponent_id,
                    confidence,
                    lane,
                }
            })
    }

    /// 提取位置信息
    fn extract_positions(
        &self,
        participant_id: i32,
        frames: &[&TimelineFrame],
    ) -> Vec<PositionInfo> {
        frames.iter()
            .filter_map(|frame| {
                let participant_frame = frame.participant_frames.get(&participant_id.to_string())?;
                Some(PositionInfo {
                    x: participant_frame.position.x,
                    y: participant_frame.position.y,
                    timestamp: frame.timestamp,
                })
            })
            .collect()
    }

    /// 获取敌方队伍ID列表
    fn get_enemy_team(&self, player_team: i32) -> Vec<i32> {
        if player_team == 1 {
            vec![6, 7, 8, 9, 10]
        } else {
            vec![1, 2, 3, 4, 5]
        }
    }

    /// 计算平均距离
    fn calculate_average_distance(
        &self,
        player_positions: &[PositionInfo],
        opponent_positions: &[PositionInfo],
    ) -> f64 {
        let mut total_distance = 0.0;
        let mut count = 0;

        for player_pos in player_positions {
            // 找到时间最接近的对手位置
            if let Some(opponent_pos) = self.find_nearest_time_position(player_pos.timestamp, opponent_positions) {
                let distance = self.calculate_distance(
                    player_pos.x, player_pos.y,
                    opponent_pos.x, opponent_pos.y
                );
                total_distance += distance;
                count += 1;
            }
        }

        if count > 0 {
            total_distance / count as f64
        } else {
            f64::MAX
        }
    }

    /// 找到时间最接近的位置
    fn find_nearest_time_position<'a>(
        &self,
        timestamp: i64,
        positions: &'a [PositionInfo],
    ) -> Option<&'a PositionInfo> {
        positions.iter()
            .min_by_key(|pos| (pos.timestamp - timestamp).abs())
    }

    /// 计算两点距离
    fn calculate_distance(&self, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
        ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt()
    }

    /// 计算置信度
    fn calculate_confidence(&self, distance: f64) -> f64 {
        // 距离越小，置信度越高
        // 假设：
        // - 距离 < 2000：高置信度 (0.8-1.0)
        // - 距离 2000-5000：中等置信度 (0.5-0.8)
        // - 距离 > 5000：低置信度 (0-0.5)

        if distance < 2000.0 {
            1.0 - (distance / 2000.0) * 0.2
        } else if distance < 5000.0 {
            0.8 - ((distance - 2000.0) / 3000.0) * 0.3
        } else {
            (0.5 - ((distance - 5000.0) / 5000.0) * 0.5).max(0.0)
        }
    }

    /// 识别对线路
    fn identify_lane(&self, positions: &[PositionInfo]) -> String {
        if positions.is_empty() {
            return "未知".to_string();
        }

        // 计算平均位置
        let avg_x: f64 = positions.iter().map(|p| p.x).sum::<f64>() / positions.len() as f64;
        let avg_y: f64 = positions.iter().map(|p| p.y).sum::<f64>() / positions.len() as f64;

        // 根据位置判断线路
        // 召唤师峡谷地图大小约为 14000 x 14000
        // 左下角 (0,0) - 右上角 (14000, 14000)

        if avg_x < 4000.0 && avg_y < 4000.0 {
            "下路".to_string()
        } else if avg_x > 10000.0 && avg_y > 10000.0 {
            "上路".to_string()
        } else if avg_x > 5000.0 && avg_x < 9000.0 && avg_y > 5000.0 && avg_y < 9000.0 {
            "中路".to_string()
        } else {
            "野区".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::analysis::analyzers::core::timeline_parser::Position;

    #[test]
    fn test_calculate_distance() {
        let identifier = OpponentIdentifier;
        let distance = identifier.calculate_distance(0.0, 0.0, 3.0, 4.0);
        assert!((distance - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_confidence() {
        let identifier = OpponentIdentifier;

        // 高置信度
        assert!(identifier.calculate_confidence(1000.0) > 0.8);

        // 中等置信度
        let mid_confidence = identifier.calculate_confidence(3500.0);
        assert!(mid_confidence > 0.5 && mid_confidence < 0.8);

        // 低置信度
        assert!(identifier.calculate_confidence(7000.0) < 0.5);
    }

    #[test]
    fn test_identify_lane() {
        let identifier = OpponentIdentifier;

        // 下路
        let bot_positions = vec![
            PositionInfo { x: 2000.0, y: 2000.0, timestamp: 0 },
            PositionInfo { x: 2500.0, y: 2500.0, timestamp: 60000 },
        ];
        assert_eq!(identifier.identify_lane(&bot_positions), "下路");

        // 上路
        let top_positions = vec![
            PositionInfo { x: 12000.0, y: 12000.0, timestamp: 0 },
        ];
        assert_eq!(identifier.identify_lane(&top_positions), "上路");

        // 中路
        let mid_positions = vec![
            PositionInfo { x: 7000.0, y: 7000.0, timestamp: 0 },
        ];
        assert_eq!(identifier.identify_lane(&mid_positions), "中路");
    }
}

