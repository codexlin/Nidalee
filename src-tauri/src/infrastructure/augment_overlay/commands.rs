use tauri::AppHandle;

use super::shortcut;
use super::state;
use super::types::AugmentDetectedPayload;

#[tauri::command]
pub async fn hide_augment_side_panel(app: AppHandle) -> Result<(), String> {
    state::hide_side_panel(app).await;
    Ok(())
}

#[tauri::command]
pub async fn get_augment_overlay_snapshot() -> Result<Option<AugmentDetectedPayload>, String> {
    Ok(state::snapshot().await)
}

#[tauri::command]
pub async fn get_augment_overlay_visible() -> Result<bool, String> {
    Ok(state::is_visible().await)
}

#[tauri::command]
pub fn get_augment_overlay_shortcut() -> Result<String, String> {
    Ok(shortcut::current())
}

#[tauri::command]
pub fn set_augment_overlay_shortcut(app: AppHandle, shortcut: String) -> Result<String, String> {
    shortcut::register(&app, &shortcut)
}
