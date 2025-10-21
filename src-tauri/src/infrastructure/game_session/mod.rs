//! 游戏会话管理领域
//!
//! 负责管理玩家的游戏会话生命周期：
//! - 认证授权
//! - 连接管理
//! - 游戏流程
//! - 大厅状态

pub mod auth;
pub mod connection;
pub mod gameflow;
pub mod lobby;

// Re-export services
