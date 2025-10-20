use super::service;
use crate::http_client;

/// 获取当前房间信息
#[tauri::command]
pub async fn get_current_lobby() -> Result<crate::shared::types::LobbyInfo, String> {
    let client = http_client::get_lcu_client();
    service::get_lobby_info(client).await
}

/// 发送房间聊天消息
///
/// # 参数
/// * `chat_id` - 聊天室 ID（从 lobby.multiUserChatId 获取）
/// * `message` - 消息内容
#[tauri::command]
pub async fn send_lobby_chat_message(chat_id: String, message: String) -> Result<(), String> {
    let client = http_client::get_lcu_client();
    service::send_chat_message(client, &chat_id, &message).await
}

/// 发送格式化消息（带前缀）
///
/// # 示例
/// `send_lobby_formatted_message(chat_id, "Nidalee", "推荐选择亚索")`
/// → 发送: `[Nidalee] 推荐选择亚索`
#[tauri::command]
pub async fn send_lobby_formatted_message(chat_id: String, prefix: String, content: String) -> Result<(), String> {
    let client = http_client::get_lcu_client();
    service::send_formatted_message(client, &chat_id, &prefix, &content).await
}
