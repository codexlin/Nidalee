//! 基础设施层 - 技术实现细节
//! 
//! 包含所有与外部系统交互的代码：
//! - LCU API 调用
//! - WebSocket 通信
//! - 数据缓存
//! - 错误处理

pub mod game_session;
pub mod match_management;
pub mod champion_selection;
pub mod real_time;
pub mod data_services;

// Re-export commonly used services for backward compatibility
