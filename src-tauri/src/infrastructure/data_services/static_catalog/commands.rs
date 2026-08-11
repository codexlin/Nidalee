use super::service::{ensure_static_catalogs, get_static_meta, refresh_static_catalogs_if_stale, StaticCatalogMeta};

/// 获取当前已加载的静态目录元信息（必要时等待首载完成）
#[tauri::command]
pub async fn get_static_catalog_meta() -> Result<StaticCatalogMeta, String> {
    ensure_static_catalogs().await?;
    get_static_meta().ok_or_else(|| "静态目录元信息不可用".to_string())
}

/// 探测最新版本；若变化则刷新英雄/技能目录。返回是否发生了刷新。
#[tauri::command]
pub async fn refresh_static_catalogs() -> Result<bool, String> {
    // 若尚未首载，先完成首载再比较
    let _ = ensure_static_catalogs().await;
    refresh_static_catalogs_if_stale().await
}
