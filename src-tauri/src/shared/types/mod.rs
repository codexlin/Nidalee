//! 共享类型定义
//! 
//! 包含跨领域使用的类型：
//! - LCU 数据结构
//! - API 响应类型
//! - 通用枚举

// 包含类型模块
pub mod types;
pub mod types_api;

// Re-export commonly used types
pub use types::*;
