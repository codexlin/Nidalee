use crate::shared::types::{MatchInfo, MatchmakingState, PlayerInfo};
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

/// 获取当前对局信息
pub async fn get_match_info(client: &Client) -> Result<MatchInfo, String> {
    let session: serde_json::Value = lcu_get(client, "/lol-champ-select/v1/session").await?;

    // 解析对局信息
    let match_id = session["gameId"].as_str().unwrap_or("unknown").to_string();

    let mut players = Vec::new();
    if let Some(team_members) = session["myTeam"].as_array() {
        for member in team_members {
            let summoner_name = member["summonerName"].as_str().unwrap_or("Unknown").to_string();
            let champion_id = member["championId"].as_i64().unwrap_or(0) as i32;
            let team_id = member["team"].as_i64().unwrap_or(0) as i32;

            players.push(PlayerInfo {
                summoner_name,
                champion_id,
                team_id,
            });
        }
    }

    Ok(MatchInfo { match_id, players })
}
