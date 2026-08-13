use super::*;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::Notify;

static FLIGHT_TEST_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

async fn install_hook(result: RefreshResult) -> (Arc<TestRefreshHook>, tokio::sync::mpsc::UnboundedReceiver<()>) {
    let (started_tx, started_rx) = tokio::sync::mpsc::unbounded_channel();
    let release = Arc::new(Notify::new());
    let hook = Arc::new(TestRefreshHook {
        calls: std::sync::atomic::AtomicUsize::new(0),
        started: started_tx,
        release: release.clone(),
        result,
    });
    *TEST_REFRESH_HOOK.lock().await = Some(hook.clone());
    (hook, started_rx)
}

async fn clear_hook_and_flight() {
    *TEST_REFRESH_HOOK.lock().await = None;
    *REFRESH_FLIGHT.lock().await = RefreshFlight::Idle;
}

#[tokio::test]
async fn concurrent_waiters_share_single_refresh() {
    let _guard = FLIGHT_TEST_LOCK.lock().await;
    clear_hook_and_flight().await;

    let (hook, mut started_rx) = install_hook(Ok(true)).await;

    let a = tokio::spawn(refresh_static_catalogs_if_stale());
    started_rx.recv().await.expect("coordinator started");

    let b = tokio::spawn(refresh_static_catalogs_if_stale());
    let c = tokio::spawn(refresh_static_catalogs_if_stale());
    tokio::task::yield_now().await;

    assert_eq!(hook.calls.load(Ordering::SeqCst), 1);
    hook.release.notify_waiters();

    assert_eq!(a.await.unwrap(), Ok(true));
    assert_eq!(b.await.unwrap(), Ok(true));
    assert_eq!(c.await.unwrap(), Ok(true));
    assert_eq!(hook.calls.load(Ordering::SeqCst), 1);

    clear_hook_and_flight().await;
}

#[tokio::test]
async fn cancelling_first_caller_does_not_stick_flight() {
    let _guard = FLIGHT_TEST_LOCK.lock().await;
    clear_hook_and_flight().await;

    let (hook, mut started_rx) = install_hook(Ok(true)).await;

    let first = tokio::spawn(refresh_static_catalogs_if_stale());
    started_rx.recv().await.expect("coordinator started");
    first.abort();
    let _ = first.await;

    let second = tokio::spawn(refresh_static_catalogs_if_stale());
    tokio::task::yield_now().await;
    assert_eq!(
        hook.calls.load(Ordering::SeqCst),
        1,
        "cancelled caller must not start a second refresh"
    );

    hook.release.notify_waiters();
    assert_eq!(second.await.unwrap(), Ok(true));

    // flight 已恢复 Idle：下一次刷新应再次进入 hook
    let (hook2, mut started_rx2) = install_hook(Ok(false)).await;
    let third = tokio::spawn(refresh_static_catalogs_if_stale());
    started_rx2.recv().await.expect("second flight started");
    hook2.release.notify_waiters();
    assert_eq!(third.await.unwrap(), Ok(false));
    assert_eq!(hook2.calls.load(Ordering::SeqCst), 1);

    clear_hook_and_flight().await;
}
