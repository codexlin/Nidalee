use super::process_review::ProcessInsight;
use super::statistics::{ChampionStat, PlayerMatchStats, TrendPoint};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// ⭐ v3.4: 单个位置的统计数据
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PositionStats.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct PositionStats {
    /// ASCII 位置码：TOP / JUNGLE / MID / ADC / SUPPORT / ARAM / FLEX / UNKNOWN
    pub position: String,

    /// 该位置的场次
    pub games: u32,

    /// 该位置的胜场
    pub wins: u32,

    /// 该位置的胜率
    pub win_rate: f64,

    /// 该位置的统计数据
    pub stats: PlayerMatchStats,

    /// 英雄池数据（Top 5）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub champion_pool: Option<Vec<ChampionStat>>,

    /// 胜率趋势（最近的对局）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub win_rate_trend: Option<Vec<TrendPoint>>,

    /// 过程复盘洞察（深度时间线）；无时间线时带降级说明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_insight: Option<ProcessInsight>,
}
