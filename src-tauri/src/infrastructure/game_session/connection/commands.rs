use super::service::ConnectionManager;
use crate::shared::types::ConnectionState;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn check_connection_state_command(
    manager: tauri::State<'_, Arc<RwLock<ConnectionManager>>>,
) -> Result<ConnectionState, String> {
    Ok(manager.read().await.check_connection_state().await)
}
