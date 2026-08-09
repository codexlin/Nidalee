use super::service;
use crate::http_client;
use crate::infrastructure::match_management::matches::analysis_service;
use crate::infrastructure::match_management::matches::service as matches_service;
use crate::shared::types::{PlayerMatchStats, SummonerInfo, SummonerWithMatches};

#[tauri::command]
pub async fn get_recent_matches_by_puuid(puuid: String, count: Option<usize>) -> Result<PlayerMatchStats, String> {
    let client = http_client::get_lcu_client();
    let count = count.unwrap_or(20);
    // 用户主动查询，不过滤队列类型
    matches_service::get_recent_matches_by_puuid(&client, &puuid, count, None).await
}

#[tauri::command]
pub async fn get_current_summoner() -> Result<SummonerInfo, String> {
    let client = http_client::get_lcu_client();
    service::get_current_summoner(client).await
}

#[tauri::command]
pub async fn get_summoner_by_id(id: u64) -> Result<Option<SummonerInfo>, String> {
    let client = http_client::get_lcu_client();
    match service::get_summoner_by_id(client, id).await {
        Ok(info) => Ok(Some(info)),
        Err(e) => {
            if e.contains("404") {
                Ok(None)
            } else {
                Err(e)
            }
        }
    }
}

#[tauri::command]
pub async fn get_summoners_and_histories(
    names: Vec<String>,
    count: Option<usize>,
) -> Result<Vec<SummonerWithMatches>, String> {
    let client = http_client::get_lcu_client();
    let mut summoners = service::get_summoners_by_names(client, names)
        .await
        .map_err(|e| format!("批量获取召唤师信息失败: {}", e))?;
    let mut result = Vec::new();
    for summoner in &mut summoners {
        let puuid = summoner.puuid.clone();
        if !puuid.is_empty() {
            service::fill_summoner_extra_info(client, summoner).await;
            // 一次 analyze：同时投影基础统计与位置分组，避免搜索页二次拉 LCU
            let game_count = count.unwrap_or(20) as u32;
            let request = analysis_service::legacy_overview_request(game_count, None, None);
            match analysis_service::analyze_matches_for_puuid(client, &puuid, &request).await {
                Ok(analysis) => {
                    result.push(SummonerWithMatches {
                        display_name: summoner.display_name.clone(),
                        summoner_info: summoner.clone(),
                        matches: analysis_service::to_player_match_stats(&analysis),
                        position_analysis: Some(analysis_service::to_multi_position_analysis(&analysis)),
                    });
                }
                Err(e) => {
                    log::warn!("Failed to fetch matches for {}: {}", summoner.display_name, e);
                    result.push(SummonerWithMatches {
                        display_name: summoner.display_name.clone(),
                        summoner_info: summoner.clone(),
                        matches: PlayerMatchStats::default(),
                        position_analysis: None,
                    });
                }
            }
        }
    }
    Ok(result)
}

#[tauri::command]
pub async fn set_summoner_background_skin(skin_id: u64) -> Result<(), String> {
    let client = http_client::get_lcu_client();
    service::set_summoner_background(client, skin_id).await
}

#[tauri::command]
pub async fn set_summoner_chat_profile(
    status_message: Option<String>,
    queue: Option<String>,
    tier: Option<String>,
    division: Option<String>,
) -> Result<(), String> {
    let client = http_client::get_lcu_client();
    service::set_summoner_chat_profile(client, status_message, queue, tier, division).await
}
