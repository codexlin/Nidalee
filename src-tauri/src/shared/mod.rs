//! 共享层
//!
//! 包含跨领域共享的代码：
//! - 类型定义
//! - 错误处理
//! - 工具函数
//! - 请求封装
//! - 性能优化

pub mod errors;
pub mod optimized_polling;
pub mod request;
pub mod types;
pub mod utils;

// Re-export commonly used items
pub use errors::{NidaleeError, Result};
pub use types::*;
