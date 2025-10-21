//! 共享工具函数
//!
//! 包含跨领域使用的工具：
//! - HTTP 请求工具
//! - 轮询管理器
//! - 通用辅助函数

// Re-export moved utilities
pub use crate::shared::optimized_polling::*;
pub use crate::shared::request::*;
