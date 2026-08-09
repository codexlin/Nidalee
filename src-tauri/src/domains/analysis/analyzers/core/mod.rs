//! 核心分析模块
//!
//! 职责：
//! - 数据解析（Parser）
//! - 统计计算（Stats）
//! - 策略选择（Strategy）
//! - 时间线分析（TimelineAnalyzer，计算已委托给 `analysis::evidence`）
//! - 时间线桥接（TimelineBridge）
//! - 事件分析（EventAnalyzer）
//!
//! **对手识别不在这里**：唯一入口是 `domains::analysis::evidence::resolve_lane_opponent`。
//! 旧的 `opponent_identifier` 仅按 `participantId <= 5` 猜队伍、且只会挑「最近的人」，
//! 打野和游走位会被配成队友或被 gank 的对象，已降级为私有模块并标注废弃。

pub mod event_analyzer;
// 私有：不再对外暴露，避免与 evidence::resolve_lane_opponent 形成两个推荐入口。
// 整个模块已无生产调用方，仅保留历史单元测试，因此整体豁免 dead_code。
#[allow(dead_code)]
mod opponent_identifier;
pub mod parser;
pub mod stats;
pub mod strategy;
pub mod timeline_analyzer;
pub mod timeline_bridge;

// 重新导出
pub use event_analyzer::{EventAnalyzer, EventStatistics, KeyMoment};
pub use parser::{
    identify_main_role, identify_position_from_game, parse_games, ParsedGame, ParsedPlayerData, TimelineData,
};
pub use stats::{analyze_player_stats, analyze_player_stats_with_resolver, AnalysisContext, ChampionNameResolver};
pub use timeline_analyzer::{parse_timeline_data, KeyEvent, OpponentComparison, PhaseAnalysis, TimelineAnalysis};
