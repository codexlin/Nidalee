use futures_util::stream::{self, StreamExt};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::RwLock;

use super::super::WsEventHandler;
use super::identity::same_riot_id;
use super::player_analysis::base_player_data;
use crate::infrastructure::data_services::{champion_data, static_catalog};
use crate::infrastructure::real_time::websocket::event_handler::context::in_progress_gameflow_context;
use crate::infrastructure::real_time::websocket::state::EventCache;
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

fn build_base_recovery_snapshot(
    my_team_players: &[crate::shared::types::LiveClientPlayer],
    enemy_team_players: &[crate::shared::types::LiveClientPlayer],
    local_name: &str,
    queue_id: i64,
    is_custom_game: bool,
) -> Result<crate::shared::types::TeamAnalysisData> {
    let mut my_team = my_team_players
        .iter()
        .enumerate()
        .map(|(index, player)| {
            base_player_data(
                player,
                index as i32,
                champion_data::get_champion_id_by_name(&player.champion_name),
            )
        })
        .collect::<Vec<_>>();
    let enemy_team = enemy_team_players
        .iter()
        .enumerate()
        .map(|(index, player)| {
            base_player_data(
                player,
                (index + 100) as i32,
                champion_data::get_champion_id_by_name(&player.champion_name),
            )
        })
        .collect::<Vec<_>>();

    let local_player = my_team
        .iter_mut()
        .find(|player| same_riot_id(&player.display_name, local_name))
        .ok_or_else(|| format!("Could not map local player '{local_name}' into recovered team data"))?;
    local_player.is_local = true;
    let local_player_cell_id = local_player.cell_id;

    Ok(crate::shared::types::TeamAnalysisData {
        my_team,
        enemy_team,
        local_player_cell_id,
        game_phase: "InProgress".to_string(),
        queue_id,
        is_custom_game,
        actions: None,
        bans: None,
        timer: None,
    })
}

