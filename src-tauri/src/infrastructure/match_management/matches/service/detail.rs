use reqwest::Client;

use crate::shared::types::GameDetail;

pub async fn get_game_detail_logic(client: &Client, game_id: u64) -> Result<GameDetail, String> {
    let raw_detail = super::super::fetcher::lcu_fetcher(client)
        .fetch_game_detail(game_id)
        .await
        .map_err(|error| format!("获取游戏详细信息失败: {error}"))?;

    super::detail_dto::map_game_detail(raw_detail)
}
