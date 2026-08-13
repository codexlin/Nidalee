use super::event_handler::WsEventHandler;
use serde_json::Value;
use std::collections::HashSet;
use std::time::Duration;

pub(super) const PHASE_URI: &str = "/lol-gameflow/v1/gameflow-phase";

#[derive(Debug)]
pub(super) struct SnapshotEntry {
    pub uri: &'static str,
    pub data: Value,
}

#[derive(Debug, Default)]
pub(super) struct SnapshotBatch {
    pub phase: Option<String>,
    pub entries: Vec<SnapshotEntry>,
}

impl SnapshotBatch {
    #[cfg(test)]
    pub(super) fn with_phase(phase: &str) -> Self {
        Self {
            phase: Some(phase.to_string()),
            entries: vec![SnapshotEntry {
                uri: PHASE_URI,
                data: Value::String(phase.to_string()),
            }],
        }
    }
}

/// Fetches a bounded authoritative snapshot without touching the event cache. The caller can
/// continue draining the WebSocket and merge events that arrived in the meantime.
pub(super) async fn fetch_snapshot(phase_hint: Option<String>) -> Result<SnapshotBatch, String> {
    tokio::time::timeout(Duration::from_secs(8), fetch_snapshot_inner(phase_hint))
        .await
        .map_err(|_| "LCU snapshot timed out after 8 seconds".to_string())
}

/// Lightweight health check used only after a long period without WebSocket traffic. It does
/// not fetch session/lobby/champ-select payloads; a differing phase asks the transport to run a
/// full snapshot separately.
pub(super) async fn fetch_phase_snapshot() -> Result<SnapshotBatch, String> {
    tokio::time::timeout(Duration::from_secs(5), async {
        let phase = crate::infrastructure::game_session::gameflow::service::get_gameflow_phase(
            crate::http_client::get_lcu_client(),
        )
        .await?;
        Ok::<SnapshotBatch, String>(SnapshotBatch {
            phase: Some(phase.clone()),
            entries: vec![SnapshotEntry {
                uri: PHASE_URI,
                data: Value::String(phase),
            }],
        })
    })
    .await
    .map_err(|_| "LCU phase health check timed out after 5 seconds".to_string())?
}

async fn fetch_snapshot_inner(phase_hint: Option<String>) -> SnapshotBatch {
    let client = crate::http_client::get_lcu_client();
    let (phase_result, session_result, summoner_result) = tokio::join!(
        crate::infrastructure::game_session::gameflow::service::get_gameflow_phase(client),
        crate::infrastructure::game_session::gameflow::service::get_gameflow_session(client),
        crate::shared::utils::lcu_get::<crate::shared::types::SummonerInfo>(
            client,
            "/lol-summoner/v1/current-summoner",
        ),
    );

    let mut batch = SnapshotBatch::default();
    match phase_result {
        Ok(phase) => {
            batch.phase = Some(phase.clone());
            batch.entries.push(SnapshotEntry {
                uri: PHASE_URI,
                data: Value::String(phase),
            });
        }
        Err(error) => log::debug!("Fallback phase fetch failed: {}", error),
    }
    match session_result {
        Ok(data) => batch.entries.push(SnapshotEntry {
            uri: "/lol-gameflow/v1/session",
            data,
        }),
        Err(error) => log::debug!("Fallback session fetch failed: {}", error),
    }
    match summoner_result.and_then(|summoner| serde_json::to_value(summoner).map_err(|error| error.to_string())) {
        Ok(data) => batch.entries.push(SnapshotEntry {
            uri: "/lol-summoner/v1/current-summoner",
            data,
        }),
        Err(error) => log::debug!("Fallback current summoner fetch failed: {}", error),
    }

    let effective_phase = batch.phase.clone().or(phase_hint);
    if batch.phase.is_none() {
        // Record the phase used to select phase-specific requests. If WS advances while these
        // requests run, merge logic can reject the stale batch even when phase fetching failed.
        batch.phase = effective_phase.clone();
    }

    match effective_phase.as_deref() {
        Some("Lobby") | Some("Matchmaking") | Some("None") => {
            let (lobby_result, matchmaking_result) = tokio::join!(
                crate::infrastructure::game_session::lobby::service::get_lobby_info(client),
                crate::infrastructure::match_management::matchmaking::service::get_matchmaking_state(client),
            );

            match lobby_result.and_then(|lobby| serde_json::to_value(lobby).map_err(|error| error.to_string())) {
                Ok(data) => batch.entries.push(SnapshotEntry {
                    uri: "/lol-lobby/v2/lobby",
                    data,
                }),
                Err(error) => log::debug!("Fallback lobby fetch failed: {}", error),
            }
            match matchmaking_result.and_then(|state| serde_json::to_value(state).map_err(|error| error.to_string())) {
                Ok(data) => batch.entries.push(SnapshotEntry {
                    uri: "/lol-matchmaking/v1/search",
                    data,
                }),
                Err(error) => {
                    log::debug!("Fallback matchmaking fetch failed: {}", error)
                }
            }
        }
        Some("ChampSelect") => {
            match crate::infrastructure::champion_selection::champ_select::service::get_champ_select_session_raw(client)
                .await
            {
                Ok(data) => batch.entries.push(SnapshotEntry {
                    uri: "/lol-champ-select/v1/session",
                    data,
                }),
                Err(error) => {
                    log::debug!("Fallback champ-select fetch failed: {}", error)
                }
            }
        }
        _ => {}
    }

    batch
}

pub(super) async fn apply_snapshot(handler: &WsEventHandler, batch: SnapshotBatch, skipped_uris: &HashSet<String>) {
    for entry in batch.entries {
        if skipped_uris.contains(entry.uri) {
            continue;
        }

        if let Err(error) = handler.handle_snapshot(entry.uri, entry.data).await {
            log::warn!("Failed to reduce HTTP snapshot for {}: {}", entry.uri, error);
        }
    }
}
