//! 共享层
//!
//! 包含跨领域共享的代码：
//! - 类型定义
//! - 错误处理
//! - 工具函数
//! - 请求封装
//! - 性能优化

pub mod types;
pub mod errors;
pub mod utils;
pub mod request;
pub mod optimized_polling;

// Re-export commonly used items
pub use types::*;
pub use errors::{NidaleeError, Result};
