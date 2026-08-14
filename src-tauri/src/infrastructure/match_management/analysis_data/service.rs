use crate::shared::types::TeamAnalysisData;

use super::{history, roster};

pub(crate) use super::history::{
    apply_player_analysis, clear_player_analysis, fetch_realtime_player_analysis_with_retry, CachedPlayerAnalysis,
    MatchStatsCache, MatchStatsCacheKey,
};
pub(crate) use super::roster::{
    apply_rank_summaries, build_team_roster_from_session, champ_select_analysis_key, patch_team_analysis_from_session,
};

/// 从 ChampSelect 会话构建完整的分析数据
///
/// 核心业务逻辑：
/// 1. 解析选人会话，提取玩家基础信息
/// 2. 批量获取召唤师详细信息（enrichment）
/// 3. 批量获取战绩数据（使用缓存优化，避免重复请求）
/// 4. 生成完整的 PlayerAnalysisData
pub async fn build_team_analysis_from_session(
    session: &serde_json::Value,
    http_client: &reqwest::Client,
    match_stats_cache: &mut MatchStatsCache,
) -> Result<TeamAnalysisData, Box<dyn std::error::Error + Send + Sync>> {
    let local_player_cell_id = session["localPlayerCellId"].as_i64().unwrap_or(0) as i32;
    let queue_id = session["queueId"].as_i64().unwrap_or(0);
    let is_custom_game = session["isCustomGame"].as_bool().unwrap_or(false);

    log::info!(
        target: "analysis::team",
        "Building team analysis data: localPlayerCellId={}, queueId={}, isCustom={}",
        local_player_cell_id,
        queue_id,
        is_custom_game
    );

    if is_custom_game {
        log::debug!(
            target: "analysis::team",
            "Custom game detected, some players may be bots"
        );
    }

    let (my_team_players, enemy_team_players) = tokio::join!(
        roster::parse_and_enrich_team(
            session["myTeam"].as_array(),
            "myTeam",
            local_player_cell_id,
            is_custom_game,
            http_client,
        ),
        roster::parse_and_enrich_team(
            session["theirTeam"].as_array(),
            "theirTeam",
            local_player_cell_id,
            is_custom_game,
            http_client,
        ),
    );
    let mut my_team_players = my_team_players;
    let mut enemy_team_players = enemy_team_players;

    // Fetch both teams through the same objective history analysis pipeline.
    let my_team_real_players: Vec<_> = my_team_players
        .iter_mut()
        .filter(|p| !p.is_bot && !p.display_name.is_empty() && p.display_name != "未知召唤师")
        .collect();

    if !my_team_real_players.is_empty() {
        log::info!(
            target: "analysis::team",
            "Fetching match stats for {} ally players",
            my_team_real_players.len()
        );
        match history::fetch_players_match_stats(
            my_team_real_players,
            http_client,
            queue_id,
            is_custom_game,
            match_stats_cache,
        )
        .await
        {
            Ok(count) => log::info!(target: "analysis::team", "Ally player analysis ready: {count}"),
            Err(error) => log::warn!(target: "analysis::team", "Failed to analyze ally players: {error}"),
        }
    }

    let enemy_real_players: Vec<_> = enemy_team_players
        .iter_mut()
        .filter(|p| !p.is_bot && !p.display_name.is_empty() && p.display_name != "未知召唤师")
        .collect();

    if !enemy_real_players.is_empty() {
        log::info!(
            target: "analysis::team",
            "Fetching match stats for {} enemy players",
            enemy_real_players.len()
        );
        match history::fetch_players_match_stats(
            enemy_real_players,
            http_client,
            queue_id,
            is_custom_game,
            match_stats_cache,
        )
        .await
        {
            Ok(count) => log::info!(target: "analysis::team", "Enemy player analysis ready: {count}"),
            Err(error) => log::warn!(target: "analysis::team", "Failed to analyze enemy players: {error}"),
        }
    }

    Ok(roster::build_team_analysis_snapshot(
        session,
        my_team_players,
        enemy_team_players,
    ))
}
