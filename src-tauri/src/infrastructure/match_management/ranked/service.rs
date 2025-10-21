use crate::shared::types::RankedStats;
use crate::shared::utils::lcu_get;
use reqwest::Client;

/// 获取当前召唤师的段位统计
#[allow(dead_code)]
pub async fn get_current_ranked_stats(client: &Client) -> Result<RankedStats, String> {
    lcu_get(client, "/lol-ranked/v1/current-ranked-stats").await
}

/// 根据召唤师ID获取段位统计
#[allow(dead_code)]
pub async fn get_ranked_stats_by_id(client: &Client, summoner_id: u64) -> Result<RankedStats, String> {
    let url = format!("/lol-ranked/v1/ranked-stats/{}", summoner_id);
    lcu_get(client, &url).await
}
