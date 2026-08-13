use tauri::AppHandle;

use super::shortcut;
use super::state;
use super::types::AugmentDetectedPayload;
use super::window;

#[tauri::command]
pub async fn set_augment_overlay_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    state::set_enabled(app, enabled).await;
    Ok(())
}

#[tauri::command]
pub async fn hide_augment_overlay(app: AppHandle) -> Result<(), String> {
    state::hide_overlay(app).await;
    Ok(())
}

#[tauri::command]
pub async fn hide_augment_card_overlay(app: AppHandle) -> Result<(), String> {
    state::dismiss_card_overlay(app).await;
    Ok(())
}

#[tauri::command]
pub async fn hide_augment_side_panel(app: AppHandle) -> Result<(), String> {
    state::hide_side_panel(app).await;
    Ok(())
}

#[tauri::command]
pub async fn toggle_augment_side_panel(app: AppHandle) -> Result<(), String> {
    state::toggle_side_panel(app).await;
    Ok(())
}

#[tauri::command]
pub async fn ensure_augment_overlay_window(app: AppHandle) -> Result<(), String> {
    window::ensure_window(&app)
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

#[tauri::command]
pub async fn get_augment_overlay_ocr_enabled() -> Result<bool, String> {
    Ok(state::is_ocr_enabled().await)
}

#[tauri::command]
pub async fn set_augment_overlay_ocr_enabled(app: AppHandle, enabled: bool) -> Result<bool, String> {
    state::set_ocr_enabled(app, enabled).await;
    Ok(state::is_ocr_enabled().await)
}
