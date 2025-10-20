//! LCU HTTP 通用请求工具，支持全局认证、自动重试、统一错误处理、泛型反序列化
use crate::infrastructure::game_session::auth::service::{ensure_valid_auth_info, refresh_auth_info};
use base64::{engine::general_purpose, Engine as _};
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Instant;

/// 内部通用：带全局认证的 HTTP 请求（返回原始 Response） 如果只想拿原始字节，可用 lcu_request_raw，然后 response.bytes().await
pub async fn lcu_request_raw(
    client: &Client,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<Response, String> {
    let auth = ensure_valid_auth_info().ok_or("认证信息不存在，请检查LCU进程或重试")?;
    let url = format!("https://127.0.0.1:{}{}", auth.app_port, path);

    // 记录起始时间
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
        return Err(format!("服务器返回错误: {}", response.status()));
    }
    Ok(response)
}

/// 自动反序列化为 T，带 401/403 自动刷新重试
pub async fn lcu_request_json<T: DeserializeOwned>(
    client: &Client,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> Result<T, String> {
    // 最多两次尝试（第一次失败尝试刷新认证后再来一次）
    for try_num in 0..2 {
        let response = lcu_request_raw(client, method.clone(), path, body.clone()).await;
        match response {
            Ok(resp) => {
                // 检查 401/403
                let status = resp.status();
                if status == 401 || status == 403 {
                    log::warn!(
                        "[LCU] {} {} 返回 {}，尝试强制刷新认证({}次)",
                        method,
                        path,
                        status,
                        try_num + 1
                    );
                    // 强制刷新认证
                    if refresh_auth_info().is_ok() {
                        continue; // 再试一次
                    } else {
                        return Err("强制刷新认证失败，请检查LCU进程".to_string());
                    }
                }
                if status == reqwest::StatusCode::NO_CONTENT {
                    // 返回默认值
                    return serde_json::from_str("null").map_err(|e| format!("204返回空，解析失败: {}", e));
                }
                return resp.json::<T>().await.map_err(|e| format!("解析响应失败: {}", e));
            }
            Err(e) => {
                // lcu_request_raw 已有日志
                return Err(e);
            }
        }
    }
    Err("多次认证失败，请重启游戏客户端或检查LCU进程".to_string())
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
