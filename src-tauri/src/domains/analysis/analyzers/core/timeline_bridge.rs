/// 时间线数据桥接器（简化版）
///
/// 职责：
/// - 统一访问 frames 数据
/// - 将 TimelineAnalysis 转换为 TimelineData 格式（兼容旧系统）
use crate::domains::analysis::analyzers::core::parser::TimelineData;
use crate::domains::analysis::analyzers::core::timeline_analyzer::{parse_timeline_data, TimelineAnalysis};
use crate::domains::analysis::evidence::GamePhase;
use serde_json::Value;

/// 时间线数据桥接器（仅支持 frames 数据）
pub struct TimelineBridge;

impl TimelineBridge {
    pub fn new() -> Self {
        Self
    }

    /// 解析时间线数据（从 frames 数据）
    pub fn parse_timeline(
        &self,
        match_data: &Value,
        participant_id: i32,
        opponent_id: Option<i32>,
    ) -> Option<TimelineData> {
        // 1. 解析 frames 数据
        let timeline_analysis = match_data
            .get("match_timeline_json")
            .and_then(|timeline_json| parse_timeline_data(timeline_json, participant_id, opponent_id))?;

        // 2. 转换为旧的 TimelineData 格式（保持兼容）
        Some(self.convert_timeline_analysis_to_legacy_format(&timeline_analysis, participant_id))
    }

    /// 获取完整的时间线分析
    pub fn get_full_timeline_analysis(
        &self,
        match_data: &Value,
        participant_id: i32,
        opponent_id: Option<i32>,
    ) -> Option<TimelineAnalysis> {
        match_data
            .get("match_timeline_json")
            .and_then(|timeline_json| parse_timeline_data(timeline_json, participant_id, opponent_id))
    }

    /// 将新的 TimelineAnalysis 转换为旧的 TimelineData 格式
    ///
    /// 阶段不存在时（短局 / remake）保持 `None`，绝不用 0.0 冒充真实数据 ——
    /// 否则下游会把「没打到后期」误判成「后期乏力」。
    fn convert_timeline_analysis_to_legacy_format(
        &self,
        analysis: &TimelineAnalysis,
        _participant_id: i32,
    ) -> TimelineData {
        let early = analysis.has_phase(GamePhase::Early).then_some(&analysis.early_game);
        let mid = analysis.has_phase(GamePhase::Mid).then_some(&analysis.mid_game);
        let late = analysis.has_phase(GamePhase::Late).then_some(&analysis.late_game);

        TimelineData {
            // 对线期数据 (0-10分钟)
            cs_per_min_0_10: early.map(|p| p.cs_per_minute),
            gold_per_min_0_10: early.map(|p| p.gold_per_minute),
            xp_per_min_0_10: early.map(|p| p.xp_per_minute),
            cs_diff_0_10: early.map(|p| p.cs_difference),
            xp_diff_0_10: early.map(|p| p.xp_difference),
            damage_taken_per_min_0_10: None,

            // 发育期数据 (10-20分钟)
            cs_per_min_10_20: mid.map(|p| p.cs_per_minute),
            gold_per_min_10_20: mid.map(|p| p.gold_per_minute),
            xp_per_min_10_20: mid.map(|p| p.xp_per_minute),
            cs_diff_10_20: mid.map(|p| p.cs_difference),
            xp_diff_10_20: mid.map(|p| p.xp_difference),
            damage_taken_per_min_10_20: None,

            // 后期数据 (20分钟+)
            cs_per_min_20_end: late.map(|p| p.cs_per_minute),
            gold_per_min_20_end: late.map(|p| p.gold_per_minute),
            cs_diff_20_end: late.map(|p| p.cs_difference),
        }
    }
}

impl Default for TimelineBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_timeline_bridge_frames_mode() {
        let bridge = TimelineBridge::new();
        let match_data = json!({
            "match_timeline_json": {
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
            }
        });

        let result = bridge.parse_timeline(&match_data, 1, None);
        assert!(result.is_some());
    }

    #[test]
    fn test_get_full_timeline_analysis() {
        let bridge = TimelineBridge::new();
        let match_data = json!({
            "match_timeline_json": {
                "frames": []
            }
        });

        let result = bridge.get_full_timeline_analysis(&match_data, 1, None);
        // 空 frames 应该返回 None
        assert!(result.is_none());
    }
}
