use super::event_handler::WsEventHandler;
use super::fallback::{self, SnapshotBatch, PHASE_URI};
use super::service::set_ws_connected;
use crate::shared::types::LcuAuthInfo;
use crate::shared::{NidaleeError, Result};
use base64::{engine::general_purpose, Engine};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{Error as TungsteniteError, Message};
use tokio_tungstenite::{connect_async_tls_with_config, MaybeTlsStream, WebSocketStream};

type LcuWebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const PHASE_HEALTH_INTERVAL: Duration = Duration::from_secs(90);
const SESSION_URI: &str = "/lol-gameflow/v1/session";
const SUMMONER_URI: &str = "/lol-summoner/v1/current-summoner";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SnapshotKind {
    Full,
    PhaseHealth,
}

pub(super) enum IncomingMessage {
    Text(String),
    Ping(Vec<u8>),
    Closed,
    Ignore,
}

enum SendOutcome {
    Sent,
    Cancelled,
    Failed(TungsteniteError),
    TimedOut,
}

struct ConnectedFlagGuard;

impl ConnectedFlagGuard {
    fn new() -> Self {
        set_ws_connected(true);
        Self
    }
}

impl Drop for ConnectedFlagGuard {
    fn drop(&mut self) {
        set_ws_connected(false);
    }
}

#[derive(Default)]
struct SnapshotMergeState {
    observed_uris: HashSet<String>,
    observed_phase: Option<String>,
}

impl SnapshotMergeState {
    fn observe_text(&mut self, text: &str) {
        let Some((uri, phase)) = event_metadata(text) else {
            return;
        };
        self.observed_uris.insert(uri);
        if phase.is_some() {
            self.observed_phase = phase;
        }
    }

    fn skipped_uris(&self, snapshot: &SnapshotBatch) -> HashSet<String> {
        let phase_conflicts = self
            .observed_phase
            .as_deref()
            .is_some_and(|observed| snapshot.phase.as_deref().map_or(true, |fetched| observed != fetched));

        if phase_conflicts {
            snapshot.entries.iter().map(|entry| entry.uri.to_string()).collect()
        } else {
            self.observed_uris.clone()
        }
    }
}

fn select_effective_phase(
    observed_phase: Option<String>,
    fetched_phase: Option<String>,
    previous_phase: Option<String>,
) -> Option<String> {
    observed_phase.or(fetched_phase).or(previous_phase)
}

fn phase_health_requires_full_snapshot(
    observed_ws_phase: Option<&str>,
    current_phase: Option<&str>,
    fetched_phase: Option<&str>,
) -> bool {
    // A phase event received while the HTTP check was in flight is newer and proves the WS
    // stream is alive. Never let the check trigger recovery over it.
    if observed_ws_phase.is_some() {
        return false;
    }

    fetched_phase.is_some() && fetched_phase != current_phase
}

pub(super) fn classify_incoming(
    incoming: Option<std::result::Result<Message, TungsteniteError>>,
) -> std::result::Result<IncomingMessage, TungsteniteError> {
    match incoming {
        None | Some(Ok(Message::Close(_))) => Ok(IncomingMessage::Closed),
        Some(Ok(Message::Text(text))) if text.trim().is_empty() => Ok(IncomingMessage::Ignore),
        Some(Ok(Message::Text(text))) => Ok(IncomingMessage::Text(text)),
        Some(Ok(Message::Ping(payload))) => Ok(IncomingMessage::Ping(payload)),
        Some(Ok(_)) => Ok(IncomingMessage::Ignore),
        Some(Err(error)) => Err(error),
    }
}

fn event_metadata(text: &str) -> Option<(String, Option<String>)> {
    let event = serde_json::from_str::<Value>(text).ok()?;
    let event = event.as_array()?;
    if event.len() < 3 || event[0].as_u64() != Some(8) || event[1].as_str() != Some("OnJsonApiEvent") {
        return None;
    }

    let payload = event[2].as_object()?;
    let uri = payload.get("uri")?.as_str()?.to_string();
    let phase = (uri == PHASE_URI)
        .then(|| payload.get("data").and_then(Value::as_str).map(ToOwned::to_owned))
        .flatten();
    Some((uri, phase))
}

