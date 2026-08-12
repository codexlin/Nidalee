use super::WsEventHandler;
use crate::infrastructure::champion_selection::summoner_spells;
use crate::infrastructure::data_services::{champion_data, static_catalog};
use crate::shared::Result;
use futures_util::stream::{self, StreamExt};
use std::collections::HashSet;
use tauri::Emitter;

const ENEMY_HISTORY_FETCH_CONCURRENCY: usize = 2;
const RECOVERY_PLAYER_FETCH_CONCURRENCY: usize = 4;

type MatchStatsCacheEntry = (
    crate::infrastructure::match_management::analysis_data::service::MatchStatsCacheKey,
    crate::infrastructure::match_management::analysis_data::service::CachedPlayerAnalysis,
);

fn same_riot_id(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    if left.eq_ignore_ascii_case(right) {
        return true;
    }

    let (left_name, left_has_tag) = left.split_once('#').map_or((left, false), |(name, _)| (name, true));
    let (right_name, right_has_tag) = right.split_once('#').map_or((right, false), |(name, _)| (name, true));

    // Only fall back to the game name when one source omitted the tag line.
    (!left_has_tag || !right_has_tag) && left_name.eq_ignore_ascii_case(right_name)
}

fn resolve_local_live_team<'a>(
    live_players: &'a [crate::shared::types::LiveClientPlayer],
    team_analysis: &crate::shared::types::TeamAnalysisData,
) -> Option<&'a str> {
    let local_player = team_analysis.my_team.iter().find(|player| player.is_local)?;

    live_players
        .iter()
        .find(|player| !player.is_bot && same_riot_id(&local_player.display_name, &player.summoner_name))
        .or_else(|| {
            let local_champion_id = local_player.champion_id?;
            live_players.iter().find(|player| {
                !player.is_bot
                    && champion_data::get_champion_id_by_name(&player.champion_name) == Some(local_champion_id)
            })
        })
        .map(|player| player.team.as_str())
}

fn apply_confirmed_bot_identity(
    cached_player: &mut crate::shared::types::PlayerAnalysisData,
    live_player: &crate::shared::types::LiveClientPlayer,
    champion_id: i32,
) {
    cached_player.display_name = live_player.summoner_name.clone();
    cached_player.is_bot = true;
    cached_player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Bot;
    cached_player.champion_id = Some(champion_id);
    cached_player.champion_name = Some(live_player.champion_name.clone());
    cached_player.position = live_position(&live_player.position);
    cached_player.spell1_id = live_spell_id(&live_player.summoner_spells, "summonerSpellOne");
    cached_player.spell2_id = live_spell_id(&live_player.summoner_spells, "summonerSpellTwo");
    cached_player.match_stats = None;
    cached_player.recent_matches.clear();
    cached_player.analysis_basis = None;
}

fn live_spell_id(spells: &serde_json::Value, slot: &str) -> Option<i64> {
    spells
        .get(slot)?
        .get("displayName")?
        .as_str()
        .and_then(summoner_spells::get_spell_id_by_name)
}

fn live_position(position: &str) -> Option<String> {
    let position = position.trim();
    (!position.is_empty() && !position.eq_ignore_ascii_case("NONE")).then(|| position.to_owned())
}

fn select_enemy_slot(
    enemy_team: &[crate::shared::types::PlayerAnalysisData],
    occupied_slots: &HashSet<usize>,
    live_player: &crate::shared::types::LiveClientPlayer,
    champion_id: i32,
) -> Option<usize> {
    let available = |index: &usize| !occupied_slots.contains(index);

    enemy_team
        .iter()
        .enumerate()
        .filter(|(index, _)| available(index))
        .find(|(_, player)| same_riot_id(&player.display_name, &live_player.summoner_name))
        .or_else(|| {
            enemy_team
                .iter()
                .enumerate()
                .filter(|(index, _)| available(index))
                .find(|(_, player)| player.champion_id == Some(champion_id))
        })
        .or_else(|| {
            enemy_team
                .iter()
                .enumerate()
                .filter(|(index, _)| available(index))
                .find(|(_, player)| {
                    player.analysis_status == crate::shared::types::PlayerAnalysisStatus::AwaitingIdentity
                })
        })
        .map(|(index, _)| index)
}

