use std::collections::HashSet;

use futures_util::stream::{self, StreamExt};
use tauri::Emitter;

use super::super::WsEventHandler;
use super::identity::{
    apply_confirmed_bot_identity, canonical_riot_id, live_position, resolve_local_live_team, same_riot_id,
    select_enemy_slot,
};
use crate::infrastructure::champion_selection::summoner_spells;
use crate::infrastructure::data_services::{champion_data, static_catalog};
use crate::shared::Result;

const ENEMY_HISTORY_FETCH_CONCURRENCY: usize = 2;

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
    /// Backfills detailed match history for the enemy team during the 'InProgress' phase.
    pub(in crate::infrastructure::real_time::websocket::event_handler) async fn backfill_enemy_team_data(
        &self,
        generation: u64,
    ) -> Result<()> {
        log::info!("开始补全敌方阵容数据");

        if !self.cache.read().await.can_commit_in_game_recovery(generation) {
            return Ok(());
        }

        static_catalog::ensure_static_catalogs()
            .await
            .map_err(|error| format!("Static catalog is unavailable for enemy backfill: {error}"))?;

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

        let enemy_live_players: Vec<_> = live_players
            .into_iter()
            .filter(|player| player.team != local_team)
            .collect();

        // LiveClient is authoritative once the game starts. Reconcile bots before filtering
        // real players so an all-bot enemy team still replaces hidden champ-select identities.
        let mut occupied_enemy_slots = HashSet::new();
        for live_player in enemy_live_players.iter().filter(|player| player.is_bot) {
            let Some(champion_id) = champion_data::get_champion_id_by_name(&live_player.champion_name) else {
                log::warn!("Could not resolve bot champion '{}'.", live_player.champion_name);
                continue;
            };

            let Some(enemy_index) = select_enemy_slot(
                &team_analysis.enemy_team,
                &occupied_enemy_slots,
                live_player,
                champion_id,
            ) else {
                log::warn!(
                    "Could not match bot '{}' (champion: {}) to the cached enemy roster.",
                    live_player.summoner_name,
                    live_player.champion_name
                );
                continue;
            };

            occupied_enemy_slots.insert(enemy_index);
            let enemy_player = &mut team_analysis.enemy_team[enemy_index];
            apply_confirmed_bot_identity(enemy_player, live_player, champion_id);
        }

        let enemy_live_players: Vec<_> = enemy_live_players
            .into_iter()
            .filter(|player| !player.is_bot && !player.summoner_name.is_empty())
            .collect();

        if enemy_live_players.is_empty() {
            let mut cache = self.cache.write().await;
            if !cache.can_commit_in_game_recovery(generation) {
                return Ok(());
            }
            cache.team_analysis_data = Some(team_analysis.clone());
            cache.in_game_recovery_abort = None;
            let _ = self.app.emit("team-analysis-data", &team_analysis);
            log::info!("Enemy roster contains no real players; confirmed bot identities were published.");
            return Ok(());
        }

        log::info!(
            "识别到 {} 名真实敌方玩家，开始补全身份、段位与战绩",
            enemy_live_players.len()
        );

        // 4. Batch fetch summoner info.
        let player_names: Vec<String> = enemy_live_players.iter().map(|p| p.summoner_name.clone()).collect();

        let summoners_info =
            match crate::infrastructure::data_services::summoner::service::get_summoners_by_names_with_rank(
                &self.client,
                player_names.clone(),
            )
            .await
            {
                Ok(info) => {
                    log::info!("已获取 {} 名敌方玩家的身份与段位", info.len());
                    info
                }
                Err(e) => {
                    log::error!(
                        "Batch fetch for enemy summoner info failed: {}. Publishing the resolved LiveClient roster without rank/history.",
                        e
                    );
                    Vec::new()
                }
            };

        // 5. Resolve every enemy identity first. The frontend can then show names/ranks while
        // history requests continue in parallel instead of waiting for the entire team.
        let mut history_targets = Vec::new();

        for live_player in enemy_live_players {
            // 5.1 Find champion ID by name.
            let Some(champion_id) = champion_data::get_champion_id_by_name(&live_player.champion_name) else {
                log::warn!(
                    "Could not find champion ID for '{}', skipping player.",
                    live_player.champion_name
                );
                continue;
            };

            // 5.2 Reconcile the LiveClient identity with the cached champ-select roster.
            // Some queues expose five fully anonymous slots (no name, PUUID or champion),
            // so unmatched live players must occupy the remaining anonymous slots.
            let enemy_index = match select_enemy_slot(
                &team_analysis.enemy_team,
                &occupied_enemy_slots,
                &live_player,
                champion_id,
            ) {
                Some(index) => index,
                None => {
                    log::warn!(
                        "Could not find player '{}' (champion: {}) in cached enemy team, skipping.",
                        live_player.summoner_name,
                        live_player.champion_name
                    );
                    continue;
                }
            };
            occupied_enemy_slots.insert(enemy_index);
            let enemy_player = &mut team_analysis.enemy_team[enemy_index];

            // 5.3 Update basic player info.
            enemy_player.display_name = live_player.summoner_name.clone();
            enemy_player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Loading;
            enemy_player.champion_id = Some(champion_id);
            enemy_player.champion_name = Some(live_player.champion_name.clone());
            enemy_player.position = live_position(&live_player.position);

            // LiveClient uses localized spell names, while the frontend consumes stable numeric IDs.
            if let Some(spells) = live_player.summoner_spells.as_object() {
                // 技能1
                if let Some(spell_one) = spells.get("summonerSpellOne") {
                    if let Some(spell_name) = spell_one.get("displayName").and_then(|v| v.as_str()) {
                        if let Some(spell_id) = summoner_spells::get_spell_id_by_name(spell_name) {
                            enemy_player.spell1_id = Some(spell_id);
                            log::trace!("召唤师技能映射 slot=1 name={} id={}", spell_name, spell_id);
                        }
                    }
                }

                // 技能2
                if let Some(spell_two) = spells.get("summonerSpellTwo") {
                    if let Some(spell_name) = spell_two.get("displayName").and_then(|v| v.as_str()) {
                        if let Some(spell_id) = summoner_spells::get_spell_id_by_name(spell_name) {
                            enemy_player.spell2_id = Some(spell_id);
                            log::trace!("召唤师技能映射 slot=2 name={} id={}", spell_name, spell_id);
                        }
                    }
                }
            }

            // 5.4 Find the corresponding summoner info.
            let summoner_info = summoners_info.iter().find(|s| {
                let full_name = if let (Some(game_name), Some(tag_line)) = (&s.game_name, &s.tag_line) {
                    format!("{}#{}", game_name, tag_line)
                } else {
                    s.display_name.clone()
                };
                same_riot_id(&full_name, &live_player.summoner_name)
            });

            if let Some(info) = summoner_info {
                // 5.5 Update rank, icon, etc.
                enemy_player.display_name =
                    canonical_riot_id(&info.display_name, info.game_name.as_deref(), info.tag_line.as_deref());
                enemy_player.puuid = Some(info.puuid.clone());
                crate::infrastructure::match_management::analysis_data::service::apply_rank_summaries(
                    enemy_player,
                    info,
                );
                enemy_player.profile_icon_id = Some(info.profile_icon_id as i32);
                enemy_player.tag_line = info.tag_line.clone();

                history_targets.push((enemy_index, info.puuid.clone(), live_player.summoner_name.clone()));
            } else {
                enemy_player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Unavailable;
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
                .map(|(index, puuid, display_name)| async move {
                    let result = crate::infrastructure::match_management::analysis_data::service::fetch_realtime_player_analysis_with_retry(
                        &self.client,
                        &puuid,
                        queue_id,
                        is_custom_game,
                    )
                    .await;
                    (index, puuid, display_name, result)
                }),
        )
        .buffer_unordered(ENEMY_HISTORY_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

        let mut stats_to_cache = Vec::new();
        for (enemy_index, puuid, display_name, result) in results {
            match result {
                Ok(analysis) => {
                    stats_to_cache.push((
                        crate::infrastructure::match_management::analysis_data::service::MatchStatsCacheKey::new(
                            &puuid,
                            queue_id,
                            is_custom_game,
                        ),
                        analysis.clone(),
                    ));
                    let enemy_player = &mut team_analysis.enemy_team[enemy_index];
                    crate::infrastructure::match_management::analysis_data::service::apply_player_analysis(
                        enemy_player,
                        analysis,
                        queue_id,
                    );
                    enemy_player.is_bot = false;
                    log::debug!("敌方玩家分析完成: {}", display_name);
                }
                Err(error) => {
                    let enemy_player = &mut team_analysis.enemy_team[enemy_index];
                    enemy_player.analysis_status =
                        refresh_failure_status(enemy_player.analysis.is_some(), error.status());
                    log::warn!("Failed to get match history for player '{}': {}", display_name, error);
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
        log::info!(
            "敌方阵容数据补全完成，共处理 {} 名真实玩家",
            updated_data.enemy_team.iter().filter(|player| !player.is_bot).count()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::refresh_failure_status;
    use crate::shared::types::PlayerAnalysisStatus;

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
}
