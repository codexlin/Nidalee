use reqwest::Client;
use serde_json::Value;
use std::sync::RwLock;
use std::time::{Duration, Instant};

const BASE: &str = "https://data.dtodo.cn/api/client/v1";
const VERSION_TTL: Duration = Duration::from_secs(300);

struct VersionCache {
    version: String,
    fetched_at: Instant,
}

/// 海克斯 / aramgg 公共 API 客户端
pub struct HextechClient {
    client: Client,
    version_cache: RwLock<Option<VersionCache>>,
}

impl HextechClient {
    pub fn new() -> Self {
        Self::with_client(crate::http_client::get_public_client().clone())
    }

    pub fn with_client(client: Client) -> Self {
        Self {
            client,
            version_cache: RwLock::new(None),
        }
    }

    pub async fn resolve_data_version(&self) -> Result<String, String> {
        if let Ok(guard) = self.version_cache.read() {
            if let Some(cache) = guard.as_ref() {
                if cache.fetched_at.elapsed() < VERSION_TTL {
                    return Ok(cache.version.clone());
                }
            }
        }

        let url = format!("{BASE}/config");
        log::info!("🌐 请求海克斯 config: {url}");
        let data = self.get_json(&url).await?;
        let version = data
            .get("dataVersion")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "config 缺少 dataVersion".to_string())?
            .to_string();

        if let Ok(mut guard) = self.version_cache.write() {
            *guard = Some(VersionCache {
                version: version.clone(),
                fetched_at: Instant::now(),
            });
        }
        Ok(version)
    }

    pub async fn get_champions(&self, version: &str) -> Result<Value, String> {
        let url = format!("{BASE}/data/{version}/champions.json");
        log::info!("🌐 请求海克斯强度榜: {url}");
        self.get_json(&url).await
    }

    pub async fn get_champion_detail(&self, version: &str, champion_id: i32) -> Result<Value, String> {
        let url = format!("{BASE}/data/{version}/champions/{champion_id}.json");
        log::info!("🌐 请求海克斯英雄详情: {url}");
        self.get_json(&url).await
    }

    pub async fn get_augments(&self, version: &str) -> Result<Value, String> {
        let url = format!("{BASE}/data/{version}/augments.json");
        log::info!("🌐 请求海克斯增强列表: {url}");
        self.get_json(&url).await
    }

    async fn get_json(&self, url: &str) -> Result<Value, String> {
        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| format!("网络请求失败: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("API 请求失败: HTTP {}", response.status()));
        }

        response
            .json::<Value>()
            .await
            .map_err(|e| format!("JSON 解析失败: {e}"))
    }
}

impl Default for HextechClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_cache_starts_empty() {
        let client = HextechClient::new();
        assert!(client.version_cache.read().unwrap().is_none());
    }
}
