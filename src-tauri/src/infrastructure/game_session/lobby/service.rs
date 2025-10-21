use crate::shared::types::LobbyInfo;
use crate::shared::utils::{lcu_get, lcu_post_no_content};
use reqwest::Client;
use serde_json::json;

/// 获取当前 Lobby 信息
pub async fn get_lobby_info(client: &Client) -> Result<LobbyInfo, String> {
    lcu_get(client, "/lol-lobby/v2/lobby").await
}

/// 发送房间聊天消息
///
/// # 参数
/// * `client` - HTTP 客户端
/// * `chat_id` - 聊天室 ID（从 LobbyInfo.multi_user_chat_id 获取）
/// * `message` - 消息内容
pub async fn send_chat_message(client: &Client, chat_id: &str, message: &str) -> Result<(), String> {
    let url = format!("/lol-chat/v1/conversations/{}/messages", chat_id);
    let payload = json!({
        "body": message,
        "type": "chat"
    });

    lcu_post_no_content(client, &url, payload).await?;
    log::info!(
        target: "lobby::service",
        "Chat message sent to room {}: '{}'",
        chat_id,
        message
    );

    Ok(())
}

/// 从 Lobby 信息中提取聊天室 ID
pub fn get_chat_id_from_lobby(lobby: &LobbyInfo) -> Option<String> {
    if lobby.multi_user_chat_id.is_empty() {
        None
    } else {
        Some(lobby.multi_user_chat_id.clone())
    }
}

/// 发送房间系统通知（带格式的消息）
pub async fn send_formatted_message(client: &Client, chat_id: &str, prefix: &str, content: &str) -> Result<(), String> {
    let message = format!("[{}] {}", prefix, content);
    send_chat_message(client, chat_id, &message).await
}
