use super::{analysis_service, service};
use crate::domains::analysis::pipeline::{MatchAnalysisRequest, MatchAnalysisResult};
use crate::domains::analysis::MatchEvidence;
use crate::http_client;
use crate::shared::types::{GameDetail, GameProcessReview};

/// Runs the complete match-analysis pipeline for an explicit player identity.
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

    analysis_service::analyze_matches_for_puuid(http_client::get_lcu_client(), &puuid, &request).await
}

#[tauri::command]
pub async fn get_game_detail(game_id: u64) -> Result<GameDetail, String> {
    log::info!("[match-detail] loading gameId={game_id}");
    service::get_game_detail_logic(http_client::get_lcu_client(), game_id).await
}

/// Builds a single-game process review, reusing compatible evidence when supplied.
#[tauri::command]
pub async fn get_game_process_review(
    puuid: String,
    game_id: u64,
    cached_evidence: Option<MatchEvidence>,
) -> Result<GameProcessReview, String> {
    log::info!(
        "[process-review] gameId={} cached={}",
        game_id,
        cached_evidence.is_some()
    );
    service::get_game_process_review_logic(http_client::get_lcu_client(), &puuid, game_id, cached_evidence).await
}
