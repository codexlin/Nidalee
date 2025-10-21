//! 对局管理领域
//!
//! 负责处理对局相关的所有数据：
//! - 对局数据获取
//! - 分析数据构建
//! - 排位信息
//! - 匹配系统

pub mod analysis_data;
pub mod matches;
pub mod matchmaking;
pub mod ranked;

// Re-export services
