use super::client::HextechClient;
use super::parser::{parse_champion_detail_merging_catalog, parse_tier_list};
use super::types::{HextechChampionDetail, HextechTierList};
use once_cell::sync::Lazy;
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::Mutex;

static HEXTECH_CLIENT: Lazy<HextechClient> =
    Lazy::new(|| HextechClient::with_client(crate::http_client::get_public_client().clone()));

struct HextechDataCache {
    version: String,
    catalog: Option<Value>,
    details: HashMap<i32, HextechChampionDetail>,
}

static DATA_CACHE: Lazy<Mutex<HextechDataCache>> = Lazy::new(|| {
    Mutex::new(HextechDataCache {
        version: String::new(),
        catalog: None,
        details: HashMap::new(),
    })
});

pub async fn get_tier_list() -> Result<HextechTierList, String> {
    let version = HEXTECH_CLIENT.resolve_data_version().await?;
    let raw = HEXTECH_CLIENT.get_champions(&version).await?;
    parse_tier_list(raw)
}

pub async fn get_champion_detail(champion_id: i32) -> Result<HextechChampionDetail, String> {
    if champion_id <= 0 {
        return Err("无效的英雄 ID".to_string());
    }
    let version = HEXTECH_CLIENT.resolve_data_version().await?;
    {
        let cache = DATA_CACHE.lock().await;
        if cache.version == version {
            if let Some(detail) = cache.details.get(&champion_id) {
                return Ok(detail.clone());
            }
        }
    }

    let (detail_res, catalog) = tokio::join!(
        HEXTECH_CLIENT.get_champion_detail(&version, champion_id),
        cached_catalog(&version)
    );
    let parsed = parse_champion_detail_merging_catalog(detail_res?, &catalog?)?;

    let mut cache = DATA_CACHE.lock().await;
    reset_cache_if_version_changed(&mut cache, &version);
    cache.details.insert(champion_id, parsed.clone());
    Ok(parsed)
}

async fn cached_catalog(version: &str) -> Result<Value, String> {
    {
        let cache = DATA_CACHE.lock().await;
        if cache.version == version {
            if let Some(catalog) = &cache.catalog {
                return Ok(catalog.clone());
            }
        }
    }

    let raw = HEXTECH_CLIENT.get_augments(version).await?;
    let mut cache = DATA_CACHE.lock().await;
    reset_cache_if_version_changed(&mut cache, version);
    cache.catalog = Some(raw.clone());
    Ok(raw)
}

fn reset_cache_if_version_changed(cache: &mut HextechDataCache, version: &str) {
    if cache.version == version {
        return;
    }
    cache.version = version.to_string();
    cache.catalog = None;
    cache.details.clear();
}
