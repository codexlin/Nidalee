/// 核心分析模块
///
/// 职责：
/// - 数据解析（Parser）
/// - 统计计算（Stats）
/// - 策略选择（Strategy）
/// - 时间线解析（TimelineParser）
/// - 时间线桥接（TimelineBridge）
/// - 对手识别（OpponentIdentifier）
/// - 事件分析（EventAnalyzer）
pub mod parser;
pub mod stats;
pub mod strategy;
pub mod timeline_parser;
pub mod timeline_bridge;
pub mod opponent_identifier;
pub mod event_analyzer;

// 重新导出
pub use opponent_identifier::{OpponentIdentifier, OpponentMatch};
pub use event_analyzer::{EventAnalyzer, EventStatistics, KeyMoment};
