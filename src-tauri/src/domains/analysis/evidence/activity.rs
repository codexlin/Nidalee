//! 邻近帧活动粗分类（过程复盘用）
//!
//! 不追求高精度 pathing，只回答「资源刷新时我大概在干嘛」。

use super::timeline::FrameSnapshot;

const BLUE_FOUNTAIN: (f64, f64) = (400.0, 400.0);
const RED_FOUNTAIN: (f64, f64) = (14400.0, 14400.0);
const FOUNTAIN_RADIUS: f64 = 1500.0;
/// 阵亡后短时间内仍视为死亡态（用于资源错过上下文）
pub const RECENT_DEATH_WINDOW_MS: i64 = 45_000;

/// 目标玩家在事件时刻的粗活动
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ActivityContext.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum ActivityContext {
    Dead,
    Base,
    OwnJungle,
    EnemyJungle,
    RiverOrObjective,
    Lane,
    Unknown,
}

/// 根据坐标与队伍粗分活动区
pub fn classify_activity(
    snapshot: Option<&FrameSnapshot>,
    team_id: i32,
    last_death_ms: Option<i64>,
    event_ms: i64,
) -> ActivityContext {
    if let Some(death_ms) = last_death_ms {
        if event_ms >= death_ms && event_ms - death_ms <= RECENT_DEATH_WINDOW_MS {
            return ActivityContext::Dead;
        }
    }

    let Some(snap) = snapshot else {
        return ActivityContext::Unknown;
    };
    let (Some(x), Some(y)) = (snap.x, snap.y) else {
        return ActivityContext::Unknown;
    };

    if in_fountain(x, y, team_id) {
        return ActivityContext::Base;
    }

    // 召唤师峡谷粗分区：蓝方野区偏左下，红方偏右上；河道沿对角线
    let blue_team = team_id == 100;
    let in_blue_jungle = x < 7000.0 && y > 4000.0 && !near_river(x, y);
    let in_red_jungle = x > 8000.0 && y < 11000.0 && !near_river(x, y);

    if near_river(x, y) {
        return ActivityContext::RiverOrObjective;
    }

    if blue_team {
        if in_blue_jungle {
            return ActivityContext::OwnJungle;
        }
        if in_red_jungle {
            return ActivityContext::EnemyJungle;
        }
    } else {
        if in_red_jungle {
            return ActivityContext::OwnJungle;
        }
        if in_blue_jungle {
            return ActivityContext::EnemyJungle;
        }
    }

    ActivityContext::Lane
}

fn near_river(x: f64, y: f64) -> bool {
    // 对角线河道：|x - y| 较小且不在泉水
    (x - y).abs() < 2200.0 && x > 3500.0 && x < 11500.0
}

fn in_fountain(x: f64, y: f64, team_id: i32) -> bool {
    let (fx, fy) = if team_id == 100 { BLUE_FOUNTAIN } else { RED_FOUNTAIN };
    ((x - fx).powi(2) + (y - fy).powi(2)).sqrt() <= FOUNTAIN_RADIUS
}

/// 在有序帧中找 `timestamp <= event_ms` 的最近目标快照
pub fn nearest_snapshot_at<'a>(frames: &'a [(i64, FrameSnapshot)], event_ms: i64) -> Option<&'a FrameSnapshot> {
    frames
        .iter()
        .rev()
        .find(|(ts, _)| *ts <= event_ms)
        .map(|(_, snap)| snap)
        .or_else(|| frames.first().map(|(_, snap)| snap))
}
