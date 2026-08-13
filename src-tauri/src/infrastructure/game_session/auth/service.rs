use futures_util::future::{BoxFuture, FutureExt, Shared};
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, Weak};

use crate::shared::types::LcuAuthInfo;

use super::process_discovery;

pub static AUTH_INFO: Lazy<RwLock<Option<LcuAuthInfo>>> = Lazy::new(|| RwLock::new(None));
static AUTH_GENERATION: AtomicU64 = AtomicU64::new(0);

type SharedAuthRefresh = Shared<BoxFuture<'static, std::result::Result<LcuAuthInfo, String>>>;

#[derive(Default)]
struct RefreshFlight {
    generation: u64,
    future: Option<SharedAuthRefresh>,
}

impl RefreshFlight {
    fn join_or_start(&mut self, create: impl FnOnce(u64) -> SharedAuthRefresh) -> SharedAuthRefresh {
        if let Some(future) = self.future.as_ref() {
            return future.clone();
        }

        self.generation = self.generation.wrapping_add(1);
        let future = create(self.generation);
        self.future = Some(future.clone());
        future
    }

    fn clear_if_generation(&mut self, generation: u64) {
        if self.generation == generation {
            self.future = None;
        }
    }
}

#[derive(Default)]
struct AuthRefreshCoordinator {
    state: tokio::sync::Mutex<RefreshFlight>,
}

impl AuthRefreshCoordinator {
    async fn clear(&self, generation: u64) {
        self.state.lock().await.clear_if_generation(generation);
    }

    #[cfg(test)]
    async fn get_or_start_with(
        self: &Arc<Self>,
        create: impl FnOnce(Weak<Self>, u64) -> SharedAuthRefresh,
    ) -> SharedAuthRefresh {
        let mut state = self.state.lock().await;
        state.join_or_start(|generation| create(Arc::downgrade(self), generation))
    }
}

static AUTH_REFRESH_COORDINATOR: Lazy<Arc<AuthRefreshCoordinator>> =
    Lazy::new(|| Arc::new(AuthRefreshCoordinator::default()));

fn cached_auth_info() -> Option<LcuAuthInfo> {
    AUTH_INFO.read().ok().and_then(|auth| auth.clone())
}

fn current_auth_info() -> Option<LcuAuthInfo> {
    AUTH_INFO.read().ok().and_then(|auth| auth.clone())
}

fn commit_auth_if_generation(
    discovered: LcuAuthInfo,
    expected_generation: u64,
) -> std::result::Result<LcuAuthInfo, String> {
    let mut auth = AUTH_INFO.write().map_err(|_| "LCU auth cache poisoned".to_string())?;
    if AUTH_GENERATION.load(Ordering::SeqCst) != expected_generation {
        return auth
            .clone()
            .ok_or_else(|| "LCU auth refresh became stale before commit".to_string());
    }

    *auth = Some(discovered.clone());
    AUTH_GENERATION.fetch_add(1, Ordering::SeqCst);
    Ok(discovered)
}

fn invalidate_auth_if_generation(expected_generation: u64) {
    let Ok(mut auth) = AUTH_INFO.write() else {
        return;
    };
    if AUTH_GENERATION.load(Ordering::SeqCst) != expected_generation {
        return;
    }
    *auth = None;
    AUTH_GENERATION.fetch_add(1, Ordering::SeqCst);
}

fn invalidate_rejected_auth(failed_auth: &LcuAuthInfo) {
    let Ok(mut auth) = AUTH_INFO.write() else {
        return;
    };
    if auth.as_ref() != Some(failed_auth) {
        return;
    }
    *auth = None;
    AUTH_GENERATION.fetch_add(1, Ordering::SeqCst);
}

fn create_process_refresh(
    coordinator: Weak<AuthRefreshCoordinator>,
    flight_generation: u64,
    expected_auth_generation: u64,
) -> SharedAuthRefresh {
    let task = tokio::spawn(async move {
        let discovered = match tokio::task::spawn_blocking(process_discovery::discover_auth_info).await {
            Ok(result) => result.map_err(String::from),
            Err(error) => Err(format!("LCU auth discovery worker failed: {error}")),
        };

        let result = match discovered {
            Ok(auth) => commit_auth_if_generation(auth, expected_auth_generation),
            Err(error) => {
                invalidate_auth_if_generation(expected_auth_generation);
                Err(error)
            }
        };

        if let Some(coordinator) = coordinator.upgrade() {
            coordinator.clear(flight_generation).await;
        }
        result
    });

    async move {
        task.await
            .map_err(|error| format!("LCU auth refresh coordinator failed: {error}"))?
    }
    .boxed()
    .shared()
}

