use crate::infrastructure::game_session::auth::service::ensure_valid_auth_info;
use crate::infrastructure::real_time::websocket::event_handler::WsEventHandler;
use crate::shared::{NidaleeError, Result};
use base64::{engine::general_purpose, Engine};
use futures_util::{SinkExt, StreamExt};
use once_cell::sync::OnceCell;
use serde_json::json;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Emitter;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async_tls_with_config, tungstenite::client::IntoClientRequest};

static WS_RUNNING: AtomicBool = AtomicBool::new(false);
static WS_TASK: OnceCell<tokio::task::JoinHandle<()>> = OnceCell::new();
static WS_SENDER: OnceCell<
    Arc<
        Mutex<
            Option<tokio_tungstenite::tungstenite::protocol::WebSocket<tokio_tungstenite::MaybeTlsStream<TcpStream>>>,
        >,
    >,
> = OnceCell::new();

// Global instance of the event handler, used to access the cache from commands.
static WS_EVENT_HANDLER: OnceCell<Arc<WsEventHandler>> = OnceCell::new();

/// Gets the global event handler instance.
pub fn get_event_handler() -> Option<Arc<WsEventHandler>> {
    WS_EVENT_HANDLER.get().cloned()
}

// Helper function to ensure a path is subscribed to, idempotently.
async fn ensure_subscribed(
    ws_stream: &mut tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<TcpStream>>,
    path: &str,
    subscribed: &mut std::collections::HashSet<String>,
) {
    use tokio_tungstenite::tungstenite::Message;
    if !subscribed.contains(path) {
        let msg = json!([5, "OnJsonApiEvent", path]).to_string();
        match ws_stream.send(Message::Text(msg)).await {
            Ok(_) => {
                subscribed.insert(path.to_string());
                log::info!("[lcu-ws] Successfully subscribed to: {}", path);
            }
            Err(e) => log::warn!("[lcu-ws] Failed to subscribe to {}: {}", path, e),
        }
    }
}

// When the WebSocket is idle, performs an HTTP fallback fetch to align state.
// phase_hint: A hint of the current game phase to prune which endpoints to fetch, reducing unnecessary 404s.
async fn fallback_fetch_and_emit(app: &tauri::AppHandle, phase_hint: Option<&str>) {
    // Construct an HTTP client with a short timeout.
    let client = match reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            log::warn!("[lcu-ws] Fallback HTTP client creation failed: {}", e);
            return;
        }
    };

    // First, try to get the latest phase. Only send positive data to avoid clearing frontend state on a temporary failure.
    let mut latest_phase: Option<String> = None;
    if let Ok(phase) = crate::infrastructure::game_session::gameflow::service::get_gameflow_phase(&client).await {
        let _ = app.emit("gameflow-phase-change", &Some(phase.clone()));
        latest_phase = Some(phase);
    }

    // Decide whether to fetch other data based on the phase.
    let effective_phase = latest_phase.as_deref().or(phase_hint);
    match effective_phase {
        Some("Lobby") | Some("Matchmaking") | Some("None") => {
            if let Ok(lobby) = crate::infrastructure::game_session::lobby::service::get_lobby_info(&client).await {
                let _ = app.emit("lobby-change", &Some(lobby));
            }
            if let Ok(state) = crate::infrastructure::match_management::matchmaking::service::get_matchmaking_state(&client).await {
                let _ = app.emit("matchmaking-state-changed", state);
            }
        }
        Some("ChampSelect") => {
            if let Ok(session) = crate::infrastructure::champion_selection::champ_select::service::get_champ_select_session(&client).await {
                let _ = app.emit("champ-select-session-changed", &session);
            }
        }
        _ => {
            // For other phases (e.g., None, InProgress), just syncing the phase is sufficient.
        }
    }
}

