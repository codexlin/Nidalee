use super::service;
use crate::http_client;
use crate::shared::types::RunePage;

#[tauri::command]
pub async fn get_current_rune_page() -> Result<Option<RunePage>, String> {
    service::get_current_rune_page(http_client::get_lcu_client()).await
}

#[tauri::command]
pub async fn apply_custom_runes(
    config_name: String,
    primary_style_id: i32,
    sub_style_id: i32,
    selected_perk_ids: Vec<i32>,
) -> Result<String, String> {
    if selected_perk_ids.len() != 9 {
        return Err(format!("符文数量错误：需要 9 个，实际 {} 个", selected_perk_ids.len()));
    }
    service::apply_rune_build(
        http_client::get_lcu_client(),
        &config_name,
        primary_style_id,
        sub_style_id,
        selected_perk_ids,
    )
    .await
}
