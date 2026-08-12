use super::context::champ_select_session_with_gameflow_context;
use super::WsEventHandler;
use crate::shared::types::TeamAnalysisData;
use crate::shared::Result;
use serde_json::Value;
use tauri::Emitter;

impl WsEventHandler {
    pub(super) async fn handle_gameflow_phase_change(&self, data: &Value, event_type: &str) -> Result<()> {
        log::info!("[ws-event] Gameflow phase change ({}) received: {}", event_type, data);
        if event_type == "Create" || event_type == "Update" {
            if let Some(phase) = data.as_str() {
                let recovery_plan = {
                    let mut cache = self.cache.write().await;
                    if cache.gameflow_phase.as_deref() == Some(phase) {
                        log::debug!("[ws-event] Gameflow phase unchanged, skipping broadcast: {}", phase);
                        return Ok(());
                    }

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
                            let _ = self.app.emit("team-analysis-data", &None::<TeamAnalysisData>);
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
        Ok(())
    }

    pub(super) async fn handle_gameflow_session_change(&self, data: &Value, event_type: &str) -> Result<()> {
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
            let _ = self.app.emit("gameflow-session-changed", &None::<Value>);
        }
        Ok(())
    }
}
