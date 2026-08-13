//! LCU HTTP 通用请求工具，支持全局认证、自动重试、统一错误处理、泛型反序列化
use crate::infrastructure::game_session::auth::service::{
    ensure_valid_auth_info_async, refresh_auth_info_after_rejection,
};
use crate::shared::types::LcuAuthInfo;
use base64::{engine::general_purpose, Engine as _};
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::borrow::Cow;
use std::future::Future;
use std::time::Instant;

fn should_retry_auth(status: reqwest::StatusCode, attempt: usize) -> bool {
    attempt == 0 && (status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN)
}

struct LcuAttempt<T> {
    value: T,
    status: reqwest::StatusCode,
    auth: LcuAuthInfo,
}

fn is_uuid_segment(segment: &str) -> bool {
    segment.len() == 36
        && segment.bytes().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
}

fn redact_lcu_path(path: &str) -> Cow<'_, str> {
    if !path.split(['/', '?', '&', '=']).any(is_uuid_segment) {
        return Cow::Borrowed(path);
    }

    let mut redacted = String::with_capacity(path.len());
    for segment in path.split_inclusive(['/', '?', '&', '=']) {
        let separator_len = segment
            .as_bytes()
            .last()
            .is_some_and(|byte| matches!(byte, b'/' | b'?' | b'&' | b'=')) as usize;
        let value_len = segment.len() - separator_len;
        let (value, separator) = segment.split_at(value_len);
        if is_uuid_segment(value) {
            redacted.push_str("<puuid>");
        } else {
            redacted.push_str(value);
        }
        redacted.push_str(separator);
    }
    Cow::Owned(redacted)
}

async fn execute_with_auth_retry<T, Send, SendFuture, Refresh, RefreshFuture>(
    mut send: Send,
    mut refresh: Refresh,
) -> Result<T, String>
where
    Send: FnMut() -> SendFuture,
    SendFuture: Future<Output = Result<LcuAttempt<T>, String>>,
    Refresh: FnMut(LcuAuthInfo) -> RefreshFuture,
    RefreshFuture: Future<Output = Result<LcuAuthInfo, String>>,
{
    for attempt in 0..2 {
        let response = send().await?;

        if response.status == reqwest::StatusCode::UNAUTHORIZED || response.status == reqwest::StatusCode::FORBIDDEN {
            if should_retry_auth(response.status, attempt) {
                log::warn!("Authentication failed with {}; refreshing once", response.status);
                refresh(response.auth).await?;
                continue;
            }
            return Err(format!("多次认证失败，服务器返回错误: {}", response.status));
        }

        if !response.status.is_success() {
            return Err(format!("服务器返回错误: {}", response.status));
        }

        return Ok(response.value);
    }

    Err("多次认证失败，请重启游戏客户端或检查LCU进程".to_string())
}

async fn send_lcu_request(
    client: &Client,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<LcuAttempt<Response>, String> {
    let auth = ensure_valid_auth_info_async()
        .await
        .ok_or("认证信息不存在，请检查LCU进程或重试")?;
    let url = format!("https://127.0.0.1:{}{}", auth.app_port, path);
    let log_path = redact_lcu_path(path);

    let start = Instant::now();
    log::info!("{} {}", method, log_path);

    let auth_string = format!("riot:{}", auth.remoting_auth_token);
    let base64_auth = general_purpose::STANDARD.encode(auth_string.as_bytes());

    let builder = client
        .request(method.clone(), &url)
        .header("Authorization", format!("Basic {}", base64_auth));

    let builder = if let Some(ref body) = body {
        log::trace!("请求包含 JSON body");
        builder.json(body)
    } else {
        builder
    };

    let response = builder.send().await.map_err(|e| {
        log::error!("{} {} 发送失败: {}", method, log_path, e);
        format!("请求失败: {}", e)
    })?;

    let duration = start.elapsed();
    log::info!(
        "{} {} -> {} (耗时: {}ms)",
        method,
        log_path,
        response.status(),
        duration.as_millis()
    );

    if !response.status().is_success() {
        log::warn!(
            "{} {} 返回非成功状态: {} (耗时: {}ms)",
            method,
            log_path,
            response.status(),
            duration.as_millis()
        );
    }

    let status = response.status();
    Ok(LcuAttempt {
        value: response,
        status,
        auth,
    })
}

/// 内部通用：带全局认证的 HTTP 请求（返回原始 Response） 如果只想拿原始字节，可用 lcu_request_raw，然后 response.bytes().await
pub async fn lcu_request_raw(
    client: &Client,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<Response, String> {
    execute_with_auth_retry(
        || send_lcu_request(client, method.clone(), path, body.clone()),
        |failed_auth| async move {
            refresh_auth_info_after_rejection(&failed_auth)
                .await
                .map_err(|error| format!("强制刷新认证失败，请检查LCU进程: {error}"))
        },
    )
    .await
}

/// 自动反序列化为 T，带 401/403 自动刷新重试
pub async fn lcu_request_json<T: DeserializeOwned>(
    client: &Client,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<T, String> {
    let response = lcu_request_raw(client, method, path, body).await?;
    if response.status() == reqwest::StatusCode::NO_CONTENT {
        return serde_json::from_str("null").map_err(|e| format!("204返回空，解析失败: {}", e));
    }

    response.json::<T>().await.map_err(|e| format!("解析响应失败: {}", e))
}

/// GET 方法，自动反序列化为 T
pub async fn lcu_get<T: DeserializeOwned>(client: &Client, path: &str) -> Result<T, String> {
    lcu_request_json(client, Method::GET, path, None).await
}

/// POST 方法，自动反序列化为 T
pub async fn lcu_post<T: DeserializeOwned>(client: &Client, path: &str, body: Value) -> Result<T, String> {
    lcu_request_json(client, Method::POST, path, Some(body)).await
}

/// PUT 方法，自动反序列化为 T
#[allow(dead_code)]
pub async fn lcu_put<T: DeserializeOwned>(client: &Client, path: &str, body: Value) -> Result<T, String> {
    lcu_request_json(client, Method::PUT, path, Some(body)).await
}

/// DELETE 方法，自动反序列化为 T
pub async fn lcu_delete<T: DeserializeOwned>(client: &Client, path: &str) -> Result<T, String> {
    lcu_request_json(client, Method::DELETE, path, None).await
}

#[allow(dead_code)]
pub async fn lcu_post_no_content(client: &Client, path: &str, body: Value) -> Result<(), String> {
    // 只关心成功，不需要反序列化
    let response = lcu_request_raw(client, Method::POST, path, Some(body)).await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("服务器返回错误: {}", response.status()))
    }
}
#[cfg(test)]
#[path = "request_tests.rs"]
mod tests;
