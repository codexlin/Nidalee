use crate::shared::utils::{lcu_get, lcu_post, lcu_put};
use crate::shared::types::{RankInfo, SummonerInfo};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{to_value, Value};

// 以下内容为原 summoner.rs 全部内容，粘贴至此

// 生涯背景设置请求体（正确的API格式）
#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileUpdateRequest {
    pub key: String,
    pub value: u64,
}

pub async fn get_current_summoner(client: &Client) -> Result<SummonerInfo, String> {
    let mut summoner_info: SummonerInfo = lcu_get(client, "/lol-summoner/v1/current-summoner").await?;
    // 获取段位信息
    fill_summoner_extra_info(client, &mut summoner_info).await;
    Ok(summoner_info)
}
// 补全信息
pub async fn fill_summoner_extra_info(client: &Client, summoner_info: &mut SummonerInfo) {
    if let Ok(rank_info) = get_rank_info(client, &summoner_info.puuid).await {
        summoner_info.solo_rank_tier = rank_info.solo_tier;
        summoner_info.solo_rank_division = rank_info.solo_division;
        summoner_info.solo_rank_lp = rank_info.solo_lp;
        summoner_info.solo_rank_wins = rank_info.solo_wins;
        summoner_info.solo_rank_losses = rank_info.solo_losses;
        summoner_info.flex_rank_tier = rank_info.flex_tier;
        summoner_info.flex_rank_division = rank_info.flex_division;
        summoner_info.flex_rank_lp = rank_info.flex_lp;
        summoner_info.flex_rank_wins = rank_info.flex_wins;
        summoner_info.flex_rank_losses = rank_info.flex_losses;
    }

    if let (Some(game_name), Some(tag_line)) = (summoner_info.game_name.clone(), summoner_info.tag_line.clone()) {
        summoner_info.display_name = format!("{}#{}", game_name, tag_line);
    }
}
pub async fn get_rank_info(client: &Client, puuid: &str) -> Result<RankInfo, String> {
    let path = &format!("/lol-ranked/v1/ranked-stats/{}", puuid);
    let rank_data: Value = lcu_get(client, path).await?;

    let mut rank_info = RankInfo::default();
    if let Some(queues) = rank_data.get("queues").and_then(|q| q.as_array()) {
        for queue in queues {
            let queue_type = queue.get("queueType").and_then(|q| q.as_str()).unwrap_or("");
            match queue_type {
                "RANKED_SOLO_5x5" => {
                    rank_info.solo_tier = queue.get("tier").and_then(|t| t.as_str()).map(String::from);
                    rank_info.solo_division = queue.get("division").and_then(|d| d.as_str()).map(String::from);
                    rank_info.solo_lp = queue.get("leaguePoints").and_then(|l| l.as_i64()).map(|l| l as i32);
                    rank_info.solo_wins = queue.get("wins").and_then(|w| w.as_i64()).map(|w| w as i32);
                    rank_info.solo_losses = queue.get("losses").and_then(|l| l.as_i64()).map(|l| l as i32);
                }
                "RANKED_FLEX_SR" => {
                    rank_info.flex_tier = queue.get("tier").and_then(|t| t.as_str()).map(String::from);
                    rank_info.flex_division = queue.get("division").and_then(|d| d.as_str()).map(String::from);
                    rank_info.flex_lp = queue.get("leaguePoints").and_then(|l| l.as_i64()).map(|l| l as i32);
                    rank_info.flex_wins = queue.get("wins").and_then(|w| w.as_i64()).map(|w| w as i32);
                    rank_info.flex_losses = queue.get("losses").and_then(|l| l.as_i64()).map(|l| l as i32);
                }
                _ => {}
            }
        }
    }
    Ok(rank_info)
}

// 获取指定ID的召唤师
pub async fn get_summoner_by_id(client: &Client, summoner_id: u64) -> Result<SummonerInfo, String> {
    let path = &format!("/lol-summoner/v1/summoners/{}", summoner_id);
    let mut summoner_info: SummonerInfo = lcu_get(client, path).await?;

    // 获取段位信息
    fill_summoner_extra_info(client, &mut summoner_info).await;

    Ok(summoner_info)
}

// 批量获取召唤师信息
pub async fn get_summoners_by_names(client: &Client, names: Vec<String>) -> Result<Vec<SummonerInfo>, String> {
    let path = &format!("/lol-summoner/v2/summoners/names");
    let summoners: Vec<SummonerInfo> = lcu_post(client, path, names.into()).await?;
    Ok(summoners)
}

// 设置生涯背景皮肤（使用正确的API - POST请求）
pub async fn set_summoner_background(client: &Client, skin_id: u64) -> Result<(), String> {
    let path = "/lol-summoner/v1/current-summoner/summoner-profile";

    let request_body = ProfileUpdateRequest {
        key: "backgroundSkinId".to_string(),
        value: skin_id,
    };

    // 将结构体序列化为 serde_json::Value
    let body_value = serde_json::to_value(request_body).map_err(|e| format!("序列化请求体失败: {}", e))?;

    // 使用 POST 请求而不是 PUT
    match lcu_post::<Value>(client, path, body_value).await {
        Ok(response) => {
            println!("成功设置生涯背景皮肤，皮肤ID: {}", skin_id);
            if !response.is_null() {
                println!("返回的 profile 信息: {}", response);
            }
            Ok(())
        }
        Err(e) => {
            println!("设置生涯背景皮肤失败: {}", e);
            // 尝试解析错误信息
            if let Ok(error_json) = serde_json::from_str::<Value>(&e) {
                if let Some(error_code) = error_json.get("errorCode") {
                    if let Some(message) = error_json.get("message") {
                        return Err(format!("设置背景失败 [{}]: {}", error_code, message));
                    }
                }
            }
            Err(format!("设置生涯背景失败: {}", e))
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatProfileLolInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rankedLeagueQueue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rankedLeagueTier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rankedLeagueDivision: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ChatProfileUpdateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statusMessage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lol: Option<ChatProfileLolInfo>,
}

/// 通用设置聊天资料（签名/段位信息，均可选）
pub async fn set_summoner_chat_profile(
    client: &Client,
    status_message: Option<String>,
    queue: Option<String>,
    tier: Option<String>,
    division: Option<String>,
) -> Result<(), String> {
    let path = "/lol-chat/v1/me";
    let lol = if queue.is_some() || tier.is_some() || division.is_some() {
        Some(ChatProfileLolInfo {
            rankedLeagueQueue: queue,
            rankedLeagueTier: tier,
            rankedLeagueDivision: division,
        })
    } else {
        None
    };
    let body = ChatProfileUpdateRequest {
        statusMessage: status_message,
        lol,
    };
    lcu_put::<serde_json::Value>(client, path, to_value(body).unwrap())
        .await
        .map(|_| ())
}
