use futures_util::stream::{self, StreamExt};
use tauri::Emitter;

use super::super::WsEventHandler;
use super::identity::same_riot_id;
use crate::infrastructure::data_services::static_catalog;
use crate::shared::Result;

const RECOVERY_PLAYER_FETCH_CONCURRENCY: usize = 4;

impl WsEventHandler {
    /// 从头构建队伍分析数据（应用重启后没有缓存时使用）
    /// 同时处理我方和敌方队伍的数据
    pub(in crate::infrastructure::real_time::websocket::event_handler) async fn build_team_data_from_scratch(
        &self,
        generation: u64,
    ) -> Result<()> {
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
}
