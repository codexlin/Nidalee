// Handles converting raw WebSocket messages into application-specific events.
mod enrichment;

use super::decoder::{decode_json_api_event, JsonApiEvent};
use super::state::EventCache;
use crate::infrastructure::match_management::analysis_data;
use crate::shared::types::{LobbyInfo, MatchmakingState, SummonerInfo};
use crate::shared::Result;
use reqwest::Client;
use serde_json::Value;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct WsEventHandler {
    app: AppHandle,
    cache: Arc<RwLock<EventCache>>,
    client: Client, // HTTP client for fetching summoner information.
}

impl WsEventHandler {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            cache: Arc::new(RwLock::new(EventCache::default())),
            client: crate::http_client::get_lcu_client().clone(),
        }
    }

    /// Gets the cached team analysis data.
    pub async fn get_cached_team_analysis_data(&self) -> Option<crate::shared::types::TeamAnalysisData> {
        let cache = self.cache.read().await;
        cache.team_analysis_data.clone()
    }

    /// Feeds an HTTP fallback snapshot through the same reducer used by WebSocket events.
    pub(crate) async fn handle_snapshot(&self, uri: &str, data: Value) -> Result<()> {
        self.dispatch_json_api_event(&JsonApiEvent {
            uri: uri.to_owned(),
            event_type: "Update".to_owned(),
            data,
        })
        .await
    }

    /// Replays current reducer state for listeners registered after the supervisor started.
    /// The read lock keeps replay ordered with every cache write + synchronous emit.
    pub(crate) async fn replay_cached_state(&self) {
        let cache = self.cache.read().await;
        if let Some(phase) = cache.gameflow_phase.as_ref() {
            let _ = self.app.emit("gameflow-phase-change", &Some(phase.clone()));
        }
        if let Some(session) = cache.gameflow_session.as_ref() {
            let _ = self.app.emit("gameflow-session-changed", session.clone());
        }
        if let Some(summoner) = cache.current_summoner.as_ref() {
            let _ = self.app.emit("summoner-change", &Some(summoner.clone()));
        }
        if let Some(lobby) = cache.lobby_info.as_ref() {
            let _ = self.app.emit("lobby-change", &Some(lobby.clone()));
        }
        if let Some(matchmaking) = cache.matchmaking_state.as_ref() {
            let _ = self.app.emit("matchmaking-state-changed", matchmaking);
        }
        if let Some(champ_select) = cache.champ_select_session.as_ref() {
            let _ = self.app.emit("champ-select-session-changed", champ_select);
        }
        if let Some(analysis) = cache.team_analysis_data.as_ref() {
            let _ = self.app.emit("team-analysis-data", analysis);
        }
    }

    /// Invalidates transport-scoped de-duplication and cancels work tied to the old socket.
    /// Game-scoped analysis is also cleared: after reconnect, the next in-game snapshot must
    /// rebuild from authoritative state rather than reuse a skeleton from a previous game.
    pub(crate) async fn handle_transport_disconnected(&self) {
        let mut cache = self.cache.write().await;
        cache.cancel_champ_select_analysis();
        cache.cancel_in_game_recovery();
        cache.gameflow_phase = None;
        cache.gameflow_session = None;
        cache.current_summoner = None;
        cache.champ_select_session = None;
        cache.matchmaking_state = None;
        cache.lobby_info = None;
        cache.match_stats_cache.clear();
        cache.team_analysis_data = None;
        let _ = self
            .app
            .emit("team-analysis-data", &None::<crate::shared::types::TeamAnalysisData>);
    }

    /// Handles a raw WebSocket event message.
    pub async fn handle_event(&self, event_data: &str) -> Result<()> {
        if event_data.trim().is_empty() {
            return Err("Received empty event data".into());
        }

        let event = decode_json_api_event(event_data).map_err(|e| {
            format!(
                "Failed to parse WebSocket event (length: {}, first 100 chars: '{}'): {}",
                event_data.len(),
                event_data.chars().take(100).collect::<String>(),
                e
            )
        })?;

        if let Some(event) = event {
            if Self::is_important_event(&event.uri) {
                self.dispatch_json_api_event(&event).await?;
            }
        }

        Ok(())
    }

    /// Checks if the event is one of the critical events that need to be processed.
    fn is_important_event(uri: &str) -> bool {
        matches!(
            uri,
            "/lol-gameflow/v1/gameflow-phase"
                | "/lol-champ-select/v1/session"
                | "/lol-lobby/v2/lobby"
                | "/lol-matchmaking/v1/search"
                | "/lol-gameflow/v1/session"
                | "/lol-summoner/v1/current-summoner"
        )
    }

    async fn dispatch_json_api_event(&self, event: &JsonApiEvent) -> Result<()> {
        match event.uri.as_str() {
            "/lol-gameflow/v1/gameflow-phase" => {
                self.handle_gameflow_phase_change(&event.data, &event.event_type)
                    .await?;
            }
            "/lol-gameflow/v1/session" => {
                self.handle_gameflow_session_change(&event.data, &event.event_type)
                    .await?;
            }
            "/lol-champ-select/v1/session" => {
                self.handle_champ_select_change(&event.data, &event.event_type).await?;
            }
            "/lol-lobby/v2/lobby" => {
                self.handle_lobby_change(&event.data, &event.event_type).await?;
            }
            "/lol-summoner/v1/current-summoner" => {
                self.handle_current_summoner_change(&event.data, &event.event_type)
                    .await?;
            }
            "/lol-matchmaking/v1/search" => {
                self.handle_matchmaking_change(&event.data, &event.event_type).await?;
            }
            _ => {
                // Other events are logged at trace level but not processed.
                log::trace!("[ws-event] Unhandled event URI: {}", event.uri);
            }
        }

        Ok(())
    }

    async fn handle_gameflow_phase_change(&self, data: &Value, event_type: &str) -> Result<()> {
        log::info!("[ws-event] Gameflow phase change ({}) received: {}", event_type, data);
        if event_type == "Create" || event_type == "Update" {
            if let Some(phase) = data.as_str() {
                let recovery_plan = {
                    let mut cache = self.cache.write().await;
                    if cache.gameflow_phase.as_deref() == Some(phase) {
                        log::debug!("[ws-event] Gameflow phase unchanged, skipping broadcast: {}", phase);
                        return Ok(());
                    }

                    // Any phase transition invalidates work created by the preceding phase/game.
                    cache.cancel_in_game_recovery();
                    cache.gameflow_phase = Some(phase.to_string());
                    if phase != "ChampSelect" {
                        cache.cancel_champ_select_analysis();
                    }

                    let recovery_plan = if phase == "InProgress" {
                        let has_cache = cache.team_analysis_data.is_some();
                        Some((cache.in_game_recovery_generation, has_cache))
                    } else if phase == "Lobby"
                        || phase == "None"
                        || phase == "EndOfGame"
                        || phase == "WaitingForStats"
                        || phase == "PreEndOfGame"
                        || phase == "Terminated"
                    {
                        // ✅ 优化：游戏结束或返回大厅时，立即清空缓存释放内存
                        // 每把游戏的缓存独立，防止长期运行导致内存累积
                        let cache_size = cache.match_stats_cache.len();
                        if cache_size > 0 {
                            log::info!(
                                target: "ws::event_handler",
                                "Phase '{}': clearing match stats cache ({} players, ~{}KB released)",
                                phase,
                                cache_size,
                                cache_size * 10
                            );
                            cache.match_stats_cache.clear();
                        }
                        if cache.team_analysis_data.is_some() {
                            log::info!(
                                target: "ws::event_handler",
                                "Phase '{}': clearing team analysis data",
                                phase
                            );
                            cache.team_analysis_data = None;
                            let _ = self
                                .app
                                .emit("team-analysis-data", &None::<crate::shared::types::TeamAnalysisData>);
                        }
                        None
                    } else {
                        None
                    };
                    let _ = self.app.emit("gameflow-phase-change", &Some(phase.to_string()));
                    recovery_plan
                };

                if let Some((generation, has_cache)) = recovery_plan {
                    let handler = self.clone();
                    // Do not let the task start network work until its abort handle has been
                    // published into the current generation.
                    let (start_tx, start_rx) = tokio::sync::oneshot::channel();
                    let task = tokio::spawn(async move {
                        if start_rx.await.is_err() {
                            return;
                        }

                        let result = if has_cache {
                            log::info!("[ws-event] Cache available, starting enemy team backfill...");
                            handler.backfill_enemy_team_data(generation).await
                        } else {
                            log::info!("[ws-event] No cache, building team data from scratch...");
                            handler.build_team_data_from_scratch(generation).await
                        };

                        if let Err(error) = result {
                            log::error!("[ws-event-recovery] In-game recovery failed: {}", error);
                        }

                        let mut cache = handler.cache.write().await;
                        if cache.in_game_recovery_generation == generation {
                            cache.in_game_recovery_abort = None;
                        }
                    });
                    let abort_handle = task.abort_handle();

                    let mut cache = self.cache.write().await;
                    if cache.can_commit_in_game_recovery(generation) {
                        cache.in_game_recovery_abort = Some(abort_handle);
                        let _ = start_tx.send(());
                    } else {
                        abort_handle.abort();
                    }
                }
            }
        } else if event_type == "Delete" {
            {
                let mut cache = self.cache.write().await;
                cache.cancel_champ_select_analysis();
                cache.cancel_in_game_recovery();
                let should_emit = cache.gameflow_phase.is_some();
                cache.gameflow_phase = None;
                if should_emit {
                    log::info!("[ws-event] Gameflow phase cleared.");
                    let _ = self.app.emit("gameflow-phase-change", &None::<String>);
                }
            }
        }
        Ok(())
    }

    async fn handle_gameflow_session_change(&self, data: &Value, event_type: &str) -> Result<()> {
        // Create and Update events both contain data.
        if event_type == "Create" || event_type == "Update" {
            let mut cache = self.cache.write().await;

            let session_changed = cache.gameflow_session.as_ref() != Some(data);

            if session_changed {
                cache.gameflow_session = Some(data.clone());
                let _ = self.app.emit("gameflow-session-changed", data);
                log::debug!("[ws-event] Gameflow session updated and broadcast ({}).", event_type);

                if let Some(champ_select) = cache.champ_select_session.clone() {
                    let session = champ_select_session_with_gameflow_context(&champ_select, Some(data));
                    if session != champ_select {
                        cache.champ_select_session = Some(session.clone());
                        let _ = self.app.emit("champ-select-session-changed", &session);
                    }
                }
            } else {
                log::trace!("[ws-event] Gameflow session unchanged, skipping broadcast.");
            }

            // Note: We don't emit a gameflow-phase-change event here because
            // /lol-gameflow/v1/gameflow-phase triggers its own handler. This prevents duplicate events.

            #[cfg(debug_assertions)]
            {
                log::debug!("[ws-event] Gameflow Session Details:");
                log::debug!("  Phase: {:?}", data["phase"]);
                log::debug!("  Map: {:?}", data["map"]["name"]);
                log::debug!("  GameData: {:?}", data["gameData"]["queue"]);
            }
        } else if event_type == "Delete" {
            log::info!("[ws-event] Gameflow session cleared.");

            let mut cache = self.cache.write().await;
            cache.gameflow_session = None;
            // Only emit the session deletion event.
            // Phase deletion is handled by handle_gameflow_phase_change.
            let _ = self.app.emit("gameflow-session-changed", &None::<Value>);
        }
        Ok(())
    }

    async fn handle_champ_select_change(&self, data: &Value, event_type: &str) -> Result<()> {
        log::info!("[ws-event] Champ select event received, type: {}", event_type);

        if event_type == "Create" || event_type == "Update" {
            // Step 1: Immediately send raw session data for fast auto-pick response.
            log::info!("[ws-event] Sending raw champ-select-session-changed event (immediate)");
            let session = {
                let mut cache = self.cache.write().await;
                let session = champ_select_session_with_gameflow_context(data, cache.gameflow_session.as_ref());
                cache.champ_select_session = Some(session.clone());
                let _ = self.app.emit("champ-select-session-changed", &session);

                // Champion intent, spells and assigned position change frequently. Keep the
                // enriched roster current without repeating summoner/rank/history requests.
                let analysis_key = analysis_data::service::champ_select_analysis_key(&session);
                if cache.champ_select_analysis_key.as_ref() == Some(&analysis_key) {
                    let patched = cache
                        .team_analysis_data
                        .as_mut()
                        .filter(|analysis| analysis.game_phase == "ChampSelect")
                        .is_some_and(|analysis| {
                            analysis_data::service::patch_team_analysis_from_session(analysis, &session)
                        });
                    if patched {
                        if let Some(analysis) = cache.team_analysis_data.as_ref() {
                            let _ = self.app.emit("team-analysis-data", analysis);
                        }
                    }
                }
                session
            };
            let analysis_key = analysis_data::service::champ_select_analysis_key(&session);

            // Step 2: Debounce the expensive enrichment work. Champ-select emits frequent
            // timer/action updates, while the raw event above must remain immediate for auto-pick.
            let app = self.app.clone();
            let client = self.client.clone();
            let cache_for_task = Arc::clone(&self.cache);
            let data_clone = session;
            let analysis_work = {
                let mut cache = self.cache.write().await;
                if cache.champ_select_analysis_key.as_ref() == Some(&analysis_key) {
                    log::trace!("[ws-event] Champ-select analysis inputs unchanged; skipping enrichment");
                    None
                } else {
                    cache.cancel_champ_select_analysis();
                    cache.champ_select_analysis_key = Some(analysis_key);

                    Some((cache.champ_select_analysis_generation, cache.match_stats_cache.clone()))
                }
            };

            let Some((generation, mut match_stats_cache)) = analysis_work else {
                return Ok(());
            };

            let task = tokio::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;

                // Do not start remote enrichment if a newer session arrived during the debounce.
                if !cache_for_task.read().await.can_commit_champ_select_analysis(generation) {
                    return;
                }

                let result = analysis_data::service::build_team_analysis_from_session(
                    &data_clone,
                    &client,
                    &mut match_stats_cache,
                )
                .await;

                match result {
                    Ok(mut enriched_data) => {
                        let mut cache = cache_for_task.write().await;
                        if !cache.can_commit_champ_select_analysis(generation) {
                            log::debug!("[ws-event] Discarding stale team analysis generation {}", generation);
                            return;
                        }

                        // The roster identity may be unchanged while champion/spell updates
                        // arrived during the network work. Commit the latest volatile projection,
                        // not the snapshot captured before the debounce.
                        if let Some(latest_session) = cache.champ_select_session.as_ref() {
                            analysis_data::service::patch_team_analysis_from_session(
                                &mut enriched_data,
                                latest_session,
                            );
                        }

                        log::info!("[ws-event] Successfully generated enriched team analysis data (with match stats).");
                        log::debug!(
                            "[ws-event] My team size: {}, Enemy team size: {}",
                            enriched_data.my_team.len(),
                            enriched_data.enemy_team.len()
                        );
                        log::info!(
                            "[ws-event] Current cached match stats count: {}",
                            match_stats_cache.len()
                        );

                        // Commit the local cache only after the latest generation wins. No global
                        // write lock is held while network requests are in flight.
                        cache.match_stats_cache = match_stats_cache;
                        cache.team_analysis_data = Some(enriched_data.clone());
                        cache.champ_select_analysis_abort = None;
                        log::info!("[ws-event] Enriched TeamAnalysisData has been cached.");

                        // Publish under the same generation guard as the cache commit so a newer
                        // champ-select session cannot invalidate this result in between.
                        let _ = app.emit("team-analysis-data", &enriched_data);
                        drop(cache);
                    }
                    Err(e) => {
                        let mut cache = cache_for_task.write().await;
                        if cache.can_commit_champ_select_analysis(generation) {
                            cache.champ_select_analysis_abort = None;
                            cache.champ_select_analysis_key = None;
                        }
                        drop(cache);

                        log::error!("[ws-event] Failed to generate enriched team analysis data: {}", e);
                        if let Some(source) = e.source() {
                            log::error!("[ws-event] Caused by: {}", source);
                        }
                        log::warn!("[ws-event] Session data already sent, match stats will be unavailable");
                    }
                }
            });

            let abort_handle = task.abort_handle();
            let mut cache = self.cache.write().await;
            if cache.can_commit_champ_select_analysis(generation) {
                cache.champ_select_analysis_abort = Some(abort_handle);
            } else {
                abort_handle.abort();
            }
        } else if event_type == "Delete" {
            log::info!("[ws-event] Champ select session cleared, but preserving analysis data for backfill.");

            let mut cache = self.cache.write().await;
            cache.cancel_champ_select_analysis();
            cache.champ_select_session = None;
            // The team_analysis_data is intentionally not cleared here.
            // It is needed for the backfill process when the game phase changes to 'InProgress'.
            // Cleanup is handled by handle_gameflow_phase_change after the game ends.
            // Send session deletion event to frontend.
            let _ = self.app.emit("champ-select-session-changed", &None::<Value>);
        }
        Ok(())
    }

    async fn handle_lobby_change(&self, data: &Value, event_type: &str) -> Result<()> {
        if event_type == "Create" || event_type == "Update" {
            if let Ok(lobby) = serde_json::from_value::<LobbyInfo>(data.clone()) {
                let mut cache = self.cache.write().await;
                if cache.lobby_info.as_ref() != Some(&lobby) {
                    log::info!("[ws-event] Lobby info updated ({}).", event_type);
                    cache.lobby_info = Some(lobby.clone());
                    let _ = self.app.emit("lobby-change", &Some(lobby));
                } else {
                    log::debug!("[ws-event] Lobby info unchanged, skipping broadcast.");
                }
            } else {
                log::warn!("[ws-event] Failed to parse lobby info, skipping.");
            }
        } else if event_type == "Delete" {
            let mut cache = self.cache.write().await;
            if cache.lobby_info.is_some() {
                log::info!("[ws-event] Left lobby.");
                cache.lobby_info = None;
                let _ = self.app.emit("lobby-change", &None::<LobbyInfo>);
            }
        }
        Ok(())
    }

    async fn handle_current_summoner_change(&self, data: &Value, event_type: &str) -> Result<()> {
        log::info!(
            target: "ws::event_handler",
            "🧪 [EXPERIMENTAL] Received current-summoner event, type: {}, testing if LCU supports this subscription",
            event_type
        );

        if event_type == "Create" || event_type == "Update" {
            // 尝试解析召唤师信息
            if let Ok(summoner) = serde_json::from_value::<SummonerInfo>(data.clone()) {
                log::info!(
                    target: "ws::event_handler",
                    "✅ [EXPERIMENTAL] LCU supports current-summoner subscription! Summoner: {}, Level: {}",
                    summoner.display_name,
                    summoner.summoner_level
                );

                let mut cache = self.cache.write().await;
                if cache.current_summoner.as_ref() != Some(&summoner) {
                    cache.current_summoner = Some(summoner.clone());
                    let _ = self.app.emit("summoner-change", &Some(summoner));
                }
            } else {
                log::warn!(
                    target: "ws::event_handler",
                    "⚠️ [EXPERIMENTAL] Failed to parse summoner data from WebSocket event"
                );
            }
        } else if event_type == "Delete" {
            log::info!(
                target: "ws::event_handler",
                "🧪 [EXPERIMENTAL] Summoner data cleared"
            );
            let mut cache = self.cache.write().await;
            if cache.current_summoner.take().is_some() {
                let _ = self.app.emit("summoner-change", &None::<SummonerInfo>);
            }
        }

        Ok(())
    }

    async fn handle_matchmaking_change(&self, data: &Value, event_type: &str) -> Result<()> {
        if event_type == "Create" || event_type == "Update" {
            if let Ok(matchmaking_state) = serde_json::from_value::<MatchmakingState>(data.clone()) {
                let mut cache = self.cache.write().await;
                if cache.matchmaking_state.as_ref() != Some(&matchmaking_state) {
                    log::info!(
                        "[ws-event] Matchmaking state updated ({}) to: {:?}",
                        event_type,
                        matchmaking_state.search_state
                    );
                    cache.matchmaking_state = Some(matchmaking_state.clone());
                    let _ = self.app.emit("matchmaking-state-changed", matchmaking_state);
                } else {
                    log::debug!(
                        "[ws-event] Matchmaking state unchanged, skipping broadcast: {:?}",
                        matchmaking_state.search_state
                    );
                }
            }
        } else if event_type == "Delete" {
            let mut cache = self.cache.write().await;
            if cache.matchmaking_state.is_some() {
                log::info!("[ws-event] Matchmaking state cleared.");
                cache.matchmaking_state = None;
                let _ = self.app.emit("matchmaking-state-changed", &None::<MatchmakingState>);
            }
        }
        Ok(())
    }
}