fn phase_from_event(text: &str) -> Option<String> {
    event_metadata(text).and_then(|(uri, phase)| (uri == PHASE_URI).then_some(phase).flatten())
}

fn event_context(text: &str) -> String {
    event_metadata(text)
        .map(|(uri, _)| format!("URI: {uri}"))
        .unwrap_or_else(|| format!("Raw text: {}", text.chars().take(100).collect::<String>()))
}

pub(super) fn is_cancelled(cancel: &watch::Receiver<bool>) -> bool {
    *cancel.borrow()
}

pub(super) async fn cancelled(cancel: &mut watch::Receiver<bool>) {
    loop {
        if is_cancelled(cancel) || cancel.changed().await.is_err() {
            return;
        }
    }
}

pub(super) async fn sleep_or_cancel(duration: Duration, cancel: &mut watch::Receiver<bool>) -> bool {
    tokio::select! {
        _ = tokio::time::sleep(duration) => false,
        _ = cancelled(cancel) => true,
    }
}

async fn send_with_cancel(
    ws_stream: &mut LcuWebSocket,
    message: Message,
    cancel: &mut watch::Receiver<bool>,
) -> SendOutcome {
    tokio::select! {
        biased;
        _ = cancelled(cancel) => SendOutcome::Cancelled,
        result = tokio::time::timeout(Duration::from_secs(2), ws_stream.send(message)) => {
            match result {
                Ok(Ok(())) => SendOutcome::Sent,
                Ok(Err(error)) => SendOutcome::Failed(error),
                Err(_) => SendOutcome::TimedOut,
            }
        }
    }
}

async fn best_effort_close(ws_stream: &mut LcuWebSocket) {
    let _ = tokio::time::timeout(Duration::from_millis(500), ws_stream.send(Message::Close(None))).await;
}

async fn ensure_subscribed(
    ws_stream: &mut LcuWebSocket,
    path: &str,
    subscribed: &mut HashSet<String>,
    cancel: &mut watch::Receiver<bool>,
) -> bool {
    if subscribed.contains(path) {
        return true;
    }

    let message = json!([5, "OnJsonApiEvent", path]).to_string();
    match send_with_cancel(ws_stream, Message::Text(message), cancel).await {
        SendOutcome::Sent => {
            subscribed.insert(path.to_string());
            log::info!("[lcu-ws] Successfully subscribed to: {}", path);
            true
        }
        SendOutcome::Cancelled => false,
        SendOutcome::Failed(error) => {
            log::warn!("[lcu-ws] Failed to subscribe to {}: {}", path, error);
            false
        }
        SendOutcome::TimedOut => {
            log::warn!("[lcu-ws] Timed out subscribing to {}", path);
            false
        }
    }
}

async fn ensure_phase_subscriptions(
    ws_stream: &mut LcuWebSocket,
    phase: &str,
    subscribed: &mut HashSet<String>,
    cancel: &mut watch::Receiver<bool>,
) -> bool {
    match phase {
        "Lobby" | "Matchmaking" | "None" => {
            ensure_subscribed(ws_stream, "/lol-lobby/v2/lobby", subscribed, cancel).await
                && ensure_subscribed(ws_stream, "/lol-matchmaking/v1/search", subscribed, cancel).await
        }
        "ChampSelect" => ensure_subscribed(ws_stream, "/lol-champ-select/v1/session", subscribed, cancel).await,
        _ => true,
    }
}

