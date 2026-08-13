use super::WsEventHandler;
use crate::shared::Result;

use super::super::decoder::{decode_json_api_event, JsonApiEvent};

impl WsEventHandler {
    /// Feeds an HTTP fallback snapshot through the same reducer used by WebSocket events.
    pub(crate) async fn handle_snapshot(&self, uri: &str, data: serde_json::Value) -> Result<()> {
        self.dispatch_json_api_event(&JsonApiEvent {
            uri: uri.to_owned(),
            event_type: "Update".to_owned(),
            data,
        })
        .await
    }

    /// Handles a raw WebSocket event message.
    pub async fn handle_event(&self, event_data: &str) -> Result<()> {
        if event_data.trim().is_empty() {
            return Err("Received empty event data".into());
        }

        let event = decode_json_api_event(event_data).map_err(|error| {
            format!(
                "Failed to parse WebSocket event (length: {}, first 100 chars: '{}'): {}",
                event_data.len(),
                event_data.chars().take(100).collect::<String>(),
                error
            )
        })?;

        if let Some(event) = event {
            if Self::is_important_event(&event.uri) {
                self.dispatch_json_api_event(&event).await?;
            }
        }

        Ok(())
    }

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
                log::trace!("Unhandled event URI: {}", event.uri);
            }
        }

        Ok(())
    }
}
