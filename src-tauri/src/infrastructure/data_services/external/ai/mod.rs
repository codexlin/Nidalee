//! OpenAI-compatible BYOK AI 客户端
//!
//! - API Key 只存 Windows Credential Manager（`keyring`），永不进入前端 Pinia / localStorage / 日志
//! - 非敏感配置（baseUrl / model）可用应用内 store 或请求参数传入

pub mod client;
pub mod commands;
pub mod credentials;
pub mod types;

pub use client::{parse_ai_insight_response, test_connection, AiClient};
pub use credentials::{clear_api_key, has_api_key, set_api_key};
pub use types::{AiPublicSettings, AiProviderConfig};