impl WsEventHandler {
    pub(crate) async fn retry_player_analysis(
        &self,
        puuid: &str,
    ) -> Result<crate::shared::types::PlayerAnalysisStatus> {
        let puuid = puuid.trim();
        if puuid.is_empty() {
            return Err("Player PUUID is required".into());
        }

        let (is_champ_select, generation, queue_id, is_custom_game, perspective) = {
            let mut cache = self.cache.write().await;
            let (is_champ_select, generation) = match cache.gameflow_phase.as_deref() {
                Some("ChampSelect") => (true, cache.champ_select_analysis_generation),
                Some("InProgress") => (false, cache.in_game_recovery_generation),
                _ => return Err("Player analysis can only be retried during champion select or an active game".into()),
            };
            let analysis = cache
                .team_analysis_data
                .as_mut()
                .ok_or("No active team analysis data is available")?;
            let queue_id = analysis.queue_id;
            let is_custom_game = analysis.is_custom_game;
            let perspective = if let Some(player) = analysis
                .my_team
                .iter_mut()
                .find(|player| player.puuid.as_deref() == Some(puuid))
            {
                if player.is_bot {
                    return Err("Bot players do not require match analysis".into());
                }
                if player.analysis_status == crate::shared::types::PlayerAnalysisStatus::Loading {
                    return Ok(crate::shared::types::PlayerAnalysisStatus::Loading);
                }
                player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Loading;
                if player.is_local {
                    crate::shared::types::AdvicePerspective::SelfImprovement
                } else {
                    crate::shared::types::AdvicePerspective::Collaboration
                }
            } else if let Some(player) = analysis
                .enemy_team
                .iter_mut()
                .find(|player| player.puuid.as_deref() == Some(puuid))
            {
                if player.is_bot {
                    return Err("Bot players do not require match analysis".into());
                }
                if player.analysis_status == crate::shared::types::PlayerAnalysisStatus::Loading {
                    return Ok(crate::shared::types::PlayerAnalysisStatus::Loading);
                }
                player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Loading;
                crate::shared::types::AdvicePerspective::Targeting
            } else {
                return Err("Player is no longer part of the active team analysis".into());
            };

            let updated_data = analysis.clone();
            let _ = self.app.emit("team-analysis-data", &updated_data);
            (is_champ_select, generation, queue_id, is_custom_game, perspective)
        };

        let result =
            crate::infrastructure::match_management::analysis_data::service::fetch_realtime_player_analysis_with_retry(
                &self.client,
                puuid,
                queue_id,
                is_custom_game,
            )
            .await;

        let mut cache = self.cache.write().await;
        let can_commit = if is_champ_select {
            cache.gameflow_phase.as_deref() == Some("ChampSelect") && cache.can_commit_champ_select_analysis(generation)
        } else {
            cache.can_commit_in_game_recovery(generation)
        };
        if !can_commit {
            return Err("The active game changed while the player was being analyzed".into());
        }

        let analysis = cache
            .team_analysis_data
            .as_mut()
            .ok_or("Team analysis data was cleared while retrying the player")?;
        let player = analysis
            .my_team
            .iter_mut()
            .chain(analysis.enemy_team.iter_mut())
            .find(|player| player.puuid.as_deref() == Some(puuid))
            .ok_or("Player left the active team analysis while the retry was running")?;

        let (cache_entry, command_result) = match result {
            Ok(player_analysis) => {
                player.match_stats = Some(player_analysis.stats.clone());
                player.recent_matches = player_analysis.recent_matches.clone();
                player.analysis_basis = Some(player_analysis.basis.clone());
                player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Ready;
                crate::infrastructure::match_management::analysis_data::service::refresh_ranked_rating(
                    player, queue_id,
                );
                (
                    Some(player_analysis),
                    Ok(crate::shared::types::PlayerAnalysisStatus::Ready),
                )
            }
            Err(error) => {
                player.match_stats = None;
                player.recent_matches.clear();
                player.analysis_basis = None;
                player.ranked_rating = None;
                player.analysis_status = error.status();
                let status = error.status();
                let command_result = if error.is_unavailable() {
                    Err(error.to_string().into())
                } else {
                    Ok(status)
                };
                (None, command_result)
            }
        };

        let updated_data = analysis.clone();
        if let Some(player_analysis) = cache_entry {
            cache.match_stats_cache.insert(
                crate::infrastructure::match_management::analysis_data::service::MatchStatsCacheKey::new(
                    puuid,
                    queue_id,
                    is_custom_game,
                    perspective,
                ),
                player_analysis,
            );
        }
        let _ = self.app.emit("team-analysis-data", &updated_data);
        command_result
    }

