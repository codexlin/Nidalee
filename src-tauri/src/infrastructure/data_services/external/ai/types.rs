use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 前端可见的 AI 配置（绝不包含明文 Key）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AiPublicSettings.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct AiPublicSettings {
    pub enabled: bool,
    pub provider: String,
    pub base_url: String,
    pub model: String,
    /// 仅表示是否已配置 Key，永不回传明文
    pub has_api_key: bool,
}

impl Default for AiPublicSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "openai-compatible".to_string(),
            base_url: "https://api.openai.com/v1".to_string(),
            model: "gpt-4o-mini".to_string(),
            has_api_key: false,
        }
    }
}

/// 运行时 Provider 配置（含从 keyring 读出的 Key，仅后端持有）
#[derive(Debug, Clone)]
pub struct AiProviderConfig {
    pub base_url: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
}

#[derive(Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ResponseFormat {
    #[serde(rename = "type")]
    pub format_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
pub struct ChatChoice {
    pub message: ChatResponseMessage,
}

#[derive(Debug, Deserialize)]
pub struct ChatResponseMessage {
    pub content: Option<String>,
}
