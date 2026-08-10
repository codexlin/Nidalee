use serde_json::Value;
use tauri::State;
use tokio::sync::Mutex;

use crate::domains::ai_analysis::{build_ai_prompt, compact_evidence_for_ai, AiInsight};
use crate::domains::analysis::pipeline::MatchAnalysisResult;
use crate::infrastructure::data_services::external::ai::{
    clear_api_key, has_api_key, parse_ai_insight_response, set_api_key, test_connection, AiClient, AiPublicSettings,
};

/// 进程内非敏感 AI 配置（不存 Key）
pub struct AiSettingsState(pub Mutex<AiPublicSettings>);

impl Default for AiSettingsState {
    fn default() -> Self {
        Self(Mutex::new(AiPublicSettings::default()))
    }
}

#[tauri::command]
pub async fn get_ai_settings(state: State<'_, AiSettingsState>) -> Result<AiPublicSettings, String> {
    let mut settings = state.0.lock().await.clone();
    settings.has_api_key = has_api_key();
    // 防御：确保绝不带 Key 字段（结构体本身就没有）
    Ok(settings)
}

#[tauri::command]
pub async fn set_ai_settings(
    state: State<'_, AiSettingsState>,
    enabled: bool,
    base_url: String,
    model: String,
) -> Result<AiPublicSettings, String> {
    let mut guard = state.0.lock().await;
    guard.enabled = enabled;
    guard.base_url = super::client::normalize_ai_base_url(&base_url)?;
    guard.model = model.trim().to_string();
    guard.provider = "openai-compatible".to_string();
    guard.has_api_key = has_api_key();
    Ok(guard.clone())
}

#[tauri::command]
pub async fn set_ai_api_key(api_key: String) -> Result<bool, String> {
    set_api_key(&api_key)?;
    Ok(true)
}

#[tauri::command]
pub async fn clear_ai_api_key() -> Result<bool, String> {
    clear_api_key()?;
    Ok(true)
}

#[tauri::command]
pub async fn test_ai_connection(state: State<'_, AiSettingsState>) -> Result<String, String> {
    let settings = state.0.lock().await.clone();
    if !has_api_key() {
        return Err("尚未配置 API Key".to_string());
    }
    test_connection(&settings.base_url, &settings.model).await
}

/// 预览将发送给模型的脱敏证据（不含 Key、不含 PUUID）
#[tauri::command]
pub async fn preview_ai_prompt(result: MatchAnalysisResult) -> Result<Value, String> {
    ensure_local_ai_eligible(&result)?;
    let bundle = compact_evidence_for_ai(&result)
        .ok_or_else(|| "当前结果没有可用的排位深度证据，无法生成 AI 解读".to_string())?;
    serde_json::to_value(bundle).map_err(|e| e.to_string())
}

/// 显式触发 AI 解读；失败不影响本地分析结果
#[tauri::command]
pub async fn analyze_with_ai(
    state: State<'_, AiSettingsState>,
    result: MatchAnalysisResult,
) -> Result<AiInsight, String> {
    let settings = state.0.lock().await.clone();
    if !settings.enabled {
        return Err("AI 分析未启用".to_string());
    }
    if !settings.has_api_key && !has_api_key() {
        return Err("尚未配置 API Key".to_string());
    }
    ensure_local_ai_eligible(&result)?;

    let bundle = compact_evidence_for_ai(&result)
        .ok_or_else(|| "当前结果没有可用的排位深度证据，无法生成 AI 解读".to_string())?;
    let (system, user) = build_ai_prompt(&bundle)?;
    let client = AiClient::from_public(&settings.base_url, &settings.model)?;
    client.analyze_evidence(&system, &user).await
}

fn ensure_local_ai_eligible(result: &MatchAnalysisResult) -> Result<(), String> {
    if !result.capabilities.local_ai {
        return Err("本地 AI 需要排位深度证据，当前结果不满足条件".to_string());
    }
    Ok(())
}

/// 供单元测试复用的解析入口
#[allow(dead_code)]
pub fn parse_insight_for_test(raw: &str) -> Result<AiInsight, String> {
    parse_ai_insight_response(raw)
}
