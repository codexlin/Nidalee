//! API Key 安全存储（Windows Credential Manager via keyring）
//!
//! 前端只能 set/clear/test；`get_ai_settings` 仅返回 `hasApiKey`。

use keyring::Entry;

const SERVICE: &str = "nidalee.ai.openai";
const ACCOUNT: &str = "api_key";

fn entry() -> Result<Entry, String> {
    Entry::new(SERVICE, ACCOUNT).map_err(|e| format!("无法访问系统凭据库: {e}"))
}

pub fn set_api_key(api_key: &str) -> Result<(), String> {
    let trimmed = api_key.trim();
    if trimmed.is_empty() {
        return Err("API Key 不能为空".to_string());
    }
    entry()?
        .set_password(trimmed)
        .map_err(|e| format!("保存 API Key 失败: {e}"))
}

pub fn clear_api_key() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(format!("清除 API Key 失败: {e}")),
    }
}

pub fn has_api_key() -> bool {
    match entry().and_then(|e| e.get_password().map_err(|err| err.to_string())) {
        Ok(value) => !value.trim().is_empty(),
        Err(_) => false,
    }
}

/// 仅供后端 HTTP 客户端使用；禁止日志打印返回值
pub(crate) fn load_api_key() -> Result<String, String> {
    let key = entry()?.get_password().map_err(|e| format!("读取 API Key 失败: {e}"))?;
    let trimmed = key.trim().to_string();
    if trimmed.is_empty() {
        return Err("尚未配置 API Key".to_string());
    }
    Ok(trimmed)
}