async fn refresh_auth_singleflight(rejected_auth: Option<&LcuAuthInfo>) -> std::result::Result<LcuAuthInfo, String> {
    let coordinator = AUTH_REFRESH_COORDINATOR.clone();
    let future = {
        let mut state = coordinator.state.lock().await;

        if let Some(rejected_auth) = rejected_auth {
            if let Some(current) = current_auth_info().filter(|current| current != rejected_auth) {
                return Ok(current);
            }
        } else if let Some(current) = cached_auth_info() {
            return Ok(current);
        }

        if let Some(future) = state.future.as_ref() {
            future.clone()
        } else {
            if let Some(rejected_auth) = rejected_auth {
                invalidate_rejected_auth(rejected_auth);
            }
            let expected_auth_generation = AUTH_GENERATION.load(Ordering::SeqCst);
            state.join_or_start(|flight_generation| {
                create_process_refresh(
                    Arc::downgrade(&coordinator),
                    flight_generation,
                    expected_auth_generation,
                )
            })
        }
    };

    future.await
}

/// Returns credentials for the current LCU process or joins the one process-wide discovery flight.
/// LCU credentials are stable for the process lifetime and are invalidated explicitly on
/// transport/session loss or an HTTP 401/403; they are not refreshed on a wall-clock TTL.
pub async fn ensure_valid_auth_info_async() -> Option<LcuAuthInfo> {
    if let Some(auth) = cached_auth_info() {
        return Some(auth);
    }

    match refresh_auth_singleflight(None).await {
        Ok(auth) => Some(auth),
        Err(error) => {
            log::error!("[LCU] Unable to refresh authentication: {}", error);
            None
        }
    }
}

/// Refreshes credentials after a 401/403 through the same process-wide discovery flight.
pub async fn refresh_auth_info_after_rejection(failed_auth: &LcuAuthInfo) -> std::result::Result<LcuAuthInfo, String> {
    refresh_auth_singleflight(Some(failed_auth)).await
}

pub fn invalidate_auth_info() {
    if let Ok(mut auth) = AUTH_INFO.write() {
        *auth = None;
        AUTH_GENERATION.fetch_add(1, Ordering::SeqCst);
        log::info!("[LCU] AuthInfo 缓存已清除");
    }
}

#[cfg(debug_assertions)]
pub fn verify_lockfile_vs_cmdline() {
    process_discovery::verify_lockfile_vs_cmdline();
}

#[cfg(test)]
mod tests {
    use super::{AuthRefreshCoordinator, SharedAuthRefresh};
    use crate::shared::types::LcuAuthInfo;
    use futures_util::future::FutureExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn test_auth(token: &str) -> LcuAuthInfo {
        LcuAuthInfo {
            app_port: 2999,
            remoting_auth_token: token.to_string(),
            riotclient_app_port: 3000,
            riotclient_auth_token: format!("riot-{token}"),
        }
    }

    #[tokio::test]
    async fn concurrent_callers_join_one_refresh_flight() {
        let coordinator = Arc::new(AuthRefreshCoordinator::default());
        let starts = Arc::new(AtomicUsize::new(0));
        let expected = test_auth("new");
        let mut tasks = Vec::new();

        for _ in 0..16 {
            let coordinator = coordinator.clone();
            let starts = starts.clone();
            let expected = expected.clone();
            tasks.push(tokio::spawn(async move {
                let future: SharedAuthRefresh = coordinator
                    .get_or_start_with(move |_coordinator, _generation| {
                        async move {
                            starts.fetch_add(1, Ordering::SeqCst);
                            tokio::task::yield_now().await;
                            Ok(expected)
                        }
                        .boxed()
                        .shared()
                    })
                    .await;
                future.await
            }));
        }

        for task in tasks {
            assert_eq!(task.await.unwrap().unwrap(), test_auth("new"));
        }
        assert_eq!(starts.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn failed_refresh_flight_can_be_cleared_and_restarted() {
        let coordinator = Arc::new(AuthRefreshCoordinator::default());
        let starts = Arc::new(AtomicUsize::new(0));

        let first = coordinator
            .get_or_start_with({
                let starts = starts.clone();
                move |weak, generation| {
                    async move {
                        starts.fetch_add(1, Ordering::SeqCst);
                        if let Some(coordinator) = weak.upgrade() {
                            coordinator.clear(generation).await;
                        }
                        Err("discovery failed".to_string())
                    }
                    .boxed()
                    .shared()
                }
            })
            .await;
        assert_eq!(first.await, Err("discovery failed".to_string()));

        let second = coordinator
            .get_or_start_with({
                let starts = starts.clone();
                move |weak, generation| {
                    async move {
                        starts.fetch_add(1, Ordering::SeqCst);
                        if let Some(coordinator) = weak.upgrade() {
                            coordinator.clear(generation).await;
                        }
                        Ok(test_auth("recovered"))
                    }
                    .boxed()
                    .shared()
                }
            })
            .await;

        assert_eq!(second.await.unwrap(), test_auth("recovered"));
        assert_eq!(starts.load(Ordering::SeqCst), 2);
    }
}
