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
