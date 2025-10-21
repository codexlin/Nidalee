//! 数据服务领域
//!
//! 负责提供各种数据服务：
//! - 召唤师信息
//! - 英雄数据
//! - 外部数据源（OP.GG等）

pub mod champion_data;
pub mod external;
pub mod summoner;

// Re-export services
