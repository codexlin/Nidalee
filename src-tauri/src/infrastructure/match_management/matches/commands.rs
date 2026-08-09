use super::analysis_service;
use super::service;
use crate::domains::analysis::pipeline::{MatchAnalysisRequest, MatchAnalysisResult};
use crate::http_client;
use crate::shared::types::{AdvicePerspective, GameAdvice, GameDetail};

/// 唯一的对局分析命令
///
/// 前端只需要这一个入口：一次调用同时拿到展示用战绩列表、基础统计、位置拆分、
/// 确定性证据与特征/建议，以及「为什么没有某个结论」的降级诊断。
///
/// - `puuid`：分析目标。分析当前召唤师时也必须显式传入，避免命令内部隐式换人。
/// - `request`：统一分析请求（camelCase 载荷，见 `MatchAnalysisRequest`）。
#[tauri::command]
pub async fn analyze_matches(puuid: String, request: MatchAnalysisRequest) -> Result<MatchAnalysisResult, String> {
    log::info!(
        "[analyze_matches] puuid={} count={} mode={:?} depth={:?} maxAnalysisGames={:?}",
        puuid,
        request.count,
        request.mode,
        request.depth,
        request.max_analysis_games
    );

    let client = http_client::get_lcu_client();
    analysis_service::analyze_matches_for_puuid(client, &puuid, &request).await
}

/// 获取指定玩家的战术建议（用于针对敌人或协作队友）
///
/// 参数：
/// - summoner_name: 召唤师名称
/// - perspective: 建议视角 ("Targeting" 针对敌人 / "Collaboration" 协作队友)
/// - target_role: 目标玩家的位置（可选，仅用于补齐建议的位置标注）
#[tauri::command]
pub async fn get_player_tactical_advice(
    summoner_name: String,
    perspective: String,
    target_role: Option<String>,
) -> Result<Vec<GameAdvice>, String> {
    let advice_perspective = match perspective.as_str() {
        "Targeting" => AdvicePerspective::Targeting,
        "Collaboration" => AdvicePerspective::Collaboration,
        _ => return Err("无效的建议视角，只支持 Targeting 或 Collaboration".to_string()),
    };

    let client = http_client::get_lcu_client();
    service::get_player_tactical_advice(&client, summoner_name, advice_perspective, target_role).await
}

#[tauri::command]
pub async fn get_game_detail(game_id: u64) -> Result<GameDetail, String> {
    log::info!("🔍 获取游戏详细信息: gameId={}", game_id);
    let client = http_client::get_lcu_client();
    service::get_game_detail_logic(&client, game_id).await
}
