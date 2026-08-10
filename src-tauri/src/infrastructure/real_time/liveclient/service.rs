use crate::infrastructure::game_session::auth::service::ensure_valid_auth_info_async;
use crate::shared::types::{PlayerAnalysisData, TeamAnalysisData};
use serde_json::Value;

/// 复用缓存，仅更新敌方队伍
pub async fn update_enemy_team_in_cache(cache: &mut TeamAnalysisData, local_summoner_name: &str) -> Result<(), String> {
    let live_team = get_live_team_analysis(local_summoner_name).await?;
    // 只更新 enemy_team 字段，其余字段保持原缓存
    cache.enemy_team = live_team.enemy_team;
    cache.local_player_cell_id = live_team.local_player_cell_id;
    cache.game_phase = "InProgress".to_string();
    // 其他字段如 actions、bans、timer、queue_id、is_custom_game 保持原缓存
    Ok(())
}

/// 动态检测 LiveClient 端口
/// 通常 LiveClient 使用固定端口 2999，但某些情况下可能会变化
async fn detect_liveclient_port() -> Option<u16> {
    // 首先尝试默认端口 2999
    if test_liveclient_port(2999).await {
        return Some(2999);
    }

    // 如果 2999 不可用，尝试其他可能的端口
    let possible_ports = [2998, 3000, 3001, 2997];
    for port in possible_ports {
        if test_liveclient_port(port).await {
            log::info!("[LiveClient] 检测到端口: {}", port);
            return Some(port);
        }
    }

    None
}

/// 测试指定端口是否可用
async fn test_liveclient_port(port: u16) -> bool {
    let client = match reqwest::Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(2))
        .build()
    {
        Ok(client) => client,
        Err(_) => return false,
    };

    let url = format!("https://127.0.0.1:{}/liveclientdata/playerlist", port);
    match client.get(&url).send().await {
        Ok(resp) => resp.status().is_success(),
        Err(_) => false,
    }
}

/// 获取游戏内玩家列表
pub async fn get_live_player_list() -> Result<Vec<crate::shared::types::LiveClientPlayer>, String> {
    // 确保 LCU 认证信息可用（虽然 LiveClient 不需要认证，但确保游戏正在运行）
    let _auth = ensure_valid_auth_info_async()
        .await
        .ok_or("无法获取 LCU 认证信息，请确保游戏正在运行")?;
    // LiveClient 通常使用固定端口 2999，但我们可以尝试动态检测
    let liveclient_port = detect_liveclient_port().await.unwrap_or(2999);

    let client = reqwest::Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://127.0.0.1:{}/liveclientdata/playerlist", liveclient_port);
    let resp = client.get(&url).send().await.map_err(|e| {
        if e.to_string().contains("tcp connect error") {
            format!(
                "LiveClient 服务不可用 (端口 {}), 请确保游戏已启动并进入对局",
                liveclient_port
            )
        } else {
            format!("请求失败: {}", e)
        }
    })?;

    if !resp.status().is_success() {
        return Err(format!("LiveClient API 返回错误状态: {}", resp.status()));
    }

    let text = resp.text().await.map_err(|e| e.to_string())?;

    // 尝试解析为 JSON 值来查看实际结构
    let json_value: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("解析 JSON 失败: {}", e))?;

    log::info!(
        "[LiveClient] 解析后的 JSON: {}",
        serde_json::to_string_pretty(&json_value).unwrap_or_default()
    );

    // 尝试解析为我们的结构
    let players: Vec<crate::shared::types::LiveClientPlayer> =
        serde_json::from_str(&text).map_err(|e| format!("解析为 LiveClientPlayer 失败: {}", e))?;

    Ok(players)
}

/// 获取游戏内事件数据
pub async fn get_live_events() -> Result<Vec<Value>, String> {
    let liveclient_port = detect_liveclient_port().await.unwrap_or(2999);

    let client = reqwest::Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://127.0.0.1:{}/liveclientdata/eventdata", liveclient_port);
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    let text = resp.text().await.map_err(|e| e.to_string())?;
    let events: Vec<Value> = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    Ok(events)
}

/// 获取游戏状态
pub async fn get_game_stats() -> Result<Value, String> {
    let liveclient_port = detect_liveclient_port().await.unwrap_or(2999);

    let client = reqwest::Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://127.0.0.1:{}/liveclientdata/gamestats", liveclient_port);
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    let text = resp.text().await.map_err(|e| e.to_string())?;
    let stats: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    Ok(stats)
}

/// 检查 LiveClient 是否可用
pub async fn is_liveclient_available() -> bool {
    detect_liveclient_port().await.is_some()
}

/// 实时获取 TeamAnalysisData 结构，保证与分析数据一致
pub async fn get_live_team_analysis(local_summoner_name: &str) -> Result<TeamAnalysisData, String> {
    let players = get_live_player_list().await?;
    // 分组 my_team/enemy_team
    let mut my_team = Vec::new();
    let mut enemy_team = Vec::new();
    let mut local_player_cell_id = 0;
    let mut found_local = false;
    let mut my_team_name = String::new();

    // 先确定本地玩家所在队伍
    for p in &players {
        if p.summoner_name == local_summoner_name {
            my_team_name = p.team.clone();
            found_local = true;
            break;
        }
    }
    if !found_local {
        return Err("未找到本地玩家，请确认召唤师名是否正确".to_string());
    }

    for (idx, p) in players.iter().enumerate() {
        let mut pa = PlayerAnalysisData {
            cell_id: idx as i32,
            display_name: p.summoner_name.clone(),
            summoner_id: None,
            puuid: None,
            is_local: p.summoner_name == local_summoner_name,
            is_bot: p.is_bot,
            champion_id: None,
            champion_name: Some(p.champion_name.clone()),
            champion_pick_intent: None,
            position: Some(p.position.clone()),
            tier: None,
            profile_icon_id: None,
            tag_line: None,
            spell1_id: None,
            spell2_id: None,
            match_stats: None,
        };
        if pa.is_local {
            local_player_cell_id = pa.cell_id;
        }
        if p.team == my_team_name {
            my_team.push(pa);
        } else {
            enemy_team.push(pa);
        }
    }

    Ok(TeamAnalysisData {
        my_team,
        enemy_team,
        local_player_cell_id,
        game_phase: "InProgress".to_string(),
        queue_id: 0,
        is_custom_game: false,
        actions: None,
        bans: None,
        timer: None,
    })
}
