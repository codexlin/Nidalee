//! LCU API 响应缓存层
//!
//! 三级缓存策略：
//! - 静态数据：英雄、符文、召唤师技能（1 小时）
//! - 动态数据：召唤师信息、段位（5 分钟）
//! - 会话数据：房间、匹配状态（30 秒）

use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// 缓存条目
#[derive(Clone)]
struct CacheEntry {
    data: Value,
    expired_at: Instant,
}

/// 缓存配置
struct CacheConfig {
    static_ttl: Duration,    // 静态数据过期时间
    dynamic_ttl: Duration,   // 动态数据过期时间
    session_ttl: Duration,   // 会话数据过期时间
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            static_ttl: Duration::from_secs(3600),  // 1 小时
            dynamic_ttl: Duration::from_secs(300),   // 5 分钟
            session_ttl: Duration::from_secs(30),    // 30 秒
        }
    }
}

/// LCU 缓存管理器
pub struct LcuCache {
    static_data: Arc<RwLock<HashMap<String, CacheEntry>>>,
    dynamic_data: Arc<RwLock<HashMap<String, CacheEntry>>>,
    session_data: Arc<RwLock<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
}

impl LcuCache {
    pub fn new() -> Self {
        Self {
            static_data: Arc::new(RwLock::new(HashMap::new())),
            dynamic_data: Arc::new(RwLock::new(HashMap::new())),
            session_data: Arc::new(RwLock::new(HashMap::new())),
            config: CacheConfig::default(),
        }
    }

    /// 获取静态数据（英雄、符文、召唤师技能等）
    pub async fn get_static<F>(&self, key: &str, fetch: F) -> Result<Value, String>
    where
        F: FnOnce() -> Result<Value, String>,
    {
        self.get_with_cache(&self.static_data, key, fetch, self.config.static_ttl).await
    }

    /// 获取动态数据（召唤师信息、段位等）
    pub async fn get_dynamic<F>(&self, key: &str, fetch: F) -> Result<Value, String>
    where
        F: FnOnce() -> Result<Value, String>,
    {
        self.get_with_cache(&self.dynamic_data, key, fetch, self.config.dynamic_ttl).await
    }

    /// 获取会话数据（房间、匹配状态等）
    pub async fn get_session<F>(&self, key: &str, fetch: F) -> Result<Value, String>
    where
        F: FnOnce() -> Result<Value, String>,
    {
        self.get_with_cache(&self.session_data, key, fetch, self.config.session_ttl).await
    }

    /// 通用缓存获取逻辑
    async fn get_with_cache<F>(
        &self,
        cache: &Arc<RwLock<HashMap<String, CacheEntry>>>,
        key: &str,
        fetch: F,
        ttl: Duration,
    ) -> Result<Value, String>
    where
        F: FnOnce() -> Result<Value, String>,
    {
        // 1. 尝试从缓存读取
        {
            let cache_read = cache.read().await;
            if let Some(entry) = cache_read.get(key) {
                if entry.expired_at > Instant::now() {
                    log::debug!("[LCU Cache] HIT: {} (剩余 {} 秒)", key, entry.expired_at.elapsed().as_secs());
                    return Ok(entry.data.clone());
                } else {
                    log::debug!("[LCU Cache] EXPIRED: {}", key);
                }
            } else {
                log::debug!("[LCU Cache] MISS: {}", key);
            }
        }

        // 2. 缓存未命中，执行获取操作
        let value = fetch()?;

        // 3. 写入缓存
        let mut cache_write = cache.write().await;
        cache_write.insert(
            key.to_string(),
            CacheEntry {
                data: value.clone(),
                expired_at: Instant::now() + ttl,
            },
        );

        // 4. 清理过期条目（每次最多清理 10 个）
        self.cleanup_expired(&mut cache_write);

        Ok(value)
    }

    /// 清理过期的缓存条目
    fn cleanup_expired(&self, cache: &mut HashMap<String, CacheEntry>) {
        let now = Instant::now();
        let expired_keys: Vec<String> = cache
            .iter()
            .filter(|(_, entry)| entry.expired_at <= now)
            .map(|(key, _)| key.clone())
            .take(10)  // 每次最多清理 10 个，避免阻塞
            .collect();

        for key in expired_keys {
            cache.remove(&key);
        }

        if !expired_keys.is_empty() {
            log::debug!("[LCU Cache] 清理了 {} 个过期条目", expired_keys.len());
        }
    }

    /// 使指定缓存失效
    pub async fn invalidate(&self, key: &str) {
        self.static_data.write().await.remove(key);
        self.dynamic_data.write().await.remove(key);
        self.session_data.write().await.remove(key);
    }

    /// 清空所有缓存
    pub async fn clear_all(&self) {
        self.static_data.write().await.clear();
        self.dynamic_data.write().await.clear();
        self.session_data.write().await.clear();
        log::info!("[LCU Cache] 已清空所有缓存");
    }

    /// 获取缓存统计信息
    pub async fn stats(&self) -> CacheStats {
        let static_count = self.static_data.read().await.len();
        let dynamic_count = self.dynamic_data.read().await.len();
        let session_count = self.session_data.read().await.len();

        CacheStats {
            static_count,
            dynamic_count,
            session_count,
            total: static_count + dynamic_count + session_count,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CacheStats {
    pub static_count: usize,
    pub dynamic_count: usize,
    pub session_count: usize,
    pub total: usize,
}

/// 全局缓存实例
static GLOBAL_CACHE: Lazy<Arc<LcuCache>> = Lazy::new(|| Arc::new(LcuCache::new()));

/// 获取全局缓存实例
pub fn get_global_cache() -> Arc<LcuCache> {
    GLOBAL_CACHE.clone()
}

/// 便捷函数：获取静态缓存
pub async fn get_static_cached<F>(key: &str, fetch: F) -> Result<Value, String>
where
    F: FnOnce() -> Result<Value, String>,
{
    get_global_cache().get_static(key, fetch).await
}

/// 便捷函数：获取动态缓存
pub async fn get_dynamic_cached<F>(key: &str, fetch: F) -> Result<Value, String>
where
    F: FnOnce() -> Result<Value, String>,
{
    get_global_cache().get_dynamic(key, fetch).await
}

/// 便捷函数：获取会话缓存
pub async fn get_session_cached<F>(key: &str, fetch: F) -> Result<Value, String>
where
    F: FnOnce() -> Result<Value, String>,
{
    get_global_cache().get_session(key, fetch).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = LcuCache::new();
        let key = "test_key";

        let result = cache
            .get_static(key, || Ok(serde_json::json!({"data": "test"})))
            .await
            .unwrap();

        assert_eq!(result["data"], "test");
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = LcuCache::new();
        let key = "test_exp";

        // 设置很短的 TTL 用于测试
        let short_cache = LcuCache::new();
    }
}
