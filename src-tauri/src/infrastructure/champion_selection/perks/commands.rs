use super::service;
use crate::http_client;
use crate::shared::types::{Perk, RunePage, RuneStyle};

#[tauri::command]
pub async fn get_lcu_rune_styles() -> Result<Vec<RuneStyle>, String> {
    let client = http_client::get_lcu_client();
    service::list_all_styles(&client).await
}

#[tauri::command]
pub async fn get_lcu_perks() -> Result<Vec<Perk>, String> {
    let client = http_client::get_lcu_client();
    service::list_all_perks(&client).await
}

#[tauri::command]
pub async fn get_lcu_perk_icon(icon_path: String) -> Result<Vec<u8>, String> {
    let client = http_client::get_lcu_client();
    service::get_perk_icon(&client, &icon_path).await
}

/// 获取当前活跃的符文页面
#[tauri::command]
pub async fn get_current_rune_page() -> Result<Option<RunePage>, String> {
    let client = http_client::get_lcu_client();
    service::get_current_rune_page(&client).await
}

/// 应用用户自定义符文配置
///
/// 参数：
/// - champion_name: 英雄名称（用于符文页命名）
/// - primary_style_id: 主系符文 ID (8000, 8100, 8200, 8300, 8400)
/// - sub_style_id: 副系符文 ID
/// - selected_perk_ids: 选中的符文 ID 数组（9个）
#[tauri::command]
pub async fn apply_custom_runes(
    champion_name: String,
    primary_style_id: i32,
    sub_style_id: i32,
    selected_perk_ids: Vec<i32>,
) -> Result<String, String> {
    log::info!("🔮 应用用户自定义符文配置");
    log::info!("🔮 英雄: {}", champion_name);
    log::info!("🔮 主系: {}, 副系: {}", primary_style_id, sub_style_id);
    log::info!("🔮 符文IDs: {:?}", selected_perk_ids);

    // 验证符文数量
    if selected_perk_ids.len() != 9 {
        return Err(format!("符文数量错误：期望 9 个，实际 {} 个", selected_perk_ids.len()));
    }

    let client = http_client::get_lcu_client();

    match service::apply_rune_build(
        &client,
        &champion_name,
        primary_style_id,
        sub_style_id,
        selected_perk_ids,
    )
    .await
    {
        Ok(message) => Ok(format!("✨ 用户自定义符文应用成功！{}", message)),
        Err(e) => Err(format!("用户自定义符文应用失败: {}", e)),
    }
}
