//! 统一错误处理模块
//!
//! 参考顶级 Rust 项目（tokio, serde, actix-web）的错误处理策略，
//! 使用 thiserror 提供结构化、类型安全的错误处理。

use thiserror::Error;

/// Nidalee 统一错误类型
///
/// # 设计原则
/// - 结构化：每种错误都有明确的类型和上下文
/// - 可组合：使用 #[from] 自动转换第三方错误
/// - 可追踪：包含详细的错误信息和上下文
#[derive(Error, Debug)]
pub enum NidaleeError {
    // ============== LCU 相关错误 ==============
    /// LCU 连接失败
    #[error("LCU connection failed: {0}")]
    LcuConnection(String),

    /// LCU 未启动或未找到
    #[error("LCU not running or not found")]
    LcuNotFound,

    /// LCU 认证失败
    #[error("LCU authentication failed: {0}")]
    LcuAuth(String),

    /// LCU WebSocket 错误
    #[error("LCU WebSocket error: {0}")]
    LcuWebSocket(String),

    /// LCU API 调用失败
    #[error("LCU API call failed: {endpoint} - {message}")]
    LcuApiCall {
        endpoint: String,
        message: String,
    },

    // ============== 网络相关错误 ==============
    /// HTTP 请求错误（自动转换 reqwest::Error）
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// 网络超时
    #[error("Network timeout: {0}")]
    Timeout(String),

    /// 网络不可用
    #[error("Network unavailable")]
    NetworkUnavailable,

    // ============== 数据相关错误 ==============
    /// JSON 序列化/反序列化错误
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// 数据解析错误
    #[error("Data parse error: {0}")]
    Parse(String),

    /// 数据验证错误
    #[error("Data validation error: {0}")]
    Validation(String),

    /// 数据未找到
    #[error("Data not found: {0}")]
    NotFound(String),

    // ============== 文件/IO 相关错误 ==============
    /// 文件操作错误
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// 配置文件错误
    #[error("Configuration error: {0}")]
    Config(String),

    // ============== 游戏状态相关错误 ==============
    /// 游戏状态不正确（如：需要在选人界面，但当前不在）
    #[error("Invalid game state: expected {expected}, got {actual}")]
    InvalidGameState {
        expected: String,
        actual: String,
    },

    /// 玩家不在游戏中
    #[error("Player not in game")]
    NotInGame,

    /// 操作超时（如：等待选人超时）
    #[error("Operation timeout: {0}")]
    OperationTimeout(String),

    // ============== 缓存相关错误 ==============
    /// 缓存错误
    #[error("Cache error: {0}")]
    Cache(String),

    /// 缓存未命中
    #[error("Cache miss: {0}")]
    CacheMiss(String),

    // ============== 分析相关错误 ==============
    /// 数据分析错误
    #[error("Analysis error: {0}")]
    Analysis(String),

    /// 数据不足（无法进行分析）
    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    // ============== 外部服务错误 ==============
    /// OP.GG API 错误
    #[error("OP.GG API error: {0}")]
    OpggApi(String),

    /// 外部服务不可用
    #[error("External service unavailable: {0}")]
    ExternalService(String),

    // ============== 通用错误 ==============
    /// 内部错误（通常是 bug）
    #[error("Internal error: {0}")]
    Internal(String),

    /// 未实现的功能
    #[error("Feature not implemented: {0}")]
    NotImplemented(String),

    /// 权限错误
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// 其他未分类错误
    #[error("{0}")]
    Other(String),
}

/// Result 类型别名，使用 NidaleeError
pub type Result<T> = std::result::Result<T, NidaleeError>;

// ============== 便捷构造函数 ==============
impl NidaleeError {
    /// 创建 LCU 连接错误
    pub fn lcu_connection(msg: impl Into<String>) -> Self {
        Self::LcuConnection(msg.into())
    }

    /// 创建 LCU API 调用错误
    pub fn lcu_api_call(endpoint: impl Into<String>, message: impl Into<String>) -> Self {
        Self::LcuApiCall {
            endpoint: endpoint.into(),
            message: message.into(),
        }
    }

    /// 创建数据未找到错误
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    /// 创建游戏状态错误
    pub fn invalid_game_state(expected: impl Into<String>, actual: impl Into<String>) -> Self {
        Self::InvalidGameState {
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// 创建分析错误
    pub fn analysis(msg: impl Into<String>) -> Self {
        Self::Analysis(msg.into())
    }

    /// 创建内部错误
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

// ============== String 转换（向后兼容） ==============
impl From<String> for NidaleeError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for NidaleeError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}

// ============== 转换为 Tauri 错误 ==============
impl From<NidaleeError> for String {
    fn from(err: NidaleeError) -> Self {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = NidaleeError::lcu_connection("timeout");
        assert_eq!(err.to_string(), "LCU connection failed: timeout");

        let err = NidaleeError::lcu_api_call("/lol-summoner/v1/current-summoner", "404 Not Found");
        assert_eq!(
            err.to_string(),
            "LCU API call failed: /lol-summoner/v1/current-summoner - 404 Not Found"
        );
    }

    #[test]
    fn test_error_conversion() {
        // 测试 String 自动转换
        let err: NidaleeError = "custom error".into();
        assert_eq!(err.to_string(), "custom error");

        // 测试 serde_json::Error 自动转换
        let json_err = serde_json::from_str::<i32>("invalid").unwrap_err();
        let err: NidaleeError = json_err.into();
        assert!(err.to_string().contains("JSON error"));
    }
}

