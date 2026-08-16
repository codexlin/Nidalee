use crate::infrastructure::match_management::analysis_data::service::MatchStatsCache;
use crate::shared::types::{LobbyInfo, MatchmakingState, SummonerInfo, TeamAnalysisData};
use serde_json::Value;

/// Reducer-owned state shared across WebSocket reconnects.
///
/// Network work must never hold the lock containing this value. Generation counters and abort
/// handles make late async results unable to overwrite a newer game/session.
#[derive(Default)]
pub(super) struct EventCache {
    pub(super) gameflow_phase: Option<String>,
    pub(super) gameflow_session: Option<Value>,
    pub(super) current_summoner: Option<SummonerInfo>,
    pub(super) champ_select_session: Option<Value>,
    pub(super) matchmaking_state: Option<MatchmakingState>,
    pub(super) lobby_info: Option<LobbyInfo>,
    pub(super) match_stats_cache: MatchStatsCache,
    pub(super) team_analysis_data: Option<TeamAnalysisData>,
    pub(super) champ_select_analysis_key: Option<String>,
    pub(super) champ_select_analysis_generation: u64,
    pub(super) champ_select_analysis_abort: Option<tokio::task::AbortHandle>,
    pub(super) in_game_recovery_generation: u64,
    pub(super) in_game_recovery_abort: Option<tokio::task::AbortHandle>,
}

impl EventCache {
    pub(super) fn cancel_champ_select_analysis(&mut self) {
        self.champ_select_analysis_generation = self.champ_select_analysis_generation.wrapping_add(1);
        if let Some(task) = self.champ_select_analysis_abort.take() {
            task.abort();
        }
        self.champ_select_analysis_key = None;
    }

    pub(super) fn can_commit_champ_select_analysis(&self, generation: u64) -> bool {
        self.champ_select_analysis_generation == generation && self.gameflow_phase.as_deref() == Some("ChampSelect")
    }

    pub(super) fn cancel_in_game_recovery(&mut self) {
        self.in_game_recovery_generation = self.in_game_recovery_generation.wrapping_add(1);
        if let Some(task) = self.in_game_recovery_abort.take() {
            task.abort();
        }
    }

    pub(super) fn can_commit_in_game_recovery(&self, generation: u64) -> bool {
        self.in_game_recovery_generation == generation && self.gameflow_phase.as_deref() == Some("InProgress")
    }
}
