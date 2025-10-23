/// 特征分析模块
///
/// 7个特征分析器：
/// - basic: 基础特征（胜率、KDA）
/// - advanced: 深度特征（参团率、伤害占比）
/// - role: 位置特征（专精、全能）
/// - distribution: 分布特征（高光、崩盘）
/// - timeline: 时间线特征（对线、发育曲线）
/// - queue_aware: 队列感知特征（队列特定评估和建议）
/// - merger: 智能去重合并
pub mod advanced;
pub mod basic;
pub mod distribution;
pub mod merger;
pub mod queue_aware;
pub mod role;
pub mod timeline;

// 重新导出
