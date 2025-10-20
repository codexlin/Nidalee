use crate::http_client;
use crate::infrastructure::real_time::liveclient;

#[tauri::command]
pub async fn get_live_player_list() -> Result<String, String> {
    let players = liveclient::get_live_player_list().await?;
    let json = serde_json::to_string(&players).map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
pub async fn get_live_events() -> Result<String, String> {
    let events = liveclient::get_live_events().await?;
    let json = serde_json::to_string(&events).map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
pub async fn get_game_stats() -> Result<String, String> {
    let stats = liveclient::get_game_stats().await?;
    let json = serde_json::to_string(&stats).map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
pub async fn is_liveclient_available() -> bool {
    liveclient::is_liveclient_available().await
}

#[tauri::command]
pub async fn get_game_version() -> Result<String, String> {
    // 尝试从公开的Riot API获取最新版本
    let client = http_client::get_public_client();

    match client
        .get("https://ddragon.leagueoflegends.com/api/versions.json")
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(versions) = response.json::<Vec<String>>().await {
                if let Some(latest_version) = versions.first() {
                    return Ok(latest_version.clone());
                }
            }
        }
        Err(_) => {
            // 如果获取失败，返回一个相对较新的默认版本
            return Ok("14.23.1".to_string());
        }
    }

    // 备用默认版本
    Ok("14.23.1".to_string())
}
