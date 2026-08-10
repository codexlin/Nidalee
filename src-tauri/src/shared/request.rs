//! LCU HTTP 通用请求工具，支持全局认证、自动重试、统一错误处理、泛型反序列化
use crate::infrastructure::game_session::auth::service::{
    ensure_valid_auth_info_async, refresh_auth_info_after_rejection,
};
use crate::shared::types::LcuAuthInfo;
use base64::{engine::general_purpose, Engine as _};
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde_json::Value;
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
                log::warn!("[LCU] Authentication failed with {}; refreshing once", response.status);
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

    let start = Instant::now();
    log::info!("[LCU] {} {}", method, url);

    let auth_string = format!("riot:{}", auth.remoting_auth_token);
    let base64_auth = general_purpose::STANDARD.encode(auth_string.as_bytes());

    let builder = client
        .request(method.clone(), &url)
        .header("Authorization", format!("Basic {}", base64_auth));

    let builder = if let Some(ref body) = body {
        log::debug!("[LCU] 请求体: {}", body);
        builder.json(body)
    } else {
        builder
    };

    let response = builder.send().await.map_err(|e| {
        log::error!("[LCU] {} {} 发送失败: {}", method, url, e);
        format!("请求失败: {}", e)
    })?;

    let duration = start.elapsed();
    log::info!(
        "[LCU] {} {} -> {} (耗时: {}ms)",
        method,
        url,
        response.status(),
        duration.as_millis()
    );

    if !response.status().is_success() {
        log::warn!(
            "[LCU] {} {} 返回非成功状态: {} (耗时: {}ms)",
            method,
            url,
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
    if response.status() == reqwest::StatusCode::NO_CONTENT {
        Ok(())
    } else if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("服务器返回错误: {}", response.status()))
    }
}
pub async fn lcu_patch_no_content(client: &Client, path: &str, body: Value) -> Result<(), String> {
    let response = lcu_request_raw(client, Method::PATCH, path, Some(body)).await?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("服务器返回错误: {}", response.status()))
    }
}

/// 通用 champ-r HTTP 请求，返回反序列化后的数据
pub async fn forin_request_json<T: DeserializeOwned>(
    client: &Client,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<T, String> {
    const FORIN_API_URL: &str = "https://c.lbj.moe";
    let url = format!("{}{}", FORIN_API_URL, path);

    let start = Instant::now();
    log::info!("[CHAMPR] {} {}", method, url);

    let builder = client.request(method.clone(), &url);

    let builder = if let Some(ref body) = body {
        log::debug!("[CHAMPR] 请求体: {}", body);
        builder.json(body)
    } else {
        builder
    };

    let response = builder.send().await.map_err(|e| {
        log::error!("[CHAMPR] {} {} 发送失败: {}", method, url, e);
        format!("请求失败: {}", e)
    })?;

    let duration = start.elapsed();
    log::info!(
        "[CHAMPR] {} {} -> {} (耗时: {}ms)",
        method,
        url,
        response.status(),
        duration.as_millis()
    );

    if !response.status().is_success() {
        log::warn!(
            "[CHAMPR] {} {} 返回非成功状态: {} (耗时: {}ms)",
            method,
            url,
            response.status(),
            duration.as_millis()
        );
        return Err(format!("服务器返回错误: {}", response.status()));
    }

    response.json::<T>().await.map_err(|e| format!("解析响应失败: {}", e))
}

#[cfg(test)]
mod tests {
    use super::{execute_with_auth_retry, LcuAttempt};
    use crate::shared::types::LcuAuthInfo;
    use reqwest::StatusCode;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    fn auth(token: &str, port: u16) -> LcuAuthInfo {
        LcuAuthInfo {
            riotclient_auth_token: format!("riot-{token}"),
            riotclient_app_port: port + 1,
            remoting_auth_token: token.to_string(),
            app_port: port,
        }
    }

    async fn run_sequence(statuses: Vec<StatusCode>) -> (Result<&'static str, String>, usize, Vec<String>) {
        let statuses = Arc::new(tokio::sync::Mutex::new(VecDeque::from(statuses)));
        let current_auth = Arc::new(tokio::sync::Mutex::new(auth("old", 2999)));
        let used_tokens = Arc::new(Mutex::new(Vec::new()));
        let refreshes = Arc::new(AtomicUsize::new(0));

        let result = execute_with_auth_retry(
            || {
                let statuses = statuses.clone();
                let current_auth = current_auth.clone();
                let used_tokens = used_tokens.clone();
                async move {
                    let status = statuses
                        .lock()
                        .await
                        .pop_front()
                        .expect("test status sequence exhausted");
                    let auth = current_auth.lock().await.clone();
                    used_tokens.lock().unwrap().push(auth.remoting_auth_token.clone());
                    Ok(LcuAttempt {
                        value: "ok",
                        status,
                        auth,
                    })
                }
            },
            |_failed_auth| {
                let current_auth = current_auth.clone();
                let refreshes = refreshes.clone();
                async move {
                    refreshes.fetch_add(1, Ordering::SeqCst);
                    let refreshed = auth("new", 3001);
                    *current_auth.lock().await = refreshed.clone();
                    Ok(refreshed)
                }
            },
        )
        .await;

        let refresh_count = refreshes.load(Ordering::SeqCst);
        let tokens = used_tokens.lock().unwrap().clone();
        (result, refresh_count, tokens)
    }

    #[tokio::test]
    async fn success_does_not_refresh() {
        let (result, refreshes, tokens) = run_sequence(vec![StatusCode::OK]).await;
        assert_eq!(result, Ok("ok"));
        assert_eq!(refreshes, 0);
        assert_eq!(tokens, vec!["old"]);
    }

    #[tokio::test]
    async fn unauthorized_refreshes_then_retries_with_new_auth() {
        let (result, refreshes, tokens) = run_sequence(vec![StatusCode::UNAUTHORIZED, StatusCode::OK]).await;
        assert_eq!(result, Ok("ok"));
        assert_eq!(refreshes, 1);
        assert_eq!(tokens, vec!["old", "new"]);
    }

    #[tokio::test]
    async fn second_unauthorized_fails_without_another_refresh() {
        let (result, refreshes, _) = run_sequence(vec![StatusCode::UNAUTHORIZED, StatusCode::UNAUTHORIZED]).await;
        assert!(result.unwrap_err().contains("多次认证失败"));
        assert_eq!(refreshes, 1);
    }

    #[tokio::test]
    async fn server_error_does_not_refresh() {
        let (result, refreshes, _) = run_sequence(vec![StatusCode::INTERNAL_SERVER_ERROR]).await;
        assert!(result.unwrap_err().contains("500"));
        assert_eq!(refreshes, 0);
    }
}
