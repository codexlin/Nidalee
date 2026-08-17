use super::context::{champ_select_session_with_gameflow_context, in_progress_gameflow_context, overlay_queue_id};
use super::WsEventHandler;
use crate::infrastructure::augment_overlay::state as augment_overlay;
use crate::shared::types::TeamAnalysisData;
use crate::shared::Result;
use serde_json::Value;
use tauri::Emitter;

pub(super) fn local_champion_id(session: Option<&Value>) -> Option<i32> {
    let session = session?;
    let cell_id = session.get("localPlayerCellId")?.as_i64()?;
    let team = session.get("myTeam")?.as_array()?;
    for player in team {
        if player.get("cellId").and_then(Value::as_i64) == Some(cell_id) {
            let id = player.get("championId").and_then(Value::as_i64)?;
            if id > 0 {
                return Some(id as i32);
            }
        }
    }
    None
}

impl WsEventHandler {
    async fn start_in_game_recovery(&self) {
        let (generation, has_cache) = {
            let cache = self.cache.read().await;
            if cache.gameflow_phase.as_deref() != Some("InProgress") {
                return;
            }
            (cache.in_game_recovery_generation, cache.team_analysis_data.is_some())
        };

        let handler = self.clone();
        let (start_tx, start_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            if start_rx.await.is_err() {
                return;
            }

            let result = if has_cache {
                log::info!("Cache available, starting in-game roster backfill...");
                handler.backfill_team_data(generation).await
            } else {
                log::info!("No cache, building team data from scratch...");
                handler.build_team_data_from_scratch(generation).await
            };

            if let Err(error) = result {
                log::error!("In-game recovery failed: {}", error);
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

    pub(super) async fn handle_gameflow_phase_change(&self, data: &Value, event_type: &str) -> Result<()> {
        log::info!("Gameflow phase change ({}) received: {}", event_type, data);
        if event_type == "Create" || event_type == "Update" {
            if let Some(phase) = data.as_str() {
                let should_start_recovery = {
                    let mut cache = self.cache.write().await;
                    if cache.gameflow_phase.as_deref() == Some(phase) {
                        log::debug!("Gameflow phase unchanged, skipping broadcast: {}", phase);
                        return Ok(());
                    }

                    cache.cancel_in_game_recovery();
                    cache.gameflow_phase = Some(phase.to_string());
                    if phase != "ChampSelect" {
                        cache.cancel_champ_select_analysis();
                    }

                    let should_start_recovery = if phase == "InProgress" {
                        true
                    } else if phase == "Lobby"
                        || phase == "None"
                        || phase == "EndOfGame"
                        || phase == "WaitingForStats"
                        || phase == "PreEndOfGame"
                        || phase == "Terminated"
                    {
                        let cache_size = cache.match_stats_cache.len();
                        if cache_size > 0 {
                            log::info!(
                                target: "ws::event",
                                "Phase '{}': clearing match stats cache ({} players, ~{}KB released)",
                                phase,
                                cache_size,
                                cache_size * 10
                            );
                            cache.match_stats_cache.clear();
                        }
                        if cache.team_analysis_data.is_some() {
                            log::info!(
                                target: "ws::event",
                                "Phase '{}': clearing team analysis data",
                                phase
                            );
                            cache.team_analysis_data = None;
                            let _ = self.app.emit("team-analysis-data", &None::<TeamAnalysisData>);
                        }
                        let cached_session_is_in_progress = cache
                            .gameflow_session
                            .as_ref()
                            .and_then(|session| session.get("phase"))
                            .and_then(Value::as_str)
                            == Some("InProgress");
                        if cached_session_is_in_progress {
                            cache.gameflow_session = None;
                        }
                        false
                    } else {
                        false
                    };
                    let _ = self.app.emit("gameflow-phase-change", &Some(phase.to_string()));
                    let champion_id = local_champion_id(cache.champ_select_session.as_ref());
                    let queue_id =
                        overlay_queue_id(cache.champ_select_session.as_ref(), cache.gameflow_session.as_ref());
                    augment_overlay::on_gameflow_phase(
                        self.app.clone(),
                        Some(phase.to_string()),
                        champion_id,
                        queue_id,
                    );
                    should_start_recovery
                };

                if should_start_recovery {
                    self.start_in_game_recovery().await;
                }
            }
        } else if event_type == "Delete" {
            let mut cache = self.cache.write().await;
            cache.cancel_champ_select_analysis();
            cache.cancel_in_game_recovery();
            let should_emit = cache.gameflow_phase.is_some();
            cache.gameflow_phase = None;
            if should_emit {
                log::info!("Gameflow phase cleared.");
                let _ = self.app.emit("gameflow-phase-change", &None::<String>);
            }
            augment_overlay::on_gameflow_phase(self.app.clone(), None, None, None);
        }
        Ok(())
    }

    pub(super) async fn handle_gameflow_session_change(&self, data: &Value, event_type: &str) -> Result<()> {
        if event_type == "Create" || event_type == "Update" {
            let mut cache = self.cache.write().await;
            let session_changed = cache.gameflow_session.as_ref() != Some(data);
            let previous_context = in_progress_gameflow_context(cache.gameflow_session.as_ref(), None);
            let current_context = in_progress_gameflow_context(Some(data), None);
            let should_restart_recovery = session_changed
                && cache.gameflow_phase.as_deref() == Some("InProgress")
                && current_context.is_some()
                && current_context != previous_context;

            if should_restart_recovery {
                cache.cancel_in_game_recovery();
                let analysis_context_changed =
                    current_context
                        .zip(cache.team_analysis_data.as_ref())
                        .is_some_and(|(context, analysis)| {
                            analysis.queue_id != context.queue_id || analysis.is_custom_game != context.is_custom_game
                        });
                if analysis_context_changed {
                    cache.team_analysis_data = None;
                    cache.match_stats_cache.clear();
                    let _ = self.app.emit("team-analysis-data", &None::<TeamAnalysisData>);
                }
            }

            if session_changed {
                cache.gameflow_session = Some(data.clone());
                let _ = self.app.emit("gameflow-session-changed", data);
                log::debug!("Gameflow session updated and broadcast ({}).", event_type);

                if let Some(champ_select) = cache.champ_select_session.clone() {
                    let session = champ_select_session_with_gameflow_context(&champ_select, Some(data));
                    if session != champ_select {
                        cache.champ_select_session = Some(session.clone());
                        let _ = self.app.emit("champ-select-session-changed", &session);
                    }
                }
            } else {
                log::trace!("Gameflow session unchanged, skipping broadcast.");
            }

            let queue = &data["gameData"]["queue"];
            log::debug!(
                "Gameflow context phase={} queue_id={} mode={} map_id={} ranked={} custom={}",
                data["phase"].as_str().unwrap_or("unknown"),
                queue["id"].as_i64().unwrap_or_default(),
                queue["gameMode"].as_str().unwrap_or("unknown"),
                queue["mapId"].as_i64().unwrap_or_default(),
                queue["isRanked"].as_bool().unwrap_or(false),
                queue["isCustom"].as_bool().unwrap_or(false),
            );
            let phase = data["phase"].as_str().map(str::to_string);
            let champion_id = local_champion_id(cache.champ_select_session.as_ref());
            let queue_id = overlay_queue_id(cache.champ_select_session.as_ref(), Some(data));
            drop(cache);

            if matches!(phase.as_deref(), Some("GameStart" | "InProgress" | "Reconnect")) {
                augment_overlay::on_gameflow_phase(self.app.clone(), phase, champion_id, queue_id);
            }

            if should_restart_recovery {
                log::info!("In-progress game context changed; restarting team recovery.");
                self.start_in_game_recovery().await;
            }
        } else if event_type == "Delete" {
            log::info!("Gameflow session cleared.");
            let mut cache = self.cache.write().await;
            cache.gameflow_session = None;
            let _ = self.app.emit("gameflow-session-changed", &None::<Value>);
        }
        Ok(())
    }
}
