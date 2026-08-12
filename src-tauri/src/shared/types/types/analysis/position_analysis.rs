use super::super::SummonerInfo;
use super::process_review::ProcessInsight;
use super::statistics::{ChampionStat, PlayerMatchStats, TrendPoint};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/SummonerWithMatches.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct SummonerWithMatches {
    pub display_name: String,
    pub summoner_info: SummonerInfo,
    pub matches: PlayerMatchStats,
    /// 与 `matches` 同源的一次 analyze 结果投影，避免搜索页再打第二轮 LCU
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub position_analysis: Option<MultiPositionAnalysis>,
}

/// ⭐ v3.4: 多位置分组分析结果
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/MultiPositionAnalysis.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct MultiPositionAnalysis {
    /// 所有位置的统计（按场次从多到少排序）
    pub position_stats: Vec<PositionStats>,

    /// 主要位置（场次最多的）
    pub main_position: String,

    /// 总览数据（所有位置合计；全部模式下为混合合计）
    pub overall_stats: PlayerMatchStats,

    /// 排位桶统计（420/440）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ranked_stats: Option<PlayerMatchStats>,

    /// 非排位桶统计
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub other_stats: Option<PlayerMatchStats>,
}

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
