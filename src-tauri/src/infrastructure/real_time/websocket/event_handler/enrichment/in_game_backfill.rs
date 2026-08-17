use std::collections::HashSet;

use futures_util::stream::{self, StreamExt};
use tauri::Emitter;

use super::super::WsEventHandler;
use super::identity::{
    apply_confirmed_bot_identity, apply_live_player_identity, canonical_riot_id, resolve_local_live_team, same_riot_id,
    select_team_slot,
};
use crate::infrastructure::data_services::{champion_data, static_catalog};
use crate::shared::types::{PlayerAnalysisData, PlayerAnalysisStatus, TeamAnalysisData};
use crate::shared::Result;

const HISTORY_FETCH_CONCURRENCY: usize = 2;

#[derive(Clone, Copy)]
enum TeamSide {
    Ally,
    Enemy,
}

impl TeamSide {
    fn players(self, analysis: &TeamAnalysisData) -> &[PlayerAnalysisData] {
        match self {
            Self::Ally => &analysis.my_team,
            Self::Enemy => &analysis.enemy_team,
        }
    }

    fn players_mut(self, analysis: &mut TeamAnalysisData) -> &mut [PlayerAnalysisData] {
        match self {
            Self::Ally => &mut analysis.my_team,
            Self::Enemy => &mut analysis.enemy_team,
        }
    }
}

struct LiveRosterTarget {
    side: TeamSide,
    index: usize,
    champion_id: i32,
    live_player: crate::shared::types::LiveClientPlayer,
}

struct HistoryTarget {
    side: TeamSide,
    index: usize,
    puuid: String,
    display_name: String,
}

fn needs_ally_identity_backfill(player: &PlayerAnalysisData) -> bool {
    player.analysis_status == PlayerAnalysisStatus::AwaitingIdentity
        || player.puuid.as_deref().is_none_or(str::is_empty)
        || player.display_name.trim().is_empty()
}

fn refresh_failure_status(
    has_cached_analysis: bool,
    failure_status: crate::shared::types::PlayerAnalysisStatus,
) -> crate::shared::types::PlayerAnalysisStatus {
    if has_cached_analysis {
        crate::shared::types::PlayerAnalysisStatus::Ready
    } else {
        failure_status
    }
}

