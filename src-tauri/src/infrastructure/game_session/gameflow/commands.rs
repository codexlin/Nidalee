use crate::infrastructure::data_services::static_catalog::{
    ensure_static_catalogs, fetch_ddragon_version, get_static_meta,
};

#[tauri::command]
pub async fn get_game_version() -> Result<String, String> {
    if let Some(meta) = get_static_meta() {
        return Ok(meta.version);
    }
    // 尽量走静态包确保（离线可 hydrate 磁盘）
    let _ = ensure_static_catalogs().await;
    if let Some(meta) = get_static_meta() {
        return Ok(meta.version);
    }
    fetch_ddragon_version()
        .await
        .ok_or_else(|| "无法获取游戏版本（网络不可用且无本地静态包）".to_string())
}
