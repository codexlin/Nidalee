// Tauri 命令模块 - 集中管理所有的 Tauri 命令
use reqwest::Client;
use serde_json::Value;

// ===== 英雄出装和符文相关命令 =====
// 注：旧的 ddragon API 相关函数已废弃，全部使用新的 OP.GG API

/// 获取英雄详细数据 - 使用新的 OP.GG API
#[tauri::command]
pub async fn get_champion_build_new(
    region: String,
    mode: String,
    champion_id: i32,
    position: Option<String>,
    tier: String,
) -> Result<Value, String> {
    log::info!(
        "🚀 获取英雄详细数据: 区域={}, 模式={}, 英雄ID={}, 位置={:?}, 段位={}",
        region,
        mode,
        champion_id,
        position,
        tier
    );

    let client = Client::new();

    // 详细 API URL
    let url = if mode == "arena" {
        format!(
            "https://lol-api-champion.op.gg/api/{}/champions/{}/{}?tier={}",
            region, mode, champion_id, tier
        )
    } else {
        let pos = position.unwrap_or("MID".to_string());
        format!(
            "https://lol-api-champion.op.gg/api/{}/champions/{}/{}/{}?tier={}",
            region, mode, champion_id, pos, tier
        )
    };

    log::info!("🌐 请求URL: {}", url);

    // 发送请求
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 请求失败: HTTP {}", response.status()));
    }

    let data: Value = response.json().await.map_err(|e| format!("解析 JSON 失败: {}", e))?;

    log::info!("✅ 成功获取英雄详细数据");
    Ok(data)
}

/// 获取所有英雄列表
#[tauri::command]
pub async fn get_champions_list(region: String, mode: String, tier: String) -> Result<Value, String> {
    log::info!("🚀 获取英雄列表: 区域={}, 模式={}, 段位={}", region, mode, tier);

    let client = Client::new();

    let url = format!(
        "https://lol-api-champion.op.gg/api/{}/champions/{}?tier={}",
        region, mode, tier
    );

    log::info!("🌐 请求URL: {}", url);

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API 请求失败: HTTP {}", response.status()));
    }

    let data: Value = response.json().await.map_err(|e| format!("解析 JSON 失败: {}", e))?;

    log::info!("✅ 成功获取英雄列表数据");
    Ok(data)
}
