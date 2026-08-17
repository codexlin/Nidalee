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

async fn liveclient_get(path: &str) -> Result<String, String> {
    let port = detect_liveclient_port()
        .await
        .ok_or_else(|| "LiveClient 不可用，请确认已进入对局".to_owned())?;
    let url = format!("https://127.0.0.1:{port}{path}");
    let response = LIVECLIENT_CLIENT.get(url).send().await.map_err(|error| {
        invalidate_liveclient_port();
        format!("LiveClient 请求失败: {error}")
    })?;
    if !response.status().is_success() {
        invalidate_liveclient_port();
        return Err(format!("LiveClient 返回错误状态: {}", response.status()));
    }
    response
        .text()
        .await
        .map_err(|error| format!("读取 LiveClient 响应失败: {error}"))
}

pub async fn get_live_player_list() -> Result<Vec<crate::shared::types::LiveClientPlayer>, String> {
    let raw_body = liveclient_get("/liveclientdata/playerlist").await?;
    let players = parse_live_player_list(&raw_body)?;
    log::debug!("loaded {} players", players.len());
    Ok(players)
}

fn parse_live_player_list(raw_body: &str) -> Result<Vec<crate::shared::types::LiveClientPlayer>, String> {
    let mut players = serde_json::from_str::<Vec<crate::shared::types::LiveClientPlayer>>(raw_body)
        .map_err(|error| format!("解析 LiveClient 玩家列表失败: {error}"))?;
    players.iter_mut().for_each(|player| player.normalize_human_identity());
    Ok(players)
}

fn parse_json_string(raw: &str) -> String {
    serde_json::from_str::<String>(raw)
        .unwrap_or_else(|_| raw.trim().trim_matches('"').to_string())
        .trim()
        .to_string()
}

fn riot_id_matches(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    if left.eq_ignore_ascii_case(right) {
        return true;
    }
    let (left_name, left_has_tag) = left.split_once('#').map_or((left, false), |(name, _)| (name, true));
    let (right_name, right_has_tag) = right.split_once('#').map_or((right, false), |(name, _)| (name, true));
    (!left_has_tag || !right_has_tag) && left_name.eq_ignore_ascii_case(right_name)
}

/// Returns the local player's Riot ID from LiveClient without depending on LCU account APIs.
pub async fn get_active_player_name() -> Result<String, String> {
    let raw = liveclient_get("/liveclientdata/activeplayername").await?;
    let name = parse_json_string(&raw);
    if name.is_empty() {
        Err("LiveClient 当前玩家名称为空".to_string())
    } else {
        Ok(name)
    }
}

/// 对局里定位本地玩家（优先 playerlist，activeplayer 在部分客户端不可用）
pub async fn get_local_live_player() -> Result<crate::shared::types::LiveClientPlayer, String> {
    let players = get_live_player_list().await?;
    if players.is_empty() {
        return Err("LiveClient 玩家列表为空".to_string());
    }

    if let Ok(name) = get_active_player_name().await {
        if let Some(player) = players
            .iter()
            .find(|player| riot_id_matches(&player.summoner_name, &name))
        {
            return Ok(player.clone());
        }
    }

    if let Ok((champion_name, raw_champion_name)) = get_active_player_champion().await {
        if let Some(player) = players.iter().find(|player| {
            (!champion_name.is_empty() && player.champion_name == champion_name)
                || (!raw_champion_name.is_empty() && player.raw_champion_name == raw_champion_name)
        }) {
            return Ok(player.clone());
        }
    }

    let summoner = crate::infrastructure::data_services::summoner::service::get_current_summoner(
        crate::http_client::get_lcu_client(),
    )
    .await?;
    let local_name = match (&summoner.game_name, &summoner.tag_line) {
        (Some(game_name), Some(tag_line)) => format!("{game_name}#{tag_line}"),
        _ => summoner.display_name.clone(),
    };
    players
        .into_iter()
        .find(|player| !player.is_bot && riot_id_matches(&player.summoner_name, &local_name))
        .ok_or_else(|| format!("未能在 LiveClient 名单里定位本地玩家 {local_name}"))
}

pub async fn get_active_player_champion() -> Result<(String, String), String> {
    let raw_body = liveclient_get("/liveclientdata/activeplayer").await?;
    let value: serde_json::Value =
        serde_json::from_str(&raw_body).map_err(|error| format!("解析 LiveClient 当前玩家失败: {error}"))?;
    let champion_name = value
        .get("championName")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let raw_champion_name = value
        .get("rawChampionName")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if champion_name.is_empty() && raw_champion_name.is_empty() {
        return Err("LiveClient 当前玩家没有英雄信息".to_string());
    }
    Ok((champion_name, raw_champion_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player_json(identity_fields: serde_json::Value) -> String {
        serde_json::json!([{
            "summonerName": identity_fields["summonerName"],
            "riotId": identity_fields["riotId"],
            "riotIdGameName": identity_fields["riotIdGameName"],
            "riotIdTagLine": identity_fields["riotIdTagLine"],
            "championName": "探险家",
            "isBot": identity_fields["isBot"],
            "isDead": false,
            "items": [],
            "level": 1,
            "position": "MIDDLE",
            "rawChampionName": "game_character_displayname_Ezreal",
            "respawnTimer": 0.0,
            "runes": {},
            "scores": {},
            "skinID": 0,
            "summonerSpells": {},
            "team": "ORDER"
        }])
        .to_string()
    }

    #[test]
    fn parse_json_string_strips_quotes() {
        assert_eq!(parse_json_string("\"bbs#23912\""), "bbs#23912");
        assert_eq!(parse_json_string("bbs#23912"), "bbs#23912");
    }

    #[test]
    fn riot_id_matches_ignores_case_and_optional_tag() {
        assert!(riot_id_matches("bbs#23912", "BBS#23912"));
        assert!(riot_id_matches("bbs", "bbs#23912"));
        assert!(!riot_id_matches("bbs#23912", "bbs#other"));
    }

    #[test]
    fn player_list_prefers_structured_riot_id_over_champion_shim() {
        let raw = player_json(serde_json::json!({
            "summonerName": "探险家",
            "riotId": "Real Player#CN1",
            "riotIdGameName": "Real Player",
            "riotIdTagLine": "CN1",
            "isBot": false
        }));

        let players = parse_live_player_list(&raw).expect("valid LiveClient fixture");

        assert_eq!(players[0].summoner_name, "Real Player#CN1");
    }

    #[test]
    fn player_list_preserves_localized_bot_name() {
        let raw = player_json(serde_json::json!({
            "summonerName": "探险家（电脑）",
            "riotId": "Ezreal#BOT",
            "riotIdGameName": "Ezreal",
            "riotIdTagLine": "BOT",
            "isBot": true
        }));

        let players = parse_live_player_list(&raw).expect("valid LiveClient fixture");

        assert_eq!(players[0].summoner_name, "探险家（电脑）");
    }

    #[test]
    fn player_list_keeps_summoner_name_when_riot_id_is_absent() {
        let raw = player_json(serde_json::json!({
            "summonerName": "Legacy Name",
            "riotId": null,
            "riotIdGameName": null,
            "riotIdTagLine": null,
            "isBot": false
        }));

        let players = parse_live_player_list(&raw).expect("valid LiveClient fixture");

        assert_eq!(players[0].summoner_name, "Legacy Name");
    }
}
