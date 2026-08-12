use super::WsEventHandler;
use crate::shared::types::TeamAnalysisData;
use tauri::Emitter;

impl WsEventHandler {
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
        let _ = self.app.emit("team-analysis-data", &None::<TeamAnalysisData>);
    }
}