async fn publish_base_recovery_snapshot<F>(
    cache: &Arc<RwLock<EventCache>>,
    generation: u64,
    snapshot: crate::shared::types::TeamAnalysisData,
    emit: F,
) -> bool
where
    F: FnOnce(&crate::shared::types::TeamAnalysisData),
{
    let mut cache = cache.write().await;
    if !cache.can_commit_in_game_recovery(generation) {
        return false;
    }
    cache.team_analysis_data = Some(snapshot.clone());
    emit(&snapshot);
    true
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

        let (cached_gameflow_session, cached_current_summoner) = {
            let cache = self.cache.read().await;
            if !cache.can_commit_in_game_recovery(generation) {
                return Ok(());
            }
            (cache.gameflow_session.clone(), cache.current_summoner.clone())
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
        let local_name = if let Some(summoner) = cached_current_summoner {
            match (&summoner.game_name, &summoner.tag_line) {
                (Some(game_name), Some(tag_line)) => format!("{game_name}#{tag_line}"),
                _ => summoner.display_name,
            }
        } else if let Ok(name) = crate::infrastructure::real_time::liveclient::service::get_active_player_name().await {
            name
        } else {
            let summoner = crate::infrastructure::data_services::summoner::service::get_current_summoner(&self.client)
                .await
                .map_err(|error| format!("Failed to resolve the local player identity: {error}"))?;
            match (&summoner.game_name, &summoner.tag_line) {
                (Some(game_name), Some(tag_line)) => format!("{game_name}#{tag_line}"),
                _ => summoner.display_name,
            }
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

        // 3. LiveClient 已经给出了权威阵容。先使用已有上下文发布无需远程补全的基础卡片，
        // 避免后续任何 HTTP 请求让页面在整个恢复阶段保持空白。
        let (base_queue_id, base_is_custom_game) = recovery_context_values(cached_gameflow_session.as_ref(), None);
        // 4. 校验、写缓存和 emit 必须在同一把锁内。
        let base_snapshot = build_base_recovery_snapshot(
            &my_team_players,
            &enemy_team_players,
            &local_name,
            base_queue_id,
            base_is_custom_game,
        )?;
        if !publish_base_recovery_snapshot(&self.cache, generation, base_snapshot, |snapshot| {
            let _ = self.app.emit("team-analysis-data", snapshot);
        })
        .await
        {
            return Ok(());
        }

        // 5. 基础阵容发布后，再在锁外刷新模式上下文并批量补全身份与段位。
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

        let all_player_names: Vec<String> = live_players
            .iter()
            .filter(|player| !player.is_bot && !player.summoner_name.is_empty())
            .map(|player| player.summoner_name.clone())
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
                    log::debug!(target: "ws::event", "Fetched {} summoner details", info.len());
                    info
                }
                Err(error) => {
                    log::warn!(
                        target: "ws::event",
                        "Batch fetch for summoner info failed; keeping the published LiveClient roster without identity enrichment: {error}"
                    );
                    Vec::new()
                }
            }
        };

        if !self.cache.read().await.can_commit_in_game_recovery(generation) {
            return Ok(());
        }

        // 6. 并发补全每名玩家的身份与战绩。
        let mut my_team_data = Vec::new();
        let mut enemy_team_data = Vec::new();
        let mut stats_to_cache = Vec::new();
        let jobs = my_team_players
            .into_iter()
            .enumerate()
            .map(|(index, player)| (true, index as i32, player))
            .chain(
                enemy_team_players
                    .into_iter()
                    .enumerate()
                    .map(|(index, player)| (false, (index + 100) as i32, player)),
            );
        let summoners_info = summoners_info.as_slice();
        let handler = self;
        let player_results = stream::iter(jobs.map(|(is_ally, cell_id, live_player)| async move {
            let player_data = handler
                .build_player_data(&live_player, cell_id, summoners_info, queue_id, is_custom_game)
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

        let local_champion_id = team_analysis_data
            .my_team
            .iter()
            .find(|player| player.is_local)
            .and_then(|player| player.champion_id);
        crate::infrastructure::augment_overlay::state::on_in_game_champion(
            self.app.clone(),
            local_champion_id,
            Some(team_analysis_data.queue_id).filter(|id| *id > 0),
        );

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
    use super::{
        build_base_recovery_snapshot, publish_base_recovery_snapshot, recovery_context_values, split_roster_by_team,
    };
    use crate::infrastructure::real_time::websocket::state::EventCache;
    use crate::shared::types::{LiveClientPlayer, PlayerAnalysisStatus, TeamAnalysisData};
    use futures_util::FutureExt;
    use std::sync::{Arc, Mutex};
    use tokio::sync::RwLock;

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

    #[tokio::test]
    async fn base_roster_is_published_before_slow_enrichment_completes() {
        let allies = vec![live_player("local#CN1", "ORDER", false), live_player("", "ORDER", true)];
        let enemies = vec![live_player("enemy#CN1", "CHAOS", false)];
        let snapshot =
            build_base_recovery_snapshot(&allies, &enemies, "local#CN1", 420, false).expect("基础阵容应可构建");
        let cache = Arc::new(RwLock::new(EventCache {
            gameflow_phase: Some("InProgress".to_string()),
            ..EventCache::default()
        }));
        let generation = cache.read().await.in_game_recovery_generation;
        let emitted = Arc::new(Mutex::new(Vec::<TeamAnalysisData>::new()));
        let emitted_for_callback = Arc::clone(&emitted);

        assert!(
            publish_base_recovery_snapshot(&cache, generation, snapshot, move |snapshot| {
                emitted_for_callback.lock().unwrap().push(snapshot.clone());
            })
            .await
        );

        let slow_enrichment = std::future::pending::<()>();
        assert!(slow_enrichment.now_or_never().is_none());
        let published = emitted.lock().unwrap();
        assert_eq!(published.len(), 1, "慢补全完成前必须已经发布一次基础阵容");
        assert!(published[0].my_team[0].is_local);
        assert_eq!(published[0].my_team[0].analysis_status, PlayerAnalysisStatus::Loading);
        assert_eq!(published[0].my_team[1].analysis_status, PlayerAnalysisStatus::Bot);
        assert_eq!(
            published[0].enemy_team[0].analysis_status,
            PlayerAnalysisStatus::Loading
        );
    }
}
