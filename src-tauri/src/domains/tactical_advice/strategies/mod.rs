/// 策略模式模块
///
/// 三种建议生成策略：
/// 1. SelfImprovementStrategy - 改进建议（对自己）
/// 2. TargetingStrategy - 针对建议（对敌人）
/// 3. CollaborationStrategy - 协作建议（对队友）⭐
pub mod base;
pub mod collaboration;
pub mod self_improvement;
pub mod targeting;

pub use base::{AdviceStrategy, ProblemData, ProblemType};
pub use collaboration::CollaborationStrategy;
pub use self_improvement::SelfImprovementStrategy;
pub use targeting::TargetingStrategy;