    /// Backfills detailed match history for the enemy team during the 'InProgress' phase.
    pub(super) async fn backfill_enemy_team_data(&self, generation: u64) -> Result<()> {
        log::info!("[ws-event-backfill] Starting backfill task...");

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
                        log::info!("[ws-event-backfill] Successfully fetched player list from LiveClient.");
                        break players;
                    }
                    Ok(_) => {
                        if attempts >= max_attempts {
                            return Err(
                                "Failed to get player list from LiveClient after multiple attempts: returned an empty list (game loading?)".into(),
                            );
                        }
                        log::warn!(
                            "[ws-event-backfill] LiveClient returned an empty list (game loading?), attempt {}/{}...",
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
                            "[ws-event-backfill] Failed to fetch LiveClient player list (attempt {}/{}), retrying in 2s: {}",
                            attempts,
                            max_attempts,
                            e
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
        };

        log::info!(
            "[ws-event-backfill] Found {} players in LiveClient data.",
            live_players.len()
        );

        // 2. Snapshot cached analysis data. All network I/O below operates on this owned copy;
        // no EventCache lock is held across an await.
        let mut team_analysis = {
            let cache = self.cache.read().await;
            if !cache.can_commit_in_game_recovery(generation) {
                log::debug!("[ws-event-backfill] Recovery generation {generation} is stale before backfill");
                return Ok(());
            }
            match cache.team_analysis_data.clone() {
                Some(data) => data,
                None => {
                    log::warn!("[ws-event-backfill] No TeamAnalysisData in cache, cannot perform backfill.");
                    return Ok(());
                }
            }
        };

        // 3. Resolve the local side from the local player instead of assuming ORDER.
        // Custom games can place the local player on either side.
        let Some(local_team) = resolve_local_live_team(&live_players, &team_analysis).map(str::to_owned) else {
            log::warn!(
                "[ws-event-backfill] Could not resolve the local player's LiveClient team; skipping roster backfill."
            );
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
                log::warn!(
                    "[ws-event-backfill] Could not resolve bot champion '{}'.",
                    live_player.champion_name
                );
                continue;
            };

            let Some(enemy_index) = select_enemy_slot(
                &team_analysis.enemy_team,
                &occupied_enemy_slots,
                live_player,
                champion_id,
            ) else {
                log::warn!(
                    "[ws-event-backfill] Could not match bot '{}' (champion: {}) to the cached enemy roster.",
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
            log::info!(
                "[ws-event-backfill] Enemy roster contains no real players; confirmed bot identities were published."
            );
            return Ok(());
        }

        log::info!(
            "[ws-event-backfill] Found {} real enemy players, starting data backfill...",
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
                    log::info!(
                        "[ws-event-backfill] Successfully fetched details for {} enemy summoners.",
                        info.len()
                    );
                    info
                }
                Err(e) => {
                    log::error!(
                        "[ws-event-backfill] Batch fetch for enemy summoner info failed: {}. Publishing the resolved LiveClient roster without rank/history.",
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
                    "[ws-event-backfill] Could not find champion ID for '{}', skipping player.",
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
                        "[ws-event-backfill] Could not find player '{}' (champion: {}) in cached enemy team, skipping.",
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

            // 5.3.1 解析并转换召唤师技能（从中文名转为 ID）
            if let Some(spells) = live_player.summoner_spells.as_object() {
                // 技能1
                if let Some(spell_one) = spells.get("summonerSpellOne") {
                    if let Some(spell_name) = spell_one.get("displayName").and_then(|v| v.as_str()) {
                        if let Some(spell_id) = summoner_spells::get_spell_id_by_name(spell_name) {
                            enemy_player.spell1_id = Some(spell_id);
                            log::debug!(
                                "[ws-event-backfill] 转换召唤师技能1: '{}' -> ID {}",
                                spell_name,
                                spell_id
                            );
                        }
                    }
                }

                // 技能2
                if let Some(spell_two) = spells.get("summonerSpellTwo") {
                    if let Some(spell_name) = spell_two.get("displayName").and_then(|v| v.as_str()) {
                        if let Some(spell_id) = summoner_spells::get_spell_id_by_name(spell_name) {
                            enemy_player.spell2_id = Some(spell_id);
                            log::debug!(
                                "[ws-event-backfill] 转换召唤师技能2: '{}' -> ID {}",
                                spell_name,
                                spell_id
                            );
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
                    "[ws-event-backfill] Could not find detailed summoner info for '{}'.",
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
                            crate::shared::types::AdvicePerspective::Targeting,
                        ),
                        analysis.clone(),
                    ));
                    let enemy_player = &mut team_analysis.enemy_team[enemy_index];
                    enemy_player.match_stats = Some(analysis.stats);
                    enemy_player.recent_matches = analysis.recent_matches;
                    enemy_player.analysis_basis = Some(analysis.basis);
                    enemy_player.is_bot = false;
                    enemy_player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Ready;
                    log::info!(
                        "[ws-event-backfill] Successfully backfilled full data for player '{}'.",
                        display_name
                    );
                }
                Err(error) => {
                    team_analysis.enemy_team[enemy_index].analysis_status = error.status();
                    log::warn!(
                        "[ws-event-backfill] Failed to get match history for player '{}': {}",
                        display_name,
                        error
                    );
                }
            }
        }

        let updated_data = team_analysis;

        // Commit only if this is still the active in-game generation.
        let mut cache = self.cache.write().await;
        if !cache.can_commit_in_game_recovery(generation) {
            log::debug!("[ws-event-backfill] Discarding stale recovery generation {generation}");
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
        log::info!("[ws-event-backfill] Emitting complete, backfilled analysis data to frontend.");
        log::info!("[ws-event-backfill] Backfill task completed.");

        Ok(())
    }

    /// 从头构建队伍分析数据（应用重启后没有缓存时使用）
    /// 同时处理我方和敌方队伍的数据
    pub(super) async fn build_team_data_from_scratch(&self, generation: u64) -> Result<()> {
        log::info!(
            target: "ws::event_handler",
            "Building team data from LiveClient (app restart recovery)"
        );

        if !self.cache.read().await.can_commit_in_game_recovery(generation) {
            return Ok(());
        }

        static_catalog::ensure_static_catalogs()
            .await
            .map_err(|error| format!("Static catalog is unavailable for in-game recovery: {error}"))?;

        if !self.cache.read().await.can_commit_in_game_recovery(generation) {
            return Ok(());
        }

        // 1. 获取 LiveClient 玩家列表（带重试）
        let live_players = {
            let mut attempts = 0;
            let max_attempts = 30;
            loop {
                attempts += 1;
                match crate::infrastructure::real_time::liveclient::service::get_live_player_list().await {
                    Ok(players) if !players.is_empty() => {
                        log::debug!(
                            target: "ws::event_handler",
                            "Fetched {} players from LiveClient",
                            players.len()
                        );
                        break players;
                    }
                    Ok(_) => {
                        if attempts >= max_attempts {
                            return Err("LiveClient returned empty list after max retries (game still loading?)".into());
                        }
                        log::debug!(
                            target: "ws::event_handler",
                            "LiveClient empty, retrying {}/{}",
                            attempts,
                            max_attempts
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                    Err(e) => {
                        if attempts >= max_attempts {
                            return Err(format!(
                                "Failed to get LiveClient data after {} attempts: {}",
                                max_attempts, e
                            )
                            .into());
                        }
                        log::warn!(
                            target: "ws::event_handler",
                            "LiveClient fetch failed (attempt {}/{}): {}, retrying",
                            attempts,
                            max_attempts,
                            e
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    }
                }
            }
        };

        log::debug!(
            target: "ws::event_handler",
            "Found {} players in LiveClient data",
            live_players.len()
        );

        // 2. 分离我方（ORDER）和敌方（CHAOS）队伍
        let local_summoner =
            crate::infrastructure::data_services::summoner::service::get_current_summoner(&self.client)
                .await
                .map_err(|error| format!("Failed to resolve the current summoner: {error}"))?;
        let local_name = match (&local_summoner.game_name, &local_summoner.tag_line) {
            (Some(game_name), Some(tag_line)) => format!("{game_name}#{tag_line}"),
            _ => local_summoner.display_name.clone(),
        };
        let local_team = live_players
            .iter()
            .find(|player| !player.is_bot && same_riot_id(&local_name, &player.summoner_name))
            .map(|player| player.team.as_str())
            .ok_or_else(|| format!("Could not find local player '{local_name}' in LiveClient roster"))?;

        // Resolve sides from the local player. The local side is not guaranteed to be ORDER.
        let my_team_players: Vec<_> = live_players
            .iter()
            .filter(|p| p.team == local_team && !p.is_bot && !p.summoner_name.is_empty())
            .cloned()
            .collect();

        let enemy_team_players: Vec<_> = live_players
            .iter()
            .filter(|p| p.team != local_team && !p.is_bot && !p.summoner_name.is_empty())
            .cloned()
            .collect();

        log::debug!(
            target: "ws::event_handler",
            "Team split: {} my team, {} enemy team",
            my_team_players.len(),
            enemy_team_players.len()
        );

        // 3. 收集所有玩家名称用于批量获取召唤师信息
        let all_player_names: Vec<String> = live_players
            .iter()
            .filter(|p| !p.is_bot && !p.summoner_name.is_empty())
            .map(|p| p.summoner_name.clone())
            .collect();

        let summoners_info =
            match crate::infrastructure::data_services::summoner::service::get_summoners_by_names_with_rank(
                &self.client,
                all_player_names.clone(),
            )
            .await
            {
                Ok(info) => {
                    log::debug!(
                        target: "ws::event_handler",
                        "Fetched {} summoner details",
                        info.len()
                    );
                    info
                }
                Err(e) => {
                    log::error!(
                        target: "ws::event_handler",
                        "Batch fetch for summoner info failed: {}",
                        e
                    );
                    return Err(format!("Failed to fetch summoner info: {}", e).into());
                }
            };

        // 4. 先获取游戏信息（需要用于战绩过滤）
        let (queue_id, is_custom_game) =
            match crate::infrastructure::game_session::gameflow::service::get_gameflow_session(&self.client).await {
                Ok(session) => {
                    let queue_id = session["gameData"]["queue"]["id"].as_i64().unwrap_or(420);
                    let is_custom = session["gameData"]["isCustomGame"].as_bool().unwrap_or(false);
                    log::debug!(
                        target: "ws::event_handler",
                        "Got game info from API: queue_id={}, is_custom={}",
                        queue_id,
                        is_custom
                    );
                    (queue_id, is_custom)
                }
                Err(e) => {
                    log::warn!(
                        target: "ws::event_handler",
                        "Failed to get gameflow session: {}, using defaults",
                        e
                    );
                    (420, false)
                }
            };

        // 5. 构建我方队伍数据
        let mut my_team_data = Vec::new();
        let mut enemy_team_data = Vec::new();
        let mut stats_to_cache = Vec::new();
        let jobs = my_team_players
            .into_iter()
            .enumerate()
            .map(|(index, player)| {
                (
                    true,
                    index as i32,
                    player,
                    crate::shared::types::AdvicePerspective::Collaboration,
                )
            })
            .chain(enemy_team_players.into_iter().enumerate().map(|(index, player)| {
                (
                    false,
                    (index + 100) as i32,
                    player,
                    crate::shared::types::AdvicePerspective::Targeting,
                )
            }));
        let summoners_info = summoners_info.as_slice();
        let handler = self;
        let player_results = stream::iter(jobs.map(|(is_ally, cell_id, live_player, perspective)| async move {
            let player_name = live_player.summoner_name.clone();
            let result = handler
                .build_player_data(
                    &live_player,
                    cell_id,
                    summoners_info,
                    queue_id,
                    is_custom_game,
                    perspective,
                )
                .await;
            (is_ally, player_name, result)
        }))
        .buffered(RECOVERY_PLAYER_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

        for (is_ally, player_name, result) in player_results {
            match result {
                Ok((player_data, cache_entry)) => {
                    if let Some(entry) = cache_entry {
                        stats_to_cache.push(entry);
                    }
                    if is_ally {
                        my_team_data.push(player_data);
                    } else {
                        enemy_team_data.push(player_data);
                    }
                }
                Err(error) => log::warn!(
                    target: "ws::event_handler",
                    "Failed to build recovery data for player '{}': {}",
                    player_name,
                    error
                ),
            }
        }

        let local_player = my_team_data
            .iter_mut()
            .find(|player| same_riot_id(&player.display_name, &local_name))
            .ok_or_else(|| format!("Could not map local player '{local_name}' into recovered team data"))?;
        local_player.is_local = true;
        let local_player_cell_id = local_player.cell_id;

        // 8. 创建 TeamAnalysisData
        let team_analysis_data = crate::shared::types::TeamAnalysisData {
            my_team: my_team_data,
            enemy_team: enemy_team_data,
            local_player_cell_id,
            game_phase: "InProgress".to_string(),
            queue_id,
            is_custom_game,
            actions: None,
            bans: None,
            timer: None,
        };

        // 9. 缓存数据
        let mut cache = self.cache.write().await;

        // 批量插入战绩缓存
        if !cache.can_commit_in_game_recovery(generation) {
            log::debug!(
                target: "ws::event_handler",
                "Discarding stale from-scratch recovery generation {}",
                generation
            );
            return Ok(());
        }

        for (key, stats) in stats_to_cache {
            cache.match_stats_cache.insert(key, stats);
        }

        cache.team_analysis_data = Some(team_analysis_data.clone());
        cache.in_game_recovery_abort = None;
        // Keep validation, cache commit and publication atomic with respect to phase changes.
        let _ = self.app.emit("team-analysis-data", &team_analysis_data);
        log::debug!(
            target: "ws::event_handler",
            "TeamAnalysisData cached, match_stats_cache size: {}",
            cache.match_stats_cache.len()
        );

        drop(cache);

        // 10. 发送到前端
        log::info!(
            target: "ws::event_handler",
            "Build from scratch completed: my_team={}, enemy_team={}",
            team_analysis_data.my_team.len(),
            team_analysis_data.enemy_team.len()
        );

        Ok(())
    }

    /// 辅助方法：从 LiveClient 玩家数据构建 PlayerAnalysisData
    async fn build_player_data(
        &self,
        live_player: &crate::shared::types::LiveClientPlayer,
        cell_id: i32,
        summoners_info: &[crate::shared::types::SummonerInfo],
        queue_id: i64,
        is_custom_game: bool,
        perspective: crate::shared::types::AdvicePerspective,
    ) -> Result<(crate::shared::types::PlayerAnalysisData, Option<MatchStatsCacheEntry>)> {
        // 获取英雄 ID
        let champion_id = champion_data::get_champion_id_by_name(&live_player.champion_name)
            .ok_or_else(|| format!("Could not find champion ID for '{}'", live_player.champion_name))?;

        // 查找召唤师信息
        let summoner_info = summoners_info.iter().find(|s| {
            let full_name = if let (Some(game_name), Some(tag_line)) = (&s.game_name, &s.tag_line) {
                format!("{}#{}", game_name, tag_line)
            } else {
                s.display_name.clone()
            };
            same_riot_id(&full_name, &live_player.summoner_name)
        });

        let mut player_data = crate::shared::types::PlayerAnalysisData {
            cell_id,
            display_name: live_player.summoner_name.clone(),
            summoner_id: None,
            puuid: None,
            is_local: false,
            is_bot: false,
            analysis_status: crate::shared::types::PlayerAnalysisStatus::Loading,
            champion_id: Some(champion_id),
            champion_name: Some(live_player.champion_name.clone()),
            champion_pick_intent: None,
            position: live_position(&live_player.position),
            solo_rank: None,
            flex_rank: None,
            profile_icon_id: None,
            tag_line: None,
            spell1_id: None,
            spell2_id: None,
            match_stats: None,
            recent_matches: Vec::new(),
            analysis_basis: None,
            ranked_rating: None,
        };
        let mut cache_entry = None;

        // 解析召唤师技能
        if let Some(spells) = live_player.summoner_spells.as_object() {
            if let Some(spell_one) = spells.get("summonerSpellOne") {
                if let Some(spell_name) = spell_one.get("displayName").and_then(|v| v.as_str()) {
                    if let Some(spell_id) = summoner_spells::get_spell_id_by_name(spell_name) {
                        player_data.spell1_id = Some(spell_id);
                    }
                }
            }

            if let Some(spell_two) = spells.get("summonerSpellTwo") {
                if let Some(spell_name) = spell_two.get("displayName").and_then(|v| v.as_str()) {
                    if let Some(spell_id) = summoner_spells::get_spell_id_by_name(spell_name) {
                        player_data.spell2_id = Some(spell_id);
                    }
                }
            }
        }

        // 填充召唤师详细信息
        if let Some(info) = summoner_info {
            player_data.puuid = Some(info.puuid.clone());
            crate::infrastructure::match_management::analysis_data::service::apply_rank_summaries(
                &mut player_data,
                info,
            );
            player_data.profile_icon_id = Some(info.profile_icon_id as i32);
            player_data.tag_line = info.tag_line.clone();

            // 获取战绩数据
            match crate::infrastructure::match_management::analysis_data::service::fetch_realtime_player_analysis_with_retry(
                &self.client,
                &info.puuid,
                queue_id,
                is_custom_game,
            )
            .await
            {
                Ok(analysis) => {
                    cache_entry = Some((
                        crate::infrastructure::match_management::analysis_data::service::MatchStatsCacheKey::new(
                            &info.puuid,
                            queue_id,
                            is_custom_game,
                            perspective,
                        ),
                        analysis.clone(),
                    ));
                    player_data.match_stats = Some(analysis.stats);
                    player_data.recent_matches = analysis.recent_matches;
                    player_data.analysis_basis = Some(analysis.basis);
                    player_data.analysis_status = crate::shared::types::PlayerAnalysisStatus::Ready;
                    crate::infrastructure::match_management::analysis_data::service::refresh_ranked_rating(
                        &mut player_data,
                        queue_id,
                    );

                    log::debug!(
                        target: "ws::event_handler",
                        "Fetched match data for player '{}'",
                        live_player.summoner_name
                    );
                }
                Err(e) => {
                    player_data.analysis_status = e.status();
                    log::warn!(
                        target: "ws::event_handler",
                        "Failed to get match history for player '{}': {}",
                        live_player.summoner_name,
                        e
                    );
                }
            }
        } else {
            player_data.analysis_status = crate::shared::types::PlayerAnalysisStatus::Unavailable;
            log::warn!(
                target: "ws::event_handler",
                "Could not find detailed summoner info for '{}'",
                live_player.summoner_name
            );
        }

        Ok((player_data, cache_entry))
    }
}

#[cfg(test)]
mod tests {
    use super::{live_position, same_riot_id, select_enemy_slot};
    use crate::shared::types::{LiveClientPlayer, PlayerAnalysisData, PlayerAnalysisStatus};
    use std::collections::HashSet;

    fn anonymous_enemy(cell_id: i32) -> PlayerAnalysisData {
        PlayerAnalysisData {
            cell_id,
            display_name: "敌方选手".to_owned(),
            summoner_id: None,
            puuid: None,
            is_local: false,
            is_bot: false,
            analysis_status: PlayerAnalysisStatus::AwaitingIdentity,
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
            match_stats: None,
            recent_matches: Vec::new(),
            analysis_basis: None,
            ranked_rating: None,
        }
    }

    fn live_player(name: &str, champion_name: &str) -> LiveClientPlayer {
        LiveClientPlayer {
            summoner_name: name.to_owned(),
            champion_name: champion_name.to_owned(),
            is_bot: false,
            is_dead: false,
            items: Vec::new(),
            level: 1,
            position: String::new(),
            raw_champion_name: String::new(),
            respawn_timer: 0.0,
            runes: serde_json::Value::Null,
            scores: serde_json::Value::Null,
            skin_id: 0,
            summoner_spells: serde_json::Value::Null,
            team: "CHAOS".to_owned(),
        }
    }

    #[test]
    fn riot_id_matching_preserves_tag_when_both_sources_have_one() {
        assert!(same_riot_id("bbs#23912", "BBS#23912"));
        assert!(!same_riot_id("bbs#23912", "bbs#other"));
        assert!(same_riot_id("bbs", "bbs#23912"));
    }

    #[test]
    fn live_position_omits_none_sentinel() {
        assert_eq!(live_position("NONE"), None);
        assert_eq!(live_position("JUNGLE").as_deref(), Some("JUNGLE"));
    }

    #[test]
    fn anonymous_enemy_slots_are_assigned_once_in_stable_order() {
        let enemies = vec![anonymous_enemy(0), anonymous_enemy(1)];
        let first = live_player("first#CN1", "Katarina");
        let second = live_player("second#CN1", "Ezreal");
        let mut occupied = HashSet::new();

        let first_index = select_enemy_slot(&enemies, &occupied, &first, 55).unwrap();
        occupied.insert(first_index);
        let second_index = select_enemy_slot(&enemies, &occupied, &second, 81).unwrap();

        assert_eq!(first_index, 0);
        assert_eq!(second_index, 1);
    }

    #[test]
    fn known_enemy_identity_wins_over_anonymous_slot() {
        let mut known = anonymous_enemy(1);
        known.display_name = "Known#CN1".to_owned();
        known.analysis_status = PlayerAnalysisStatus::Loading;
        let enemies = vec![anonymous_enemy(0), known];

        let index = select_enemy_slot(&enemies, &HashSet::new(), &live_player("known#CN1", "Ezreal"), 81);

        assert_eq!(index, Some(1));
    }
}
