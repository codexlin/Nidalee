use crate::shared::types::MatchmakingState;
use crate::shared::utils::{lcu_delete, lcu_get, lcu_post};
use reqwest::Client;
use serde_json::Value;

/// 开始匹配
pub async fn start_matchmaking(client: &Client) -> Result<(), String> {
    lcu_post::<Value>(client, "/lol-lobby/v2/lobby/matchmaking/search", Value::Null).await?;
    Ok(())
}

/// 停止匹配
pub async fn stop_matchmaking(client: &Client) -> Result<(), String> {
    lcu_delete::<Value>(client, "/lol-lobby/v2/lobby/matchmaking/search").await?;
    Ok(())
}

/// 接受匹配
pub async fn accept_match(client: &Client) -> Result<(), String> {
    lcu_post::<Value>(client, "/lol-matchmaking/v1/ready-check/accept", Value::Null).await?;
    Ok(())
}

/// 拒绝匹配
pub async fn decline_match(client: &Client) -> Result<(), String> {
    lcu_post::<Value>(client, "/lol-matchmaking/v1/ready-check/decline", Value::Null).await?;
    Ok(())
}

/// 获取当前匹配状态
pub async fn get_matchmaking_state(client: &Client) -> Result<MatchmakingState, String> {
    lcu_get(client, "/lol-lobby/v2/lobby/matchmaking/search-state").await
}
