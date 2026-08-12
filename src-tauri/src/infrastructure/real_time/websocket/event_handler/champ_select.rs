use std::sync::Arc;

use serde_json::Value;
use tauri::Emitter;

use super::context::champ_select_session_with_gameflow_context;
use super::WsEventHandler;
use crate::infrastructure::match_management::analysis_data;
use crate::shared::Result;

impl WsEventHandler {
    pub(super) async fn handle_champ_select_change(&self, data: &Value, event_type: &str) -> Result<()> {
        log::info!("[ws-event] Champ select event received, type: {}", event_type);

        if event_type == "Create" || event_type == "Update" {
            log::info!("[ws-event] Sending raw champ-select-session-changed event (immediate)");
            let session = {
                let mut cache = self.cache.write().await;
                let session = champ_select_session_with_gameflow_context(data, cache.gameflow_session.as_ref());
                cache.champ_select_session = Some(session.clone());
                let _ = self.app.emit("champ-select-session-changed", &session);

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

                        cache.match_stats_cache = match_stats_cache;
                        cache.team_analysis_data = Some(enriched_data.clone());
                        cache.champ_select_analysis_abort = None;
                        log::info!("[ws-event] Enriched TeamAnalysisData has been cached.");
                        let _ = app.emit("team-analysis-data", &enriched_data);
                        drop(cache);
                    }
                    Err(error) => {
                        let mut cache = cache_for_task.write().await;
                        if cache.can_commit_champ_select_analysis(generation) {
                            cache.champ_select_analysis_abort = None;
                            cache.champ_select_analysis_key = None;
                        }
                        drop(cache);

                        log::error!("[ws-event] Failed to generate enriched team analysis data: {}", error);
                        if let Some(source) = error.source() {
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
            let _ = self.app.emit("champ-select-session-changed", &None::<Value>);
        }
        Ok(())
    }
}