impl WsEventHandler {
    /// Reconciles cached champion-select slots with the authoritative in-game roster.
    pub(in crate::infrastructure::real_time::websocket::event_handler) async fn backfill_team_data(
        &self,
        generation: u64,
    ) -> Result<()> {
        log::info!("开始补全对局双方阵容数据");

        if !self.cache.read().await.can_commit_in_game_recovery(generation) {
            return Ok(());
        }

        static_catalog::ensure_static_catalogs()
            .await
            .map_err(|error| format!("Static catalog is unavailable for in-game backfill: {error}"))?;

        if !self.cache.read().await.can_commit_in_game_recovery(generation) {
            return Ok(());
        }

        // 1. Fetch the full player list from the LiveClient API.
        // Retry logic is necessary as the LiveClient API may not be ready immediately at game start.
        let live_players = {
            let mut attempts = 0;
            let max_attempts = 30; // Increased to 30 attempts, total wait time ~1 minute.
            loop {
                attempts += 1;
                match crate::infrastructure::real_time::liveclient::service::get_live_player_list().await {
                    Ok(players) if !players.is_empty() => {
                        log::info!("LiveClient 玩家列表已就绪");
                        break players;
                    }
                    Ok(_) => {
                        if attempts >= max_attempts {
                            return Err(
                                "Failed to get player list from LiveClient after multiple attempts: returned an empty list (game loading?)".into(),
                            );
                        }
                        log::warn!(
                            "LiveClient returned an empty list (game loading?), attempt {}/{}...",
                            attempts,
                            max_attempts
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                    Err(e) => {
                        if attempts >= max_attempts {
                            return Err(format!(
                                "Failed to get player list from LiveClient after {} attempts: {}",
                                max_attempts, e
                            )
                            .into());
                        }
                        log::warn!(
                            "Failed to fetch LiveClient player list (attempt {}/{}), retrying in 2s: {}",
                            attempts,
                            max_attempts,
                            e
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
        };

        log::info!("LiveClient 返回 {} 名玩家", live_players.len());

        // 2. Snapshot cached analysis data. All network I/O below operates on this owned copy;
        // no EventCache lock is held across an await.
        let mut team_analysis = {
            let cache = self.cache.read().await;
            if !cache.can_commit_in_game_recovery(generation) {
                log::debug!("Recovery generation {generation} is stale before backfill");
                return Ok(());
            }
            match cache.team_analysis_data.clone() {
                Some(data) => data,
                None => {
                    log::warn!("No TeamAnalysisData in cache, cannot perform backfill.");
                    return Ok(());
                }
            }
        };

        // 3. Resolve the local side from the local player instead of assuming ORDER.
        // Custom games can place the local player on either side.
        let Some(local_team) = resolve_local_live_team(&live_players, &team_analysis).map(str::to_owned) else {
            log::warn!("Could not resolve the local player's LiveClient team; skipping roster backfill.");
            return Ok(());
        };

        let (ally_live_players, enemy_live_players): (Vec<_>, Vec<_>) =
            live_players.into_iter().partition(|player| player.team == local_team);

        // LiveClient is authoritative once the game starts. Enemy players are always refreshed
        // because their identities were hidden during champion select. Allies are refreshed only
        // when champion select left an unresolved identity, preserving completed ally analysis.
        let mut roster_targets = Vec::new();
        for (side, live_team) in [
            (TeamSide::Ally, ally_live_players),
            (TeamSide::Enemy, enemy_live_players),
        ] {
            let mut occupied_slots = HashSet::new();
            for live_player in live_team {
                let Some(champion_id) = champion_data::get_champion_id_by_name(&live_player.champion_name) else {
                    log::warn!("Could not resolve champion '{}'.", live_player.champion_name);
                    continue;
                };

                let Some(index) =
                    select_team_slot(side.players(&team_analysis), &occupied_slots, &live_player, champion_id)
                else {
                    log::warn!(
                        "Could not match player '{}' (champion: {}) to the cached roster.",
                        live_player.summoner_name,
                        live_player.champion_name
                    );
                    continue;
                };
                occupied_slots.insert(index);

                if live_player.is_bot {
                    apply_confirmed_bot_identity(
                        &mut side.players_mut(&mut team_analysis)[index],
                        &live_player,
                        champion_id,
                    );
                    continue;
                }
                if live_player.summoner_name.trim().is_empty() {
                    continue;
                }
                if matches!(side, TeamSide::Ally) && !needs_ally_identity_backfill(&side.players(&team_analysis)[index])
                {
                    continue;
                }

                roster_targets.push(LiveRosterTarget {
                    side,
                    index,
                    champion_id,
                    live_player,
                });
            }
        }

        if roster_targets.is_empty() {
            let mut cache = self.cache.write().await;
            if !cache.can_commit_in_game_recovery(generation) {
                return Ok(());
            }
            cache.team_analysis_data = Some(team_analysis.clone());
            cache.in_game_recovery_abort = None;
            let _ = self.app.emit("team-analysis-data", &team_analysis);
            log::info!("In-game roster required no player identity enrichment.");
            return Ok(());
        }

        log::info!("识别到 {} 名待补全玩家，开始获取身份、段位与战绩", roster_targets.len());

        // 4. Batch fetch summoner info.
        let player_names: Vec<String> = roster_targets
            .iter()
            .map(|target| target.live_player.summoner_name.clone())
            .collect();

        let summoners_info =
            match crate::infrastructure::data_services::summoner::service::get_summoners_by_names_with_rank(
                &self.client,
                player_names.clone(),
            )
            .await
            {
                Ok(info) => {
                    log::info!("已获取 {} 名玩家的身份与段位", info.len());
                    info
                }
                Err(e) => {
                    log::error!(
                        "Batch fetch for summoner info failed: {}. Publishing the resolved LiveClient roster without rank/history.",
                        e
                    );
                    Vec::new()
                }
            };

        // 5. Resolve every missing identity first. The frontend can then show names/ranks while
        // history requests continue in parallel instead of waiting for the entire team.
        let mut history_targets = Vec::new();

        for target in roster_targets {
            let live_player = target.live_player;
            let player = &mut target.side.players_mut(&mut team_analysis)[target.index];
            apply_live_player_identity(player, &live_player, target.champion_id);

            // 5.1 Find the corresponding summoner info.
            let summoner_info = summoners_info.iter().find(|s| {
                let full_name = if let (Some(game_name), Some(tag_line)) = (&s.game_name, &s.tag_line) {
                    format!("{}#{}", game_name, tag_line)
                } else {
                    s.display_name.clone()
                };
                same_riot_id(&full_name, &live_player.summoner_name)
            });

            if let Some(info) = summoner_info {
                // 5.2 Update rank, icon, etc.
                player.display_name =
                    canonical_riot_id(&info.display_name, info.game_name.as_deref(), info.tag_line.as_deref());
                player.summoner_id = Some(info.summoner_id.clone());
                player.puuid = Some(info.puuid.clone());
                crate::infrastructure::match_management::analysis_data::service::apply_rank_summaries(player, info);
                player.profile_icon_id = Some(info.profile_icon_id as i32);
                player.tag_line = info.tag_line.clone();

                history_targets.push(HistoryTarget {
                    side: target.side,
                    index: target.index,
                    puuid: info.puuid.clone(),
                    display_name: live_player.summoner_name.clone(),
                });
            } else {
                player.analysis_status =
                    refresh_failure_status(player.analysis.is_some(), PlayerAnalysisStatus::Unavailable);
                log::warn!(
                    "Could not find detailed summoner info for '{}'.",
                    live_player.summoner_name
                );
            }
        }

        // Publish the resolved roster before the slower history analysis starts.
        {
            let mut cache = self.cache.write().await;
            if !cache.can_commit_in_game_recovery(generation) {
                return Ok(());
            }
            cache.team_analysis_data = Some(team_analysis.clone());
            let _ = self.app.emit("team-analysis-data", &team_analysis);
        }

        let queue_id = team_analysis.queue_id;
        let is_custom_game = team_analysis.is_custom_game;
        let results = stream::iter(
            history_targets
                .into_iter()
                .map(|target| async move {
                    let result = crate::infrastructure::match_management::analysis_data::service::fetch_realtime_player_analysis_with_retry(
                        &self.client,
                        &target.puuid,
                        queue_id,
                        is_custom_game,
                    )
                    .await;
                    (target, result)
                }),
        )
        .buffer_unordered(HISTORY_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

        let mut stats_to_cache = Vec::new();
        let processed_count = results.len();
        for (target, result) in results {
            match result {
                Ok(analysis) => {
                    stats_to_cache.push((
                        crate::infrastructure::match_management::analysis_data::service::MatchStatsCacheKey::new(
                            &target.puuid,
                            queue_id,
                            is_custom_game,
                        ),
                        analysis.clone(),
                    ));
                    let player = &mut target.side.players_mut(&mut team_analysis)[target.index];
                    crate::infrastructure::match_management::analysis_data::service::apply_player_analysis(
                        player, analysis, queue_id,
                    );
                    player.is_bot = false;
                    log::debug!("玩家分析完成: {}", target.display_name);
                }
                Err(error) => {
                    let player = &mut target.side.players_mut(&mut team_analysis)[target.index];
                    player.analysis_status = refresh_failure_status(player.analysis.is_some(), error.status());
                    log::warn!(
                        "Failed to get match history for player '{}': {}",
                        target.display_name,
                        error
                    );
                }
            }
        }

        let updated_data = team_analysis;

        // Commit only if this is still the active in-game generation.
        let mut cache = self.cache.write().await;
        if !cache.can_commit_in_game_recovery(generation) {
            log::debug!("Discarding stale recovery generation {generation}");
            return Ok(());
        }
        for (key, stats) in stats_to_cache {
            cache.match_stats_cache.insert(key, stats);
        }
        cache.team_analysis_data = Some(updated_data.clone());
        cache.in_game_recovery_abort = None;
        // Emit while the generation guard is still held. A phase transition cannot invalidate
        // this recovery between the final check and publication.
        let _ = self.app.emit("team-analysis-data", &updated_data);
        drop(cache);

        // 6. Emit the updated data to the frontend.
        log::info!("对局双方阵容数据补全完成，共处理 {} 名待补全玩家", processed_count);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{needs_ally_identity_backfill, refresh_failure_status};
    use crate::shared::types::{PlayerAnalysisData, PlayerAnalysisStatus};

    fn ally(status: PlayerAnalysisStatus, puuid: Option<&str>) -> PlayerAnalysisData {
        PlayerAnalysisData {
            cell_id: 1,
            display_name: "ally#CN1".to_owned(),
            summoner_id: None,
            puuid: puuid.map(str::to_owned),
            is_local: false,
            is_bot: false,
            analysis_status: status,
            champion_id: Some(81),
            champion_name: Some("探险家".to_owned()),
            champion_pick_intent: None,
            position: Some("ADC".to_owned()),
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
    fn failed_refresh_keeps_cached_analysis_ready() {
        assert_eq!(
            refresh_failure_status(true, PlayerAnalysisStatus::Unavailable),
            PlayerAnalysisStatus::Ready
        );
        assert_eq!(
            refresh_failure_status(false, PlayerAnalysisStatus::Unavailable),
            PlayerAnalysisStatus::Unavailable
        );
    }

    #[test]
    fn ally_backfill_targets_only_missing_identity() {
        assert!(needs_ally_identity_backfill(&ally(
            PlayerAnalysisStatus::AwaitingIdentity,
            None
        )));
        assert!(!needs_ally_identity_backfill(&ally(
            PlayerAnalysisStatus::Ready,
            Some("known-puuid")
        )));
    }
}