pub async fn start_ws(app: tauri::AppHandle) {
    if WS_RUNNING.swap(true, Ordering::SeqCst) {
        return;
    }

    let handle = tokio::spawn(async move {
        // Main loop: continuously attempts to connect and reconnect.
        while WS_RUNNING.load(Ordering::SeqCst) {
            // Wait for authentication info to become available.
            let mut retry_count = 0;
            let max_retries = 10;

            while retry_count < max_retries && WS_RUNNING.load(Ordering::SeqCst) {
                if let Some(auth) = ensure_valid_auth_info() {
                    log::info!("[lcu-ws] Auth info obtained, attempting to connect...");
                    if let Err(e) = connect_and_run_ws(&app, &auth).await {
                        log::warn!("[lcu-ws] Connection failed: {}, retrying in 3 seconds.", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    } else {
                        log::info!("[lcu-ws] Connection ended gracefully, will reconnect in 3 seconds.");
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                    }
                } else {
                    retry_count += 1;
                    log::debug!("[lcu-ws] Waiting for auth info... ({}/{})", retry_count, max_retries);
                    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                }
            }

            if retry_count >= max_retries {
                log::warn!("[lcu-ws] Could not get auth info after multiple retries, will try again in 5 seconds.");
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }

        log::info!("[lcu-ws] Main loop has exited.");
        WS_RUNNING.store(false, Ordering::SeqCst);
    });

    let _ = WS_TASK.set(handle);
}

async fn connect_and_run_ws(app: &tauri::AppHandle, auth: &crate::shared::types::LcuAuthInfo) -> Result<()> {
    let url = format!("wss://127.0.0.1:{}/", auth.app_port);
    let auth_string = format!("riot:{}", auth.remoting_auth_token);
    let auth_b64 = general_purpose::STANDARD.encode(auth_string.as_bytes());

    let mut req = url
        .into_client_request()
        .map_err(|e| format!("Failed to build request: {}", e))?;
    req.headers_mut()
        .insert("Authorization", format!("Basic {}", auth_b64).parse().unwrap());

    let tls_connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true) // Allow self-signed certificates for local LCU.
        .build()
        .map_err(|e| format!("Failed to build TLS connector: {}", e))?;
    let connector = tokio_tungstenite::Connector::NativeTls(tls_connector);

    let connect_res = connect_async_tls_with_config(req, None, false, Some(connector)).await;
    let (mut ws_stream, _) = match connect_res {
        Ok(res) => res,
        Err(e) => return Err(format!("LCU WebSocket connection failed: {}", e).into()),
    };

    log::info!("[lcu-ws] WebSocket connection successful.");

    // Dynamic subscription set: start with base subscriptions and add more as the game phase changes.
    let mut subscribed: HashSet<String> = HashSet::new();
    let mut current_phase: Option<String> = None;

    // Base subscriptions: always subscribe to gameflow events.
    ensure_subscribed(&mut ws_stream, "/lol-gameflow/v1/gameflow-phase", &mut subscribed).await;
    ensure_subscribed(&mut ws_stream, "/lol-gameflow/v1/session", &mut subscribed).await;

    // 🧪 实验：尝试订阅召唤师信息变化（测试 LCU 是否支持）
    ensure_subscribed(&mut ws_stream, "/lol-summoner/v1/current-summoner", &mut subscribed).await;

    // Create the event handler.
    let event_handler = Arc::new(WsEventHandler::new(app.clone()));

    // Store in a global variable for access from Tauri commands.
    let _ = WS_EVENT_HANDLER.set(event_handler.clone());

    // Receive loop: process WebSocket events.
    while WS_RUNNING.load(Ordering::SeqCst) {
        // Use a select macro to respond to WS messages and perform fallback fetches during idle periods.
        tokio::select! {
            biased;
            Some(msg_result) = ws_stream.next() => {
                match msg_result {
                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                        // Attempt to parse event for richer logging context.
                        let event_info = if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
                            if let Some(event_array) = data.as_array() {
                                if event_array.len() >= 3 {
                                    if let (Some(8), Some("OnJsonApiEvent"), Some(payload)) = (
                                        event_array[0].as_u64(),
                                        event_array[1].as_str(),
                                        event_array[2].as_object()
                                    ) {
                                        if let Some(uri) = payload.get("uri").and_then(|v| v.as_str()) {
                                            // Dynamic subscription: add more subscriptions based on the current game phase.
                                            if uri == "/lol-gameflow/v1/gameflow-phase" {
                                                if let Some(phase_str) = payload.get("data").and_then(|v| v.as_str()) {
                                                    if current_phase.as_deref() != Some(phase_str) {
                                                        log::info!("[lcu-ws] Phase changed: {:?} -> {} (triggering dynamic subscription).", current_phase, phase_str);
                                                        current_phase = Some(phase_str.to_string());
                                                        match phase_str {
                                                            "Lobby" | "Matchmaking" | "None" => {
                                                                ensure_subscribed(&mut ws_stream, "/lol-lobby/v2/lobby", &mut subscribed).await;
                                                                ensure_subscribed(&mut ws_stream, "/lol-matchmaking/v1/search", &mut subscribed).await;
                                                            }
                                                            "ChampSelect" => {
                                                                ensure_subscribed(&mut ws_stream, "/lol-champ-select/v1/session", &mut subscribed).await;
                                                            }
                                                            _ => {}
                                                        }
                                                    }
                                                }
                                            }
                                            format!("URI: {}", uri)
                                        } else {
                                            "Unknown Event Type".to_string()
                                        }
                                    } else {
                                        format!("Non-API Event: {:?}", event_array)
                                    }
                                } else {
                                    format!("Event array too short: {}", event_array.len())
                                }
                            } else {
                                "Non-array event".to_string()
                            }
                        } else {
                            format!("Raw text: {}", text.chars().take(100).collect::<String>())
                        };

                        // Process the event using the handler.
                        if let Err(e) = event_handler.handle_event(&text).await {
                            log::warn!("[lcu-ws] Event handling failed for [{}]: {}", event_info, e);
                        }
                    }
                    Ok(_) => { /* Ignore non-text messages */ }
                    Err(e) => {
                        log::error!("[lcu-ws] WebSocket read error: {}", e);
                        return Err(format!("WebSocket read error: {}", e).into());
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                // If idle for 10s, perform an HTTP fallback fetch to align state.
                log::debug!("[lcu-ws] No WS events for 10s, performing HTTP fallback fetch.");
                fallback_fetch_and_emit(app, current_phase.as_deref()).await;
            }
        }
    }

    Ok(())
}

pub async fn stop_ws() {
    WS_RUNNING.store(false, Ordering::SeqCst);
}
