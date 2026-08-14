use std::collections::HashMap;

use futures_util::stream::{self, StreamExt};
use thiserror::Error;

use crate::domains::analysis::queue_config::{AnalysisProfile, QueueType};
use crate::infrastructure::data_services::summoner::service::get_summoners_by_names;
use crate::infrastructure::match_management::matches::fetcher::fetch_match_list;
use crate::shared::types::{PlayerAnalysisData, PlayerAnalysisResult, PlayerAnalysisStatus};

const HISTORY_FETCH_CONCURRENCY: usize = 2;
const RANKED_HISTORY_FETCH_COUNT: usize = 50;
const SIMPLE_HISTORY_FETCH_COUNT: usize = 20;
const REALTIME_HISTORY_FETCH_ATTEMPTS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MatchStatsCacheKey {
    puuid: String,
    queue_id: i64,
    is_custom_game: bool,
}

impl MatchStatsCacheKey {
    pub(crate) fn new(puuid: impl Into<String>, queue_id: i64, is_custom_game: bool) -> Self {
        Self {
            puuid: puuid.into(),
            queue_id,
            is_custom_game,
        }
    }
}

pub(crate) type CachedPlayerAnalysis = PlayerAnalysisResult;
pub(crate) type MatchStatsCache = HashMap<MatchStatsCacheKey, CachedPlayerAnalysis>;

#[derive(Debug, Error)]
pub(crate) enum RealtimePlayerAnalysisError {
    #[error("战绩服务暂不可用: {0}")]
    Unavailable(String),
    #[error("近期战绩中没有可用于实时分析的完整对局")]
    InsufficientData,
}

impl RealtimePlayerAnalysisError {
    pub(crate) fn status(&self) -> PlayerAnalysisStatus {
        match self {
            Self::Unavailable(_) => PlayerAnalysisStatus::Unavailable,
            Self::InsufficientData => PlayerAnalysisStatus::InsufficientData,
        }
    }

    pub(crate) fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable(_))
    }
}

pub(crate) fn apply_player_analysis(
    player: &mut PlayerAnalysisData,
    mut analysis: PlayerAnalysisResult,
    queue_id: i64,
) {
    if let Some(ranked) = analysis.ranked.as_mut() {
        ranked.rating = crate::domains::analysis::realtime::build_ranked_player_rating(
            queue_id,
            player.solo_rank.as_ref(),
            player.flex_rank.as_ref(),
            &analysis.stats,
            &analysis.basis,
        );
    }
    player.analysis = Some(analysis);
    reproject_player_analysis(player);
    player.analysis_status = PlayerAnalysisStatus::Ready;
}

pub(crate) fn clear_player_analysis(player: &mut PlayerAnalysisData) {
    player.analysis = None;
}

fn mark_player_unavailable(player: &mut PlayerAnalysisData) {
    clear_player_analysis(player);
    player.analysis_status = PlayerAnalysisStatus::Unavailable;
}

pub(crate) fn reproject_player_analysis(player: &mut PlayerAnalysisData) {
    let Some(ranked) = player.analysis.as_mut().and_then(|analysis| analysis.ranked.as_mut()) else {
        return;
    };
    let champion_name = |champion_id| {
        crate::infrastructure::data_services::champion_data::service::get_champion_info(champion_id)
            .map(|champion| champion.name)
    };
    crate::domains::analysis::realtime::project_ranked_context(
        ranked,
        player.position.as_deref(),
        player.champion_id,
        Some(&champion_name),
    );
}

