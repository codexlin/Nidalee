/// 领域层 (Domain Layer)
///
/// 职责：
/// - 核心业务逻辑
/// - 领域模型定义
/// - 不依赖外部基础设施

pub mod analysis;          // 对局分析领域
pub mod tactical_advice;   // 战术建议领域

// 重新导出常用类型（保持向后兼容）

