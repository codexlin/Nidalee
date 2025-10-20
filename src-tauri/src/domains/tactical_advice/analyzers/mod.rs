/// 建议分析器模块
///
/// 5个责任链节点：
/// 1. LaningAdviceAnalyzer - 对线期建议
/// 2. FarmingAdviceAnalyzer - 发育建议
/// 3. TeamfightAdviceAnalyzer - 团战建议
/// 4. VisionAdviceAnalyzer - 视野建议
/// 5. ChampionAdviceAnalyzer - 英雄池建议

pub mod base;
pub mod laning;
pub mod farming;
pub mod teamfight;
pub mod vision;
pub mod champion;

pub use laning::LaningAdviceAnalyzer;
pub use farming::FarmingAdviceAnalyzer;
pub use teamfight::TeamfightAdviceAnalyzer;
pub use vision::VisionAdviceAnalyzer;
pub use champion::ChampionAdviceAnalyzer;

