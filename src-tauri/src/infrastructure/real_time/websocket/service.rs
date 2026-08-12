use super::event_handler::WsEventHandler;
use super::transport;
use crate::infrastructure::game_session::auth::service::{ensure_valid_auth_info_async, invalidate_auth_info};
use once_cell::sync::{Lazy, OnceCell};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{watch, Mutex};
use tokio::task::JoinHandle;

static WS_RUNNING: AtomicBool = AtomicBool::new(false);
static WS_CONNECTED: AtomicBool = AtomicBool::new(false);
static WS_CONNECTION_STATE: Lazy<watch::Sender<bool>> = Lazy::new(|| {
    let (sender, _) = watch::channel(false);
    sender
});
static WS_EVENT_HANDLER: OnceCell<Arc<WsEventHandler>> = OnceCell::new();
static WS_SUPERVISOR: Lazy<Mutex<WsSupervisorState>> = Lazy::new(|| Mutex::new(WsSupervisorState::default()));

#[derive(Default)]
struct WsSupervisorState {
    cancel: Option<watch::Sender<bool>>,
    task: Option<JoinHandle<()>>,
}

impl WsSupervisorState {
    fn can_start(&self) -> bool {
        self.task.as_ref().map_or(true, JoinHandle::is_finished)
    }
}

struct RunningFlagGuard;

impl Drop for RunningFlagGuard {
    fn drop(&mut self) {
        set_ws_connected(false);
        WS_RUNNING.store(false, Ordering::SeqCst);
    }
}

pub(super) fn set_ws_connected(connected: bool) {
    let previous = WS_CONNECTED.swap(connected, Ordering::SeqCst);
    if previous != connected {
        WS_CONNECTION_STATE.send_replace(connected);
    }
}

/// Subscribes to transport lifecycle changes so the connection domain can react immediately
/// instead of discovering a closed socket through a delayed polling cycle.
pub fn subscribe_ws_connection_state() -> watch::Receiver<bool> {
    WS_CONNECTION_STATE.subscribe()
}

fn get_or_init_shared<T>(cell: &OnceCell<Arc<T>>, init: impl FnOnce() -> T) -> Arc<T> {
    cell.get_or_init(|| Arc::new(init())).clone()
}

fn shared_event_handler(app: tauri::AppHandle) -> Arc<WsEventHandler> {
    get_or_init_shared(&WS_EVENT_HANDLER, || WsEventHandler::new(app))
}

/// Gets the process-long event handler and its cache.
pub fn get_event_handler() -> Option<Arc<WsEventHandler>> {
    WS_EVENT_HANDLER.get().cloned()
}

async fn cancel_and_join(state: &mut WsSupervisorState) {
    if let Some(cancel) = state.cancel.take() {
        let _ = cancel.send(true);
    }

    if let Some(mut task) = state.task.take() {
        match tokio::time::timeout(Duration::from_secs(3), &mut task).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) => log::warn!("[lcu-ws] Supervisor join failed: {}", error),
            Err(_) => {
                log::warn!("[lcu-ws] Supervisor did not stop in time; aborting it.");
                task.abort();
                let _ = task.await;
            }
        }
    }
}

/// Starts one idempotent, process-long supervisor. Returns true only for a new generation.
pub async fn start_ws(app: tauri::AppHandle) -> bool {
    let handler = shared_event_handler(app);
    let mut state = WS_SUPERVISOR.lock().await;
    if !state.can_start() {
        return false;
    }

    if let Some(task) = state.task.take() {
        if let Err(error) = task.await {
            log::warn!("[lcu-ws] Previous supervisor join failed: {}", error);
        }
    }
    state.cancel = None;

    let (cancel_tx, cancel_rx) = watch::channel(false);
    WS_RUNNING.store(true, Ordering::SeqCst);
    state.task = Some(tokio::spawn(run_supervisor(handler, cancel_rx)));
    state.cancel = Some(cancel_tx);
    true
}