pub(super) async fn connect_and_run_ws(
    auth: &LcuAuthInfo,
    event_handler: std::sync::Arc<WsEventHandler>,
    cancel: &mut watch::Receiver<bool>,
) -> Result<()> {
    let url = format!("wss://127.0.0.1:{}/", auth.app_port);
    let auth_string = format!("riot:{}", auth.remoting_auth_token);
    let auth_b64 = general_purpose::STANDARD.encode(auth_string.as_bytes());
    let mut request = url
        .into_client_request()
        .map_err(|error| NidaleeError::LcuWebSocket(format!("Failed to build request: {error}")))?;
    let authorization = format!("Basic {auth_b64}")
        .parse()
        .map_err(|error| NidaleeError::LcuWebSocket(format!("Invalid authorization header: {error}")))?;
    request.headers_mut().insert("Authorization", authorization);

    let tls_connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|error| NidaleeError::LcuWebSocket(format!("Failed to build TLS connector: {error}")))?;
    let connector = tokio_tungstenite::Connector::NativeTls(tls_connector);
    let connect_result = tokio::select! {
        _ = cancelled(cancel) => return Ok(()),
        result = connect_async_tls_with_config(request, None, false, Some(connector)) => result,
    };
    let (mut ws_stream, _) = connect_result
        .map_err(|error| NidaleeError::LcuWebSocket(format!("LCU WebSocket connection failed: {error}")))?;

    log::info!("[lcu-ws] WebSocket connection successful.");
    let _connected_guard = ConnectedFlagGuard::new();
    let mut subscribed = HashSet::new();

    for path in [PHASE_URI, SESSION_URI, SUMMONER_URI] {
        if !ensure_subscribed(&mut ws_stream, path, &mut subscribed, cancel).await {
            best_effort_close(&mut ws_stream).await;
            return Ok(());
        }
    }

    let result = run_session(&mut ws_stream, event_handler, &mut subscribed, cancel).await;
    best_effort_close(&mut ws_stream).await;
    log::info!("[lcu-ws] WebSocket connection closed.");
    result
}

