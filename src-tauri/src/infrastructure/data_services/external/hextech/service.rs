use super::client::HextechClient;
use super::parser::{parse_champion_detail_merging_catalog, parse_tier_list};
use super::types::{HextechChampionDetail, HextechTierList};

pub async fn get_tier_list() -> Result<HextechTierList, String> {
    let client = HextechClient::new();
    let version = client.resolve_data_version().await?;
    let raw = client.get_champions(&version).await?;
    parse_tier_list(raw)
}

pub async fn get_champion_detail(champion_id: i32) -> Result<HextechChampionDetail, String> {
    if champion_id <= 0 {
        return Err("无效的英雄 ID".to_string());
    }
    let client = HextechClient::new();
    let version = client.resolve_data_version().await?;

    let (detail_res, catalog_res) = tokio::join!(
        client.get_champion_detail(&version, champion_id),
        client.get_augments(&version)
    );
    let detail = detail_res?;
    let catalog = catalog_res.unwrap_or_else(|err| {
        log::warn!("海克斯增强目录拉取失败，三连名称可能不完整: {err}");
        serde_json::json!({ "data": [] })
    });
    parse_champion_detail_merging_catalog(detail, &catalog)
}