pub(super) async fn fetch_players_match_stats(
    mut players: Vec<&mut PlayerAnalysisData>,
    http_client: &reqwest::Client,
    queue_id: i64,
    is_custom_game: bool,
    match_stats_cache: &mut MatchStatsCache,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let mut cached_count = 0;
    let mut need_fetch_indices = Vec::new();

    for (index, player) in players.iter_mut().enumerate() {
        if player.display_name.is_empty() || player.display_name == "未知召唤师" {
            continue;
        }
        let cached = player
            .puuid
            .as_deref()
            .and_then(|puuid| match_stats_cache.get(&MatchStatsCacheKey::new(puuid, queue_id, is_custom_game)));
        if let Some(analysis) = cached {
            apply_player_analysis(player, analysis.clone(), queue_id);
            cached_count += 1;
        } else {
            need_fetch_indices.push(index);
        }
    }

    log::info!(
        target: "analysis::team",
        "Player analysis cache: {}/{} hit, {} need fetch",
        cached_count,
        players.len(),
        need_fetch_indices.len()
    );

    if need_fetch_indices.is_empty() {
        return Ok(cached_count);
    }

    let player_names = need_fetch_indices
        .iter()
        .filter(|&&index| players[index].puuid.as_deref().is_none_or(str::is_empty))
        .map(|&index| players[index].display_name.clone())
        .collect::<Vec<_>>();
    let summoners = if player_names.is_empty() {
        Vec::new()
    } else {
        get_summoners_by_names(http_client, player_names)
            .await
            .inspect_err(|error| {
                log::warn!(
                    target: "analysis::team",
                    "Batch summoner query failed; known PUUIDs can still continue: {error}"
                )
            })
            .unwrap_or_default()
    };

    let mut requests = Vec::new();
    for index in need_fetch_indices {
        let player = &mut players[index];
        let puuid = player.puuid.clone().filter(|value| !value.is_empty()).or_else(|| {
            summoners
                .iter()
                .find(|summoner| {
                    let full_name = match (&summoner.game_name, &summoner.tag_line) {
                        (Some(game_name), Some(tag_line)) => format!("{game_name}#{tag_line}"),
                        _ => summoner.display_name.clone(),
                    };
                    full_name.eq_ignore_ascii_case(&player.display_name)
                })
                .map(|summoner| summoner.puuid.clone())
        });
        let Some(puuid) = puuid else {
            log::warn!(target: "analysis::team", "Summoner info not found for '{}'", player.display_name);
            mark_player_unavailable(player);
            continue;
        };
        requests.push((index, puuid, player.display_name.clone()));
    }

    let results = stream::iter(requests)
        .map(|(index, puuid, display_name)| async move {
            let result = fetch_realtime_player_analysis_with_retry(http_client, &puuid, queue_id, is_custom_game).await;
            (index, puuid, display_name, result)
        })
        .buffer_unordered(HISTORY_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    for (index, puuid, display_name, result) in results {
        let player = &mut players[index];
        match result {
            Ok(analysis) => {
                player.puuid = Some(puuid.clone());
                match_stats_cache.insert(
                    MatchStatsCacheKey::new(puuid, queue_id, is_custom_game),
                    analysis.clone(),
                );
                apply_player_analysis(player, analysis, queue_id);
            }
            Err(error) => {
                log::warn!(
                    target: "analysis::team",
                    "Failed to analyze player '{}': {}",
                    display_name,
                    error
                );
                clear_player_analysis(player);
                player.analysis_status = error.status();
            }
        }
    }

    Ok(players.iter().filter(|player| player.analysis.is_some()).count())
}

async fn fetch_realtime_player_analysis(
    http_client: &reqwest::Client,
    puuid: &str,
    queue_id: i64,
    is_custom_game: bool,
) -> Result<CachedPlayerAnalysis, RealtimePlayerAnalysisError> {
    let fetch_count = history_fetch_count(queue_id, is_custom_game);
    let response = fetch_match_list(http_client, puuid, fetch_count)
        .await
        .map_err(RealtimePlayerAnalysisError::Unavailable)?;
    let games = response
        .get("games")
        .and_then(|games| games.get("games"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| RealtimePlayerAnalysisError::Unavailable("LCU 战绩响应缺少 games.games".to_owned()))?;
    let context = crate::domains::analysis::realtime::RealtimeMatchContext::from_lcu(queue_id, is_custom_game);
    let champion_name = |champion_id| {
        crate::infrastructure::data_services::champion_data::service::get_champion_info(champion_id)
            .map(|champion| champion.name)
    };

    crate::domains::analysis::realtime::analyze_realtime_history(
        games,
        puuid,
        context,
        None,
        None,
        Some(&champion_name),
    )
    .ok_or(RealtimePlayerAnalysisError::InsufficientData)
}

fn history_fetch_count(queue_id: i64, is_custom_game: bool) -> usize {
    let profile = if is_custom_game {
        AnalysisProfile::Simple
    } else {
        QueueType::from_queue_id(queue_id as i32).analysis_profile()
    };
    match profile {
        AnalysisProfile::Ranked(_) => RANKED_HISTORY_FETCH_COUNT,
        AnalysisProfile::Simple => SIMPLE_HISTORY_FETCH_COUNT,
    }
}

pub(crate) async fn fetch_realtime_player_analysis_with_retry(
    http_client: &reqwest::Client,
    puuid: &str,
    queue_id: i64,
    is_custom_game: bool,
) -> Result<CachedPlayerAnalysis, RealtimePlayerAnalysisError> {
    for attempt in 1..=REALTIME_HISTORY_FETCH_ATTEMPTS {
        let result = fetch_realtime_player_analysis(http_client, puuid, queue_id, is_custom_game).await;
        match result {
            Err(error) if error.is_unavailable() && attempt < REALTIME_HISTORY_FETCH_ATTEMPTS => {
                log::debug!(
                    target: "analysis::team",
                    "Realtime history request failed on attempt {attempt}; retrying once: {error}"
                );
                tokio::time::sleep(std::time::Duration::from_millis(750)).await;
            }
            result => return result,
        }
    }

    unreachable!("history retry loop always returns on its final attempt")
}

#[cfg(test)]
mod tests {
    use super::{history_fetch_count, mark_player_unavailable};
    use crate::shared::types::{PlayerAnalysisData, PlayerAnalysisStatus};

    fn loading_player() -> PlayerAnalysisData {
        PlayerAnalysisData {
            cell_id: 1,
            display_name: "Player#CN1".to_owned(),
            summoner_id: None,
            puuid: None,
            is_local: false,
            is_bot: false,
            analysis_status: PlayerAnalysisStatus::Loading,
            champion_id: None,
            champion_name: None,
            champion_pick_intent: None,
            position: None,
            solo_rank: None,
            flex_rank: None,
            profile_icon_id: None,
            tag_line: None,
            spell1_id: None,
            spell2_id: None,
            analysis: None,
        }
    }

    #[test]
    fn unresolved_identity_does_not_remain_loading() {
        let mut player = loading_player();

        mark_player_unavailable(&mut player);

        assert_eq!(player.analysis_status, PlayerAnalysisStatus::Unavailable);
    }

    #[test]
    fn ranked_fetches_fifty_candidates_but_simple_fetches_twenty() {
        assert_eq!(history_fetch_count(420, false), 50);
        assert_eq!(history_fetch_count(440, false), 50);
        assert_eq!(history_fetch_count(450, false), 20);
        assert_eq!(history_fetch_count(420, true), 20);
    }
}