async fn run_session(
    ws_stream: &mut LcuWebSocket,
    event_handler: std::sync::Arc<WsEventHandler>,
    subscribed: &mut HashSet<String>,
    cancel: &mut watch::Receiver<bool>,
) -> Result<()> {
    // Start bootstrap after the base subscriptions, but do not await it here. The select below
    // keeps draining Ping/Close/Text while the bounded HTTP snapshot is in flight.
    let mut snapshot_task = Some(tokio::spawn(fallback::fetch_snapshot(None)));
    let mut snapshot_kind = SnapshotKind::Full;
    let mut snapshot_merge = Some(SnapshotMergeState::default());
    let mut current_phase: Option<String> = None;
    let mut idle = Box::pin(tokio::time::sleep(PHASE_HEALTH_INTERVAL));

    let result = loop {
        tokio::select! {
            biased;
            _ = cancelled(cancel) => break Ok(()),
            incoming = ws_stream.next() => {
                idle.as_mut().reset(tokio::time::Instant::now() + PHASE_HEALTH_INTERVAL);
                match classify_incoming(incoming) {
                    Ok(IncomingMessage::Closed) => {
                        log::info!("[lcu-ws] Server closed the WebSocket stream.");
                        break Ok(());
                    }
                    Ok(IncomingMessage::Ping(payload)) => {
                        match send_with_cancel(ws_stream, Message::Pong(payload), cancel).await {
                            SendOutcome::Sent => {}
                            SendOutcome::Cancelled => break Ok(()),
                            SendOutcome::Failed(error) => break Err(NidaleeError::LcuWebSocket(
                                format!("WebSocket pong failed: {error}"),
                            )),
                            SendOutcome::TimedOut => break Err(NidaleeError::LcuWebSocket(
                                "WebSocket pong timed out".to_string(),
                            )),
                        }
                    }
                    Ok(IncomingMessage::Text(text)) => {
                        if let Some(state) = snapshot_merge.as_mut() {
                            state.observe_text(&text);
                        }
                        if let Some(phase) = phase_from_event(&text) {
                            let changed = current_phase.as_deref() != Some(&phase);
                            current_phase = Some(phase.clone());
                            // Reduce and emit the phase before a potentially backpressured
                            // subscription send. Bootstrap completion repeats this idempotently.
                            let context = event_context(&text);
                            let handled = tokio::select! {
                                _ = cancelled(cancel) => break Ok(()),
                                handled = event_handler.handle_event(&text) => handled,
                            };
                            if let Err(error) = handled {
                                log::warn!("[lcu-ws] Event handling failed for [{}]: {}", context, error);
                            }
                            if changed
                                && !ensure_phase_subscriptions(ws_stream, &phase, subscribed, cancel).await
                            {
                                break Ok(());
                            }
                            continue;
                        }

                        let context = event_context(&text);
                        let handled = tokio::select! {
                            _ = cancelled(cancel) => break Ok(()),
                            handled = event_handler.handle_event(&text) => handled,
                        };
                        if let Err(error) = handled {
                            log::warn!("[lcu-ws] Event handling failed for [{}]: {}", context, error);
                        }
                    }
                    Ok(IncomingMessage::Ignore) => {}
                    Err(error) => break Err(NidaleeError::LcuWebSocket(format!(
                        "WebSocket read error: {error}"
                    ))),
                }
            }
            completed = async {
                match snapshot_task.as_mut() {
                    Some(task) => Some(task.await),
                    None => std::future::pending().await,
                }
            } => {
                snapshot_task = None;
                let completed_kind = snapshot_kind;
                let merge = snapshot_merge.take().unwrap_or_default();
                let observed_phase = merge.observed_phase.clone();

                if completed_kind == SnapshotKind::PhaseHealth {
                    match completed {
                        Some(Ok(Ok(batch))) => {
                            if phase_health_requires_full_snapshot(
                                observed_phase.as_deref(),
                                current_phase.as_deref(),
                                batch.phase.as_deref(),
                            ) {
                                log::info!(
                                    "[lcu-ws] Phase health check diverged ({:?} -> {:?}); running full snapshot.",
                                    current_phase,
                                    batch.phase
                                );
                                let phase_hint = batch.phase.or(current_phase.clone());
                                snapshot_task = Some(tokio::spawn(fallback::fetch_snapshot(phase_hint)));
                                snapshot_kind = SnapshotKind::Full;
                                snapshot_merge = Some(SnapshotMergeState::default());
                            }
                        }
                        Some(Ok(Err(error))) => {
                            log::debug!("[lcu-ws] Phase health check failed: {}", error);
                        }
                        Some(Err(error)) => {
                            log::warn!("[lcu-ws] Phase health check task failed: {}", error);
                        }
                        None => {}
                    }
                    idle.as_mut().reset(tokio::time::Instant::now() + PHASE_HEALTH_INTERVAL);
                    continue;
                }

                match completed {
                    Some(Ok(Ok(batch))) => {
                        let fetched_phase = batch.phase.clone();
                        let skipped = merge.skipped_uris(&batch);
                        fallback::apply_snapshot(&event_handler, batch, &skipped).await;

                        // A WS phase observed during this flight is newest. Otherwise trust this
                        // flight's HTTP phase before the phase that existed when it started.
                        let effective_phase =
                            select_effective_phase(observed_phase, fetched_phase, current_phase.clone());
                        if let Some(phase) = effective_phase {
                            if !ensure_phase_subscriptions(ws_stream, &phase, subscribed, cancel).await {
                                break Ok(());
                            }
                            current_phase = Some(phase);
                        }
                    }
                    Some(Ok(Err(error))) => {
                        log::debug!("[lcu-ws] HTTP snapshot failed: {}", error);
                        if let Some(phase) = observed_phase.or(current_phase.clone()) {
                            if !ensure_phase_subscriptions(ws_stream, &phase, subscribed, cancel).await {
                                break Ok(());
                            }
                            current_phase = Some(phase);
                        }
                    }
                    Some(Err(error)) => {
                        log::warn!("[lcu-ws] HTTP snapshot task failed: {}", error);
                        if let Some(phase) = observed_phase.or(current_phase.clone()) {
                            if !ensure_phase_subscriptions(ws_stream, &phase, subscribed, cancel).await {
                                break Ok(());
                            }
                        }
                    }
                    None => {}
                }
                idle.as_mut().reset(tokio::time::Instant::now() + PHASE_HEALTH_INTERVAL);
            }
            _ = &mut idle, if snapshot_task.is_none() => {
                log::debug!("[lcu-ws] No WS events for 90s, checking gameflow phase.");
                snapshot_task = Some(tokio::spawn(fallback::fetch_phase_snapshot()));
                snapshot_kind = SnapshotKind::PhaseHealth;
                snapshot_merge = Some(SnapshotMergeState::default());
                idle.as_mut().reset(tokio::time::Instant::now() + PHASE_HEALTH_INTERVAL);
            }
        }
    };

    if let Some(task) = snapshot_task.take() {
        task.abort();
        let _ = task.await;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        classify_incoming, phase_health_requires_full_snapshot, select_effective_phase, IncomingMessage,
        SnapshotMergeState,
    };
    use crate::infrastructure::real_time::websocket::fallback::{SnapshotBatch, SnapshotEntry, PHASE_URI};
    use serde_json::Value;
    use tokio_tungstenite::tungstenite::Message;

    #[test]
    fn phase_health_only_recovers_without_newer_ws_phase() {
        assert!(phase_health_requires_full_snapshot(None, Some("None"), Some("Lobby")));
        assert!(!phase_health_requires_full_snapshot(None, Some("Lobby"), Some("Lobby")));
        assert!(!phase_health_requires_full_snapshot(
            Some("ChampSelect"),
            Some("Lobby"),
            Some("None")
        ));
    }

    #[test]
    fn eof_and_close_are_both_terminal() {
        assert!(matches!(classify_incoming(None), Ok(IncomingMessage::Closed)));
        assert!(matches!(
            classify_incoming(Some(Ok(Message::Close(None)))),
            Ok(IncomingMessage::Closed)
        ));
    }

    #[test]
    fn snapshot_merge_tracks_text_while_ping_and_close_remain_classifiable() {
        let mut merge = SnapshotMergeState::default();
        let text = r#"[8,"OnJsonApiEvent",{"uri":"/lol-gameflow/v1/gameflow-phase","eventType":"Update","data":"ChampSelect"}]"#;
        match classify_incoming(Some(Ok(Message::Text(text.to_string())))).unwrap() {
            IncomingMessage::Text(text) => merge.observe_text(&text),
            _ => panic!("expected text"),
        }
        assert!(matches!(
            classify_incoming(Some(Ok(Message::Ping(vec![1, 2])))),
            Ok(IncomingMessage::Ping(_))
        ));

        let skipped = merge.skipped_uris(&SnapshotBatch::with_phase("Lobby"));
        assert!(skipped.contains(PHASE_URI));
        assert_eq!(merge.observed_phase.as_deref(), Some("ChampSelect"));
    }

    #[test]
    fn same_phase_snapshot_skips_only_uris_already_seen_on_ws() {
        let mut merge = SnapshotMergeState::default();
        merge.observe_text(
            r#"[8,"OnJsonApiEvent",{"uri":"/lol-gameflow/v1/gameflow-phase","eventType":"Update","data":"ChampSelect"}]"#,
        );
        let mut snapshot = SnapshotBatch::with_phase("ChampSelect");
        snapshot.entries.push(SnapshotEntry {
            uri: "/lol-gameflow/v1/session",
            data: Value::Null,
        });

        let skipped = merge.skipped_uris(&snapshot);

        assert!(skipped.contains(PHASE_URI));
        assert!(!skipped.contains("/lol-gameflow/v1/session"));
    }

    #[test]
    fn observed_ws_phase_rejects_snapshot_when_http_phase_is_unknown() {
        let mut merge = SnapshotMergeState::default();
        merge.observe_text(
            r#"[8,"OnJsonApiEvent",{"uri":"/lol-gameflow/v1/gameflow-phase","eventType":"Update","data":"ChampSelect"}]"#,
        );
        let snapshot = SnapshotBatch {
            phase: None,
            entries: vec![SnapshotEntry {
                uri: "/lol-gameflow/v1/session",
                data: Value::Null,
            }],
        };

        let skipped = merge.skipped_uris(&snapshot);

        assert!(skipped.contains("/lol-gameflow/v1/session"));
    }

    #[test]
    fn effective_phase_prefers_observed_ws_then_http_then_previous_state() {
        assert_eq!(
            select_effective_phase(
                Some("ChampSelect".to_string()),
                Some("Lobby".to_string()),
                Some("None".to_string()),
            )
            .as_deref(),
            Some("ChampSelect")
        );
        assert_eq!(
            select_effective_phase(None, Some("Lobby".to_string()), Some("None".to_string())).as_deref(),
            Some("Lobby")
        );
        assert_eq!(
            select_effective_phase(None, None, Some("None".to_string())).as_deref(),
            Some("None")
        );
    }
}
