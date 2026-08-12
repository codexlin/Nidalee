use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

use crate::domains::ai_analysis::{AiInsight, AiPromptBundle};

use super::credentials::load_api_key;
use super::types::{AiProviderConfig, ChatCompletionRequest, ChatCompletionResponse, ChatMessage, ResponseFormat};

pub struct AiClient {
    http: Client,
    config: AiProviderConfig,
}

impl AiClient {
    pub fn from_public(base_url: &str, model: &str) -> Result<Self, String> {
        let api_key = load_api_key()?;
        let base_url = normalize_ai_base_url(base_url)?;
        let http = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("Nidalee/1.0")
            .build()
            .map_err(|e| format!("创建 AI HTTP 客户端失败: {e}"))?;
        Ok(Self {
            http,
            config: AiProviderConfig {
                base_url,
                model: model.to_string(),
                api_key,
            },
        })
    }

    pub async fn complete_json(&self, system: &str, user: &str) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = ChatCompletionRequest {
            model: self.config.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user.to_string(),
                },
            ],
            temperature: 0.2,
            response_format: Some(ResponseFormat {
                format_type: "json_object".to_string(),
            }),
        };

        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("AI 请求失败: {e}"))?;

        let status = response.status();
        let text = response.text().await.map_err(|e| format!("读取 AI 响应失败: {e}"))?;

        if !status.is_success() {
            // 绝不把 Key 打进错误
            let sanitized = text.replace(&self.config.api_key, "***");
            return Err(format!("AI 服务返回 {status}: {sanitized}"));
        }

        let parsed: ChatCompletionResponse =
            serde_json::from_str(&text).map_err(|e| format!("解析 AI 响应失败: {e}"))?;
        parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| "AI 响应缺少 content".to_string())
    }

    /// 结构化解读；Serde 失败时最多重试一次
    pub async fn analyze_evidence(&self, system: &str, user: &str) -> Result<AiInsight, String> {
        let first = self.complete_json(system, user).await?;
        match parse_ai_insight_response(&first) {
            Ok(insight) => Ok(insight),
            Err(first_err) => {
                log::warn!("AI 结构化解析失败，重试一次: {first_err}");
                let retry_user = format!(
                    "{user}\n\n上一次输出无法通过 schema 校验（{first_err}）。请严格只输出符合 schema 的 JSON。"
                );
                let second = self.complete_json(system, &retry_user).await?;
                parse_ai_insight_response(&second)
            }
        }
    }
}

pub async fn test_connection(base_url: &str, model: &str) -> Result<String, String> {
    let client = AiClient::from_public(base_url, model)?;
    let content = client
        .complete_json("Reply with a tiny JSON object {\"ok\":true}.", "ping")
        .await?;
    Ok(format!("连接成功，模型返回 {} 字符", content.len()))
}

/// 校验 BYOK Base URL：公网端点必须使用 HTTPS，仅本机开发端点允许 HTTP。
pub fn normalize_ai_base_url(base_url: &str) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    let Ok(url) = reqwest::Url::parse(trimmed) else {
        return Err("Base URL 无法解析，请使用 https://api.example.com/v1 形式".to_string());
    };
    let Some(host) = url.host_str() else {
        return Err("Base URL 缺少有效主机名".to_string());
    };
    let is_local_development_host = matches!(host, "localhost" | "127.0.0.1");
    match url.scheme() {
        "https" => {}
        "http" if is_local_development_host => {}
        "http" => {
            return Err("公网 Base URL 必须使用 HTTPS；仅 localhost 或 127.0.0.1 可使用 HTTP".to_string());
        }
        _ => {
            return Err("Base URL 仅支持 HTTPS；本机开发可使用 HTTP".to_string());
        }
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("Base URL 不得包含用户名或密码".to_string());
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err("Base URL 不得包含查询参数或片段".to_string());
    }
    Ok(trimmed.to_string())
}

/// 从模型文本中提取并校验 AiInsight（支持裸 JSON 或 ```json 包裹）
pub fn parse_ai_insight_response(raw: &str) -> Result<AiInsight, String> {
    let trimmed = raw.trim();
    let json_slice = extract_json_object(trimmed).ok_or_else(|| "响应中未找到 JSON 对象".to_string())?;
    let value: Value = serde_json::from_str(json_slice).map_err(|e| format!("JSON 解析失败: {e}"))?;
    let insight: AiInsight = serde_json::from_value(value).map_err(|e| format!("AiInsight schema 校验失败: {e}"))?;

    if insight.summary.trim().is_empty() {
        return Err("summary 不能为空".to_string());
    }
    if !(0.0..=1.0).contains(&insight.confidence) {
        return Err("confidence 必须在 0~1".to_string());
    }
    Ok(insight)
}

fn extract_json_object(text: &str) -> Option<&str> {
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            if end > start {
                return Some(&text[start..=end]);
            }
        }
    }
    None
}

#[allow(dead_code)]
pub fn preview_prompt_bundle(bundle: &AiPromptBundle) -> String {
    serde_json::to_string_pretty(bundle).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::normalize_ai_base_url;

    #[test]
    fn accepts_https_openai_compatible_url() {
        assert_eq!(
            normalize_ai_base_url("https://api.openai.com/v1/").unwrap(),
            "https://api.openai.com/v1"
        );
    }

    #[test]
    fn accepts_http_only_for_explicit_local_development_hosts() {
        assert_eq!(
            normalize_ai_base_url("http://localhost:11434/v1/").unwrap(),
            "http://localhost:11434/v1"
        );
        assert_eq!(
            normalize_ai_base_url("http://127.0.0.1:8080/v1").unwrap(),
            "http://127.0.0.1:8080/v1"
        );
    }

    #[test]
    fn rejects_insecure_remote_and_lookalike_hosts() {
        assert!(normalize_ai_base_url("http://api.example.com/v1").is_err());
        assert!(normalize_ai_base_url("http://localhost.example.com/v1").is_err());
        assert!(normalize_ai_base_url("http://127.0.0.1.example.com/v1").is_err());
    }

    #[test]
    fn rejects_missing_scheme_and_host() {
        assert!(normalize_ai_base_url("api.openai.com/v1").is_err());
        assert!(normalize_ai_base_url("https://").is_err());
        assert!(normalize_ai_base_url("ftp://example.com/v1").is_err());
        assert!(normalize_ai_base_url("https://user:pass@example.com/v1").is_err());
        assert!(normalize_ai_base_url("https://example.com/v1?tenant=1").is_err());
    }
}
