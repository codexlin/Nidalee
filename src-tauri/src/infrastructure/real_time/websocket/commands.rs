use crate::infrastructure::real_time::websocket::service;

#[tauri::command]
pub async fn start_lcu_ws(app: tauri::AppHandle) -> Result<(), String> {
    if service::start_ws(app).await {
        Ok(())
    } else {
        service::sync_snapshot().await
    }
}

#[tauri::command]
pub async fn stop_lcu_ws() -> Result<(), String> {
    service::stop_ws().await;
    Ok(())
}
