use crate::infrastructure::game_session::connection::service::ConnectionInfo;
use crate::infrastructure::game_session::connection::service::ConnectionManager;
use crate::shared::types::ConnectionState;
use std::sync::Arc;
use tauri;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn get_connection_state(
    manager: tauri::State<'_, Arc<RwLock<ConnectionManager>>>,
) -> Result<ConnectionInfo, String> {
    let manager = manager.read().await;
    Ok(manager.get_connection_info().await)
}

#[tauri::command]
pub async fn force_refresh_connection(manager: tauri::State<'_, Arc<RwLock<ConnectionManager>>>) -> Result<(), String> {
    let manager = manager.read().await;
    manager.force_refresh().await;
    Ok(())
}

#[tauri::command]
pub async fn check_connection_state_command(
    manager: tauri::State<'_, Arc<RwLock<ConnectionManager>>>,
) -> Result<ConnectionState, String> {
    let manager = manager.read().await;
    let state = manager.check_connection_state().await;
    Ok(state)
}
