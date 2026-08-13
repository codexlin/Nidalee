use super::WsEventHandler;
use crate::shared::types::{LobbyInfo, MatchmakingState, SummonerInfo};
use crate::shared::Result;
use serde_json::Value;
use tauri::Emitter;

impl WsEventHandler {
    pub(super) async fn handle_lobby_change(&self, data: &Value, event_type: &str) -> Result<()> {
        if event_type == "Create" || event_type == "Update" {
            if let Ok(lobby) = serde_json::from_value::<LobbyInfo>(data.clone()) {
                let mut cache = self.cache.write().await;
                if cache.lobby_info.as_ref() != Some(&lobby) {
                    log::info!("Lobby info updated ({}).", event_type);
                    cache.lobby_info = Some(lobby.clone());
                    let _ = self.app.emit("lobby-change", &Some(lobby));
                } else {
                    log::debug!("Lobby info unchanged, skipping broadcast.");
                }
            } else {
                log::warn!("Failed to parse lobby info, skipping.");
            }
        } else if event_type == "Delete" {
            let mut cache = self.cache.write().await;
            if cache.lobby_info.is_some() {
                log::info!("Left lobby.");
                cache.lobby_info = None;
                let _ = self.app.emit("lobby-change", &None::<LobbyInfo>);
            }
        }
        Ok(())
    }

    pub(super) async fn handle_current_summoner_change(&self, data: &Value, event_type: &str) -> Result<()> {
        log::info!(
            target: "ws::event",
            "Received current-summoner event type={} (subscription capability probe)",
            event_type
        );

        if event_type == "Create" || event_type == "Update" {
            if let Ok(summoner) = serde_json::from_value::<SummonerInfo>(data.clone()) {
                log::info!(
                    target: "ws::event",
                    "Current-summoner subscription is available summoner={} level={}",
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
                    target: "ws::event",
                    "Failed to parse current-summoner WebSocket payload"
                );
            }
        } else if event_type == "Delete" {
            log::info!(target: "ws::event", "Current-summoner data cleared");
            let mut cache = self.cache.write().await;
            if cache.current_summoner.take().is_some() {
                let _ = self.app.emit("summoner-change", &None::<SummonerInfo>);
            }
        }

        Ok(())
    }

    pub(super) async fn handle_matchmaking_change(&self, data: &Value, event_type: &str) -> Result<()> {
        if event_type == "Create" || event_type == "Update" {
            if let Ok(matchmaking_state) = serde_json::from_value::<MatchmakingState>(data.clone()) {
                let mut cache = self.cache.write().await;
                if cache.matchmaking_state.as_ref() != Some(&matchmaking_state) {
                    log::info!(
                        "Matchmaking state updated ({}) to: {:?}",
                        event_type,
                        matchmaking_state.search_state
                    );
                    cache.matchmaking_state = Some(matchmaking_state.clone());
                    let _ = self.app.emit("matchmaking-state-changed", matchmaking_state);
                } else {
                    log::debug!(
                        "Matchmaking state unchanged, skipping broadcast: {:?}",
                        matchmaking_state.search_state
                    );
                }
            }
        } else if event_type == "Delete" {
            let mut cache = self.cache.write().await;
            if cache.matchmaking_state.is_some() {
                log::info!("Matchmaking state cleared.");
                cache.matchmaking_state = None;
                let _ = self.app.emit("matchmaking-state-changed", &None::<MatchmakingState>);
            }
        }
        Ok(())
    }
}
