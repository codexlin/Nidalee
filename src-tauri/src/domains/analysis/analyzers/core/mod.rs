/// 核心分析模块
///
/// 职责：
/// - 数据解析（Parser）
/// - 统计计算（Stats）
/// - 策略选择（Strategy）
/// - 时间线分析（TimelineAnalyzer）
/// - 时间线桥接（TimelineBridge）
/// - 对手识别（OpponentIdentifier）
/// - 事件分析（EventAnalyzer）
pub mod parser;
pub mod stats;
pub mod strategy;
pub mod timeline_analyzer;
pub mod timeline_bridge;
pub mod opponent_identifier;
pub mod event_analyzer;

// 重新导出
pub use parser::{parse_games, ParsedGame, ParsedPlayerData, TimelineData, identify_main_role, identify_position_from_game};
pub use timeline_analyzer::{parse_timeline_data, TimelineAnalysis, PhaseAnalysis, KeyEvent, OpponentComparison};
pub use opponent_identifier::{OpponentIdentifier, OpponentMatch};
pub use event_analyzer::{EventAnalyzer, EventStatistics, KeyMoment};