fn champ_select_session_with_gameflow_context(champ_select: &Value, gameflow_session: Option<&Value>) -> Value {
    let Some(context) = gameflow_session.and_then(gameflow_build_context) else {
        return champ_select.clone();
    };
    let Some(session) = champ_select.as_object() else {
        return champ_select.clone();
    };

    let mut enriched = session.clone();
    enriched.insert("queueId".to_string(), Value::from(context.queue_id));
    enriched.insert("isCustomGame".to_string(), Value::from(context.is_custom_game));
    Value::Object(enriched)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GameflowBuildContext {
    queue_id: i64,
    is_custom_game: bool,
}

fn gameflow_build_context(session: &Value) -> Option<GameflowBuildContext> {
    if session.get("phase")?.as_str()? != "ChampSelect" {
        return None;
    }
    let game_data = session.get("gameData")?;
    Some(GameflowBuildContext {
        queue_id: game_data.get("queue")?.get("id")?.as_i64()?,
        is_custom_game: game_data.get("isCustomGame")?.as_bool()?,
    })
}

#[cfg(test)]
mod tests {
    use super::{champ_select_session_with_gameflow_context, gameflow_build_context, EventCache, GameflowBuildContext};
    use serde_json::json;

    #[test]
    fn champ_select_session_uses_authoritative_gameflow_context() {
        let raw = json!({ "localPlayerCellId": 4, "queueId": 0, "isCustomGame": true, "myTeam": [] });
        let gameflow = json!({
            "phase": "ChampSelect",
            "gameData": {
                "queue": { "id": 440 },
                "isCustomGame": false
            }
        });

        let session = champ_select_session_with_gameflow_context(&raw, Some(&gameflow));

        assert_eq!(session["queueId"], 440);
        assert_eq!(session["isCustomGame"], false);
        assert_eq!(session["localPlayerCellId"], 4);
    }

    #[test]
    fn gameflow_context_requires_both_runtime_fields() {
        assert_eq!(
            gameflow_build_context(&json!({
                "phase": "ChampSelect",
                "gameData": { "queue": { "id": 450 }, "isCustomGame": false }
            })),
            Some(GameflowBuildContext {
                queue_id: 450,
                is_custom_game: false
            })
        );
        assert_eq!(
            gameflow_build_context(&json!({
                "phase": "ChampSelect",
                "gameData": { "queue": { "id": 450 } }
            })),
            None
        );
        assert_eq!(
            gameflow_build_context(&json!({
                "phase": "InProgress",
                "gameData": { "queue": { "id": 450 }, "isCustomGame": false }
            })),
            None
        );
    }

    #[test]
    fn in_game_recovery_generation_invalidates_old_work() {
        let mut cache = EventCache {
            gameflow_phase: Some("InProgress".to_string()),
            ..EventCache::default()
        };
        let generation = cache.in_game_recovery_generation;

        assert!(cache.can_commit_in_game_recovery(generation));

        cache.cancel_in_game_recovery();

        assert!(!cache.can_commit_in_game_recovery(generation));
        assert!(cache.can_commit_in_game_recovery(cache.in_game_recovery_generation));
    }

    #[test]
    fn in_game_recovery_cannot_commit_outside_in_progress() {
        let cache = EventCache::default();

        assert!(!cache.can_commit_in_game_recovery(cache.in_game_recovery_generation));
    }

    #[test]
    fn champ_select_generation_invalidates_old_enrichment() {
        let mut cache = EventCache::default();
        let generation = cache.champ_select_analysis_generation;

        assert!(cache.can_commit_champ_select_analysis(generation));
        cache.cancel_champ_select_analysis();
        assert!(!cache.can_commit_champ_select_analysis(generation));
    }
}
