use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::time::{Duration, Instant};

static LIVECLIENT_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(5))
        .build()
        .expect("valid LiveClient HTTP configuration")
});
static LIVECLIENT_PORT: Lazy<Mutex<Option<(u16, Instant)>>> = Lazy::new(|| Mutex::new(None));
const LIVECLIENT_PORT_TTL: Duration = Duration::from_secs(30);

fn cached_liveclient_port() -> Option<u16> {
    LIVECLIENT_PORT.lock().ok().and_then(|cache| {
        cache
            .as_ref()
            .filter(|(_, cached_at)| cached_at.elapsed() < LIVECLIENT_PORT_TTL)
            .map(|(port, _)| *port)
    })
}

fn cache_liveclient_port(port: u16) {
    if let Ok(mut cache) = LIVECLIENT_PORT.lock() {
        *cache = Some((port, Instant::now()));
    }
}

fn invalidate_liveclient_port() {
    if let Ok(mut cache) = LIVECLIENT_PORT.lock() {
        *cache = None;
    }
}

async fn test_liveclient_port(port: u16) -> bool {
    let url = format!("https://127.0.0.1:{port}/liveclientdata/playerlist");
    LIVECLIENT_CLIENT
        .get(url)
        .send()
        .await
        .is_ok_and(|response| response.status().is_success())
}

async fn detect_liveclient_port() -> Option<u16> {
    if let Some(port) = cached_liveclient_port() {
        if test_liveclient_port(port).await {
            return Some(port);
        }
        invalidate_liveclient_port();
    }

    for port in [2999, 2998, 3000, 3001, 2997] {
        if test_liveclient_port(port).await {
            cache_liveclient_port(port);
            return Some(port);
        }
    }
    None
}

pub async fn get_live_player_list() -> Result<Vec<crate::shared::types::LiveClientPlayer>, String> {
    let port = detect_liveclient_port()
        .await
        .ok_or_else(|| "LiveClient 不可用，请确认已进入对局".to_owned())?;
    let url = format!("https://127.0.0.1:{port}/liveclientdata/playerlist");
    let response = LIVECLIENT_CLIENT.get(url).send().await.map_err(|error| {
        invalidate_liveclient_port();
        format!("LiveClient 请求失败: {error}")
    })?;
    if !response.status().is_success() {
        invalidate_liveclient_port();
        return Err(format!("LiveClient 返回错误状态: {}", response.status()));
    }

    let raw_body = response
        .text()
        .await
        .map_err(|error| format!("读取 LiveClient 玩家列表失败: {error}"))?;
    let players = serde_json::from_str::<Vec<crate::shared::types::LiveClientPlayer>>(&raw_body)
        .map_err(|error| format!("解析 LiveClient 玩家列表失败: {error}"))?;
    log::debug!("loaded {} players", players.len());
    Ok(players)
}
