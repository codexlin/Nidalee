//! Reduces LCU WebSocket and fallback snapshot events into process-long cached state.

mod basic_state;
mod champ_select;
mod context;
mod dispatch;
mod enrichment;
mod lifecycle;
mod phase_session;

use std::sync::Arc;

use reqwest::Client;
use tauri::AppHandle;
use tokio::sync::RwLock;

use super::state::EventCache;

#[derive(Clone)]
pub struct WsEventHandler {
    app: AppHandle,
    cache: Arc<RwLock<EventCache>>,
    client: Client,
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
}

#[cfg(test)]
mod tests {
    use super::EventCache;

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
