use super::service;
use crate::http_client;
use crate::shared::types::{AdvicePerspective, GameAdvice, GameDetail, PlayerMatchStats};

#[tauri::command]
pub async fn get_match_history(count: Option<u32>, queue_id: Option<i32>) -> Result<PlayerMatchStats, String> {
    println!("🔢 ===== get_match_history 命令被调用 =====");
    println!("📥 接收到的参数:");
    println!("   - count: {:?}", count);
    println!("   - queue_id: {:?}", queue_id);

    let client = http_client::get_lcu_client();
    let end_count: usize = count.unwrap_or(20) as usize;

    if queue_id.is_none() {
        println!("⚠️ 警告: queue_id 是 None，将不进行队列过滤");
    } else {
        println!("✅ queue_id 有值: {}", queue_id.unwrap());
    }

    service::get_match_history(client, end_count, queue_id).await
}

/// 获取指定玩家的战术建议（用于针对敌人或协作队友）
///
/// 参数：
/// - summoner_name: 召唤师名称
/// - perspective: 建议视角 ("Targeting" 针对敌人 / "Collaboration" 协作队友)
/// - target_role: 目标玩家的位置（可选）
#[tauri::command]
pub async fn get_player_tactical_advice(
    summoner_name: String,
    perspective: String,
    target_role: Option<String>,
) -> Result<Vec<GameAdvice>, String> {
    println!("🎯 ===== 获取玩家战术建议 =====");
    println!("📥 召唤师: {}", summoner_name);
    println!("📥 视角: {}", perspective);
    println!("📥 位置: {:?}", target_role);

    let client = http_client::get_lcu_client();

    // 解析视角
    let advice_perspective = match perspective.as_str() {
        "Targeting" => AdvicePerspective::Targeting,
        "Collaboration" => AdvicePerspective::Collaboration,
        _ => return Err("无效的建议视角，只支持 Targeting 或 Collaboration".to_string()),
    };

    service::get_player_tactical_advice(&client, summoner_name, advice_perspective, target_role).await
}

#[tauri::command]
pub async fn get_game_detail(game_id: u64) -> Result<GameDetail, String> {
    log::info!("🔍 ===== 获取游戏详细信息 =====");
    log::info!("🎮 游戏ID: {}", game_id);
    let client = http_client::get_lcu_client();
    service::get_game_detail_logic(&client, game_id).await
}
