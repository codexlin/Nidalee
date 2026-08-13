use futures_util::stream::{self, StreamExt};
use tauri::Emitter;

use super::super::WsEventHandler;
use super::identity::same_riot_id;
use crate::infrastructure::data_services::static_catalog;
use crate::infrastructure::real_time::websocket::event_handler::context::in_progress_gameflow_context;
use crate::shared::Result;

const RECOVERY_PLAYER_FETCH_CONCURRENCY: usize = 4;

fn split_roster_by_team(
    live_players: &[crate::shared::types::LiveClientPlayer],
    local_team: &str,
) -> (
    Vec<crate::shared::types::LiveClientPlayer>,
    Vec<crate::shared::types::LiveClientPlayer>,
) {
    live_players
        .iter()
        .cloned()
        .partition(|player| player.team == local_team)
}

fn recovery_context_values(
    cached_session: Option<&serde_json::Value>,
    fetched_session: Option<&serde_json::Value>,
) -> (i64, bool) {
    in_progress_gameflow_context(cached_session, fetched_session)
        .map_or((0, false), |context| (context.queue_id, context.is_custom_game))
}

impl WsEventHandler {
    /// 从头构建队伍分析数据（应用重启后没有缓存时使用）
    /// 同时处理我方和敌方队伍的数据
    pub(in crate::infrastructure::real_time::websocket::event_handler) async fn build_team_data_from_scratch(
        &self,
        generation: u64,
    ) -> Result<()> {
        log::info!(
            target: "ws::event",
            "Building team data from LiveClient (app restart recovery)"
        );

        let cached_gameflow_session = {
            let cache = self.cache.read().await;
            if !cache.can_commit_in_game_recovery(generation) {
                return Ok(());
            }
            cache.gameflow_session.clone()
        };

        if let Err(error) = static_catalog::ensure_static_catalogs().await {
            log::warn!(
                target: "ws::event",
                "Static catalog is unavailable during in-game recovery; unresolved champion IDs will remain empty: {error}"
            );
        }

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
                            target: "ws::event",
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
                            target: "ws::event",
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
                            target: "ws::event",
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
            target: "ws::event",
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

        // LiveClient owns the active roster. Bots and players whose identity lookup fails still
        // need stable cards, so only the local player's side decides the split.
        let (my_team_players, enemy_team_players) = split_roster_by_team(&live_players, local_team);

        log::debug!(
            target: "ws::event",
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

        let summoners_info = if all_player_names.is_empty() {
            Vec::new()
        } else {
            match crate::infrastructure::data_services::summoner::service::get_summoners_by_names_with_rank(
                &self.client,
                all_player_names,
            )
            .await
            {
                Ok(info) => {
                    log::debug!(
                        target: "ws::event",
                        "Fetched {} summoner details",
                        info.len()
                    );
                    info
                }
                Err(error) => {
                    log::warn!(
                        target: "ws::event",
                        "Batch fetch for summoner info failed; publishing the LiveClient roster without identity enrichment: {error}"
                    );
                    Vec::new()
                }
            }
        };

        // 4. 先获取游戏信息（需要用于战绩过滤）
        let fetched_gameflow_session =
            match crate::infrastructure::game_session::gameflow::service::get_gameflow_session(&self.client).await {
                Ok(session) => Some(session),
                Err(error) => {
                    log::warn!(
                        target: "ws::event",
                        "Failed to refresh gameflow context during recovery; falling back to the cached session: {error}"
                    );
                    None
                }
            };
        let (queue_id, is_custom_game) =
            recovery_context_values(cached_gameflow_session.as_ref(), fetched_gameflow_session.as_ref());
        if queue_id == 0 {
            log::warn!(
                target: "ws::event",
                "Game mode is unknown during recovery; history analysis will use the explicit unknown-mode policy"
            );
        }

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
            let player_data = handler
                .build_player_data(
                    &live_player,
                    cell_id,
                    summoners_info,
                    queue_id,
                    is_custom_game,
                    perspective,
                )
                .await;
            (is_ally, player_data)
        }))
        .buffered(RECOVERY_PLAYER_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

        for (is_ally, (player_data, cache_entry)) in player_results {
            if let Some(entry) = cache_entry {
                stats_to_cache.push(entry);
            }
            if is_ally {
                my_team_data.push(player_data);
            } else {
                enemy_team_data.push(player_data);
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
                target: "ws::event",
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
            target: "ws::event",
            "TeamAnalysisData cached, match_stats_cache size: {}",
            cache.match_stats_cache.len()
        );

        drop(cache);

        // 10. 发送到前端
        log::info!(
            target: "ws::event",
            "Build from scratch completed: my_team={}, enemy_team={}",
            team_analysis_data.my_team.len(),
            team_analysis_data.enemy_team.len()
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{recovery_context_values, split_roster_by_team};
    use crate::shared::types::LiveClientPlayer;

    fn live_player(name: &str, team: &str, is_bot: bool) -> LiveClientPlayer {
        LiveClientPlayer {
            summoner_name: name.to_owned(),
            champion_name: "Champion".to_owned(),
            is_bot,
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
            team: team.to_owned(),
        }
    }

    #[test]
    fn split_roster_by_team_keeps_bots_and_empty_identities() {
        let players = vec![
            live_player("local#CN1", "ORDER", false),
            live_player("", "ORDER", true),
            live_player("enemy#CN1", "CHAOS", false),
            live_player("", "CHAOS", true),
        ];

        let (allies, enemies) = split_roster_by_team(&players, "ORDER");

        assert_eq!(
            (
                allies.len(),
                enemies.len(),
                allies.iter().filter(|player| player.is_bot).count(),
                enemies.iter().filter(|player| player.is_bot).count(),
            ),
            (2, 2, 1, 1)
        );
    }

    #[test]
    fn recovery_context_values_never_invent_ranked_queue() {
        let context = recovery_context_values(None, None);

        assert_eq!(context, (0, false));
    }

    #[test]
    fn recovery_context_values_prefers_fresh_session_over_cached_session() {
        let cached = serde_json::json!({
            "phase": "InProgress",
            "gameData": { "queue": { "id": 450 }, "isCustomGame": false }
        });
        let fresh = serde_json::json!({
            "phase": "InProgress",
            "gameData": { "queue": { "id": 440 }, "isCustomGame": false }
        });

        let context = recovery_context_values(Some(&cached), Some(&fresh));

        assert_eq!(context, (440, false));
    }
}
