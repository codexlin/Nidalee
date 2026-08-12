use crate::infrastructure::real_time::websocket::service::subscribe_ws_connection_state;
use crate::shared::types::{ConnectionState, LcuAuthInfo};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/ConnectionInfo.ts")]
pub struct ConnectionInfo {
    pub state: ConnectionState,
    #[ts(skip)]
    #[serde(skip)]
    pub auth_info: Option<LcuAuthInfo>,
    #[ts(skip)]
    #[serde(skip)]
    pub last_successful_connection: Option<Instant>,
    pub consecutive_failures: u32,
    pub error_message: Option<String>,
}

impl Default for ConnectionInfo {
    fn default() -> Self {
        Self {
            state: ConnectionState::Disconnected,
            auth_info: None,
            last_successful_connection: None,
            consecutive_failures: 0,
            error_message: None,
        }
    }
}

/// Read model for the WebSocket transport lifecycle.
///
/// Background authentication discovery and reconnection are owned by the WebSocket supervisor.
/// This manager only projects its lifecycle into the command/event contract used by the
/// frontend; it never scans processes or validates credentials on its own.
pub struct ConnectionManager {
    info: Arc<RwLock<ConnectionInfo>>,
    app: AppHandle,
}

impl ConnectionManager {
    pub fn new(app: AppHandle) -> Self {
        Self {
            info: Arc::new(RwLock::new(ConnectionInfo::default())),
            app,
        }
    }

    pub async fn start_monitoring(&self) {
        let info = self.info.clone();
        let app = self.app.clone();
        let mut transport = subscribe_ws_connection_state();

        tokio::spawn(async move {
            let initial_connected = *transport.borrow_and_update();
            project_transport_state(&info, &app, initial_connected).await;

            loop {
                if transport.changed().await.is_err() {
                    log::warn!(target: "connection::service", "WebSocket lifecycle channel closed");
                    break;
                }
                let connected = *transport.borrow_and_update();
                project_transport_state(&info, &app, connected).await;
            }
        });

        log::info!(
            target: "connection::service",
            "Connection state projection started (WebSocket supervisor owns background discovery)"
        );
    }

    /// Returns the latest supervisor-projected state without performing process discovery or
    /// an HTTP validation request.
    pub async fn check_connection_state(&self) -> ConnectionState {
        self.info.read().await.state.clone()
    }
}

async fn project_transport_state(info: &Arc<RwLock<ConnectionInfo>>, app: &AppHandle, connected: bool) {
    let next_state = if connected {
        ConnectionState::Connected
    } else {
        ConnectionState::Disconnected
    };

    let snapshot = {
        let mut current = info.write().await;
        if current.state == next_state {
            return;
        }

        let previous = current.state.clone();
        current.state = next_state.clone();
        current.auth_info = None;
        current.consecutive_failures = 0;
        if connected {
            current.last_successful_connection = Some(Instant::now());
            current.error_message = None;
        } else {
            current.last_successful_connection = None;
            current.error_message = Some("客户端连接已断开".to_string());
        }

        log::info!(
            target: "connection::service",
            "Connection state changed: {:?} -> {:?}",
            previous,
            next_state
        );
        current.clone()
    };

    if let Err(error) = app.emit("connection-state-changed", snapshot) {
        log::warn!(target: "connection::service", "Failed to emit connection state: {error}");
    }
}
