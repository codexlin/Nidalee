use crate::shared::utils::{lcu_get, lcu_request_json};
use reqwest::{Client, Method};
use serde_json::{json, Value};

pub async fn get_champ_select_session_raw(client: &Client) -> Result<Value, String> {
    lcu_get(client, "/lol-champ-select/v1/session").await
}

async fn champion_action(client: &Client, action_id: u64, champion_id: u64, completed: bool) -> Result<(), String> {
    let path = format!("/lol-champ-select/v1/session/actions/{action_id}");
    let body = json!({ "championId": champion_id, "completed": completed });
    lcu_request_json::<Value>(client, Method::PATCH, &path, Some(body))
        .await
        .map(|_| ())
        .map_err(|error| format!("更新英雄选择失败: {error}"))
}

pub async fn pick_champion(client: &Client, action_id: u64, champion_id: u64, completed: bool) -> Result<(), String> {
    champion_action(client, action_id, champion_id, completed).await
}

pub async fn ban_champion(client: &Client, action_id: u64, champion_id: u64) -> Result<(), String> {
    champion_action(client, action_id, champion_id, true).await
}