async fn run_supervisor(handler: Arc<WsEventHandler>, mut cancel: watch::Receiver<bool>) {
    let _running_guard = RunningFlagGuard;
    let mut discovery_failures = 0_u32;

    while !transport::is_cancelled(&cancel) {
        let auth = tokio::select! {
            _ = transport::cancelled(&mut cancel) => break,
            auth = ensure_valid_auth_info_async() => auth,
        };
        let Some(auth) = auth else {
            discovery_failures = discovery_failures.saturating_add(1);
            let delay = discovery_backoff(discovery_failures);
            log::debug!(
                "[lcu-ws] Waiting for LCU authentication info; retrying in {} seconds.",
                delay.as_secs()
            );
            if transport::sleep_or_cancel(delay, &mut cancel).await {
                break;
            }
            continue;
        };
        discovery_failures = 0;

        log::info!("[lcu-ws] Auth info obtained, attempting to connect...");
        let result = transport::connect_and_run_ws(&auth, handler.clone(), &mut cancel).await;
        handler.handle_transport_disconnected().await;
        // A transport generation ended. The same LCU process may still yield identical
        // credentials, but rediscovery is now the single authoritative way to distinguish it
        // from a restarted process with a new port/token.
        invalidate_auth_info();

        if transport::is_cancelled(&cancel) {
            break;
        }
        match result {
            Ok(()) => log::info!("[lcu-ws] Connection closed; reconnecting in 3 seconds."),
            Err(error) => log::warn!("[lcu-ws] Connection failed: {}; retrying in 3 seconds.", error),
        }
        if transport::sleep_or_cancel(Duration::from_secs(3), &mut cancel).await {
            break;
        }
    }

    handler.handle_transport_disconnected().await;
    log::info!("[lcu-ws] Supervisor stopped.");
}

fn discovery_backoff(failures: u32) -> Duration {
    match failures {
        0..=2 => Duration::from_secs(2),
        3..=5 => Duration::from_secs(5),
        6..=8 => Duration::from_secs(15),
        _ => Duration::from_secs(30),
    }
}

/// Replays the process-long reducer cache for a frontend that just registered listeners.
pub async fn sync_snapshot() -> Result<(), String> {
    let handler = get_event_handler().ok_or_else(|| "LCU WebSocket has not been started".to_string())?;
    handler.replay_cached_state().await;
    Ok(())
}

/// Cancels and joins the active supervisor. Calling stop repeatedly is safe.
pub async fn stop_ws() {
    let mut state = WS_SUPERVISOR.lock().await;
    cancel_and_join(&mut state).await;
    if let Some(handler) = WS_EVENT_HANDLER.get() {
        // Also covers the hard-abort path where the supervisor could not reach its own cleanup.
        handler.handle_transport_disconnected().await;
    }
    set_ws_connected(false);
    WS_RUNNING.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::{cancel_and_join, discovery_backoff, get_or_init_shared, WsSupervisorState};
    use once_cell::sync::OnceCell;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::watch;

    #[test]
    fn discovery_backoff_grows_and_caps() {
        assert_eq!(discovery_backoff(1), Duration::from_secs(2));
        assert_eq!(discovery_backoff(3), Duration::from_secs(5));
        assert_eq!(discovery_backoff(6), Duration::from_secs(15));
        assert_eq!(discovery_backoff(9), Duration::from_secs(30));
        assert_eq!(discovery_backoff(99), Duration::from_secs(30));
    }

    #[test]
    fn shared_handler_seam_reuses_identity() {
        let cell = OnceCell::new();
        let first = get_or_init_shared(&cell, || 1usize);
        let second = get_or_init_shared(&cell, || 2usize);
        assert!(Arc::ptr_eq(&first, &second));
        assert_eq!(*second, 1);
    }

    fn install_cancellable_task(state: &mut WsSupervisorState, stopped: Arc<AtomicBool>) {
        let (cancel_tx, mut cancel_rx) = watch::channel(false);
        state.cancel = Some(cancel_tx);
        state.task = Some(tokio::spawn(async move {
            while !*cancel_rx.borrow() {
                if cancel_rx.changed().await.is_err() {
                    break;
                }
            }
            stopped.store(true, Ordering::SeqCst);
        }));
    }

    #[tokio::test]
    async fn supervisor_state_supports_idempotent_start_stop_restart() {
        let first_stopped = Arc::new(AtomicBool::new(false));
        let mut state = WsSupervisorState::default();
        assert!(state.can_start());
        install_cancellable_task(&mut state, first_stopped.clone());
        assert!(!state.can_start(), "a repeated start must reuse the live supervisor");

        cancel_and_join(&mut state).await;
        assert!(first_stopped.load(Ordering::SeqCst));
        assert!(state.can_start());

        let restarted_stopped = Arc::new(AtomicBool::new(false));
        install_cancellable_task(&mut state, restarted_stopped.clone());
        assert!(!state.can_start());
        cancel_and_join(&mut state).await;
        assert!(restarted_stopped.load(Ordering::SeqCst));

        // A repeated stop remains harmless after the restarted task was joined.
        cancel_and_join(&mut state).await;
        assert!(state.can_start());
    }
}
