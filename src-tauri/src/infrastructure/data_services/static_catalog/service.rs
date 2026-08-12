//! 静态目录编排：版本探测 → 磁盘命中 / 网络拉取 → 可替换 store

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{oneshot, Mutex, OnceCell as TokioOnceCell};
use ts_rs::TS;

use crate::http_client;
use crate::infrastructure::champion_selection::summoner_spells::service::{
    fetch_summoner_spell_data_from_network, install_summoner_spell_maps, is_loaded as spells_loaded, SummonerSpellInfo,
};
use crate::infrastructure::data_services::champion_data::service::{
    fetch_champion_data_from_network, install_champion_maps, is_loaded as champions_loaded, ChampionInfo,
};

static META: Lazy<RwLock<Option<StaticCatalogMeta>>> = Lazy::new(|| RwLock::new(None));
static INIT: TokioOnceCell<()> = TokioOnceCell::const_new();

type RefreshResult = Result<bool, String>;

/// 真正的 singleflight：并发调用共享同一次 refresh 结果
enum RefreshFlight {
    Idle,
    Running {
        waiters: Vec<oneshot::Sender<RefreshResult>>,
    },
}

static REFRESH_FLIGHT: Lazy<Mutex<RefreshFlight>> = Lazy::new(|| Mutex::new(RefreshFlight::Idle));

/// 测试用：替换 `refresh_static_catalogs_inner`，便于验证 singleflight / 取消语义
#[cfg(test)]
static TEST_REFRESH_HOOK: Lazy<Mutex<Option<std::sync::Arc<TestRefreshHook>>>> = Lazy::new(|| Mutex::new(None));

#[cfg(test)]
struct TestRefreshHook {
    calls: std::sync::atomic::AtomicUsize,
    started: tokio::sync::mpsc::UnboundedSender<()>,
    release: std::sync::Arc<tokio::sync::Notify>,
    result: RefreshResult,
}

const CHAMPIONS_FILE: &str = "champions.json";
const SPELLS_FILE: &str = "summoner-spells.json";
const META_FILE: &str = "meta.json";

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/StaticCatalogMeta.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct StaticCatalogMeta {
    pub version: String,
    /// Unix 毫秒（JSON IPC 为 JS number，不是 bigint）
    #[ts(type = "number")]
    pub loaded_at: i64,
    /// `disk` | `network` | `disk-offline` | `disk-stale`
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiskMeta {
    version: String,
    loaded_at: i64,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
static TEST_CACHE_ROOT: Lazy<RwLock<Option<PathBuf>>> = Lazy::new(|| RwLock::new(None));

fn cache_root() -> PathBuf {
    #[cfg(test)]
    {
        if let Ok(guard) = TEST_CACHE_ROOT.read() {
            if let Some(path) = guard.as_ref() {
                return path.clone();
            }
        }
    }
    dirs::data_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join("nidalee")
        .join("static")
}

fn version_dir(version: &str) -> PathBuf {
    cache_root().join(version)
}

fn set_meta(meta: StaticCatalogMeta) {
    match META.write() {
        Ok(mut guard) => *guard = Some(meta),
        Err(e) => log::error!("[StaticCatalog] META 锁中毒，无法写入: {e}"),
    }
}

pub fn get_static_meta() -> Option<StaticCatalogMeta> {
    META.read().ok().and_then(|g| g.clone())
}

/// 探测最新 DDragon 版本；失败返回 None（禁止伪造旧版本号）
pub async fn fetch_ddragon_version() -> Option<String> {
    let client = http_client::get_public_client();
    match client
        .get("https://ddragon.leagueoflegends.com/api/versions.json")
        .send()
        .await
    {
        Ok(response) => {
            if let Ok(versions) = response.json::<Vec<String>>().await {
                if let Some(latest) = versions.first() {
                    return Some(latest.clone());
                }
            }
            log::warn!("[StaticCatalog] DDragon versions.json 无有效条目");
        }
        Err(e) => {
            log::warn!("[StaticCatalog] 获取 DDragon 版本失败: {e}");
        }
    }
    None
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T> {
    let bytes = std::fs::read(path).ok()?;
    serde_json::from_slice(&bytes).ok()
}

/// 跨平台替换：Windows 上目标存在时 `rename(tmp→path)` 会失败，需备份再换
fn replace_file(tmp: &Path, path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        if path.exists() {
            let backup = path.with_extension("json.bak");
            let _ = std::fs::remove_file(&backup);
            std::fs::rename(path, &backup).map_err(|e| format!("备份旧静态缓存失败: {e}"))?;
            match std::fs::rename(tmp, path) {
                Ok(()) => {
                    let _ = std::fs::remove_file(&backup);
                    Ok(())
                }
                Err(e) => {
                    let _ = std::fs::rename(&backup, path);
                    let _ = std::fs::remove_file(tmp);
                    Err(format!("Windows 替换静态缓存失败: {e}"))
                }
            }
        } else {
            std::fs::rename(tmp, path).map_err(|e| format!("写入静态缓存失败: {e}"))
        }
    }
    #[cfg(not(windows))]
    {
        std::fs::rename(tmp, path).map_err(|e| format!("原子替换静态缓存失败: {e}"))
    }
}

/// 先写临时文件再替换，避免进程崩溃留下半截 JSON
fn write_json_file_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建静态缓存目录失败: {e}"))?;
    }
    let bytes = serde_json::to_vec(value).map_err(|e| format!("序列化静态缓存失败: {e}"))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &bytes).map_err(|e| format!("写入临时静态缓存失败: {e}"))?;
    replace_file(&tmp, path)
}

fn try_load_from_disk(version: &str) -> Option<(Vec<ChampionInfo>, Vec<SummonerSpellInfo>, i64)> {
    let dir = version_dir(version);
    let meta: DiskMeta = read_json_file(&dir.join(META_FILE))?;
    if meta.version != version {
        return None;
    }
    let champions: Vec<ChampionInfo> = read_json_file(&dir.join(CHAMPIONS_FILE))?;
    let spells: Vec<SummonerSpellInfo> = read_json_file(&dir.join(SPELLS_FILE))?;
    if champions.is_empty() || spells.is_empty() {
        return None;
    }
    Some((champions, spells, meta.loaded_at))
}

/// 扫描磁盘上最新一份完整静态包（按 meta.loaded_at）
fn find_newest_disk_bundle() -> Option<(String, Vec<ChampionInfo>, Vec<SummonerSpellInfo>, i64)> {
    let root = cache_root();
    let entries = std::fs::read_dir(&root).ok()?;
    let mut best: Option<(String, Vec<ChampionInfo>, Vec<SummonerSpellInfo>, i64)> = None;

    for entry in entries.flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let version = entry.file_name().to_string_lossy().into_owned();
        let Some((champions, spells, loaded_at)) = try_load_from_disk(&version) else {
            continue;
        };
        let replace = match &best {
            None => true,
            Some((_, _, _, best_at)) => loaded_at >= *best_at,
        };
        if replace {
            best = Some((version, champions, spells, loaded_at));
        }
    }
    best
}

async fn load_disk_version(version: String) -> Option<(Vec<ChampionInfo>, Vec<SummonerSpellInfo>, i64)> {
    tokio::task::spawn_blocking(move || try_load_from_disk(&version))
        .await
        .ok()
        .flatten()
}

async fn load_newest_disk_bundle() -> Option<(String, Vec<ChampionInfo>, Vec<SummonerSpellInfo>, i64)> {
    tokio::task::spawn_blocking(find_newest_disk_bundle)
        .await
        .ok()
        .flatten()
}

fn persist_to_disk(version: &str, champions: &[ChampionInfo], spells: &[SummonerSpellInfo]) -> Result<i64, String> {
    let dir = version_dir(version);
    let loaded_at = now_ms();
    write_json_file_atomic(&dir.join(CHAMPIONS_FILE), &champions)?;
    write_json_file_atomic(&dir.join(SPELLS_FILE), &spells)?;
    write_json_file_atomic(
        &dir.join(META_FILE),
        &DiskMeta {
            version: version.to_string(),
            loaded_at,
        },
    )?;
    prune_old_versions(version);
    Ok(loaded_at)
}

fn prune_old_versions(keep: &str) {
    let root = cache_root();
    let Ok(entries) = std::fs::read_dir(&root) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.as_ref() != keep && entry.path().is_dir() {
            let _ = std::fs::remove_dir_all(entry.path());
            log::info!("[StaticCatalog] 已清理旧静态包: {name}");
        }
    }
}

fn install_bundle(
    version: &str,
    champions: Vec<ChampionInfo>,
    spells: Vec<SummonerSpellInfo>,
    source: &str,
    loaded_at: i64,
) -> Result<(), String> {
    // 先两边都装上，再写 meta；任一步失败不谎称成功
    install_champion_maps(champions)?;
    install_summoner_spell_maps(spells)?;
    set_meta(StaticCatalogMeta {
        version: version.to_string(),
        loaded_at,
        source: source.to_string(),
    });
    Ok(())
}

fn install_disk_bundle(
    version: &str,
    champions: Vec<ChampionInfo>,
    spells: Vec<SummonerSpellInfo>,
    loaded_at: i64,
    source: &str,
) -> Result<(), String> {
    log::info!(
        "[StaticCatalog] 💾 使用磁盘静态目录 version={version} source={source} champions={} spells={}",
        champions.len(),
        spells.len()
    );
    install_bundle(version, champions, spells, source, loaded_at)
}

async fn load_from_network(version: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    log::info!("[StaticCatalog] 🌐 网络拉取静态目录 version={version}");
    let (champions, spells) = tokio::try_join!(
        fetch_champion_data_from_network(),
        fetch_summoner_spell_data_from_network()
    )?;
    let disk_version = version.to_string();
    let (loaded_at, champions, spells) = tokio::task::spawn_blocking(move || {
        let loaded_at = persist_to_disk(&disk_version, &champions, &spells);
        (loaded_at, champions, spells)
    })
    .await
    .map_err(|error| format!("静态目录落盘任务失败: {error}"))?;
    let loaded_at = loaded_at.unwrap_or_else(|e| {
        log::warn!("[StaticCatalog] 落盘失败（仍将 hydrate 内存）: {e}");
        now_ms()
    });
    install_bundle(version, champions, spells, "network", loaded_at)?;
    log::info!("[StaticCatalog] ✅ 网络静态目录已就绪 version={version}");
    Ok(())
}

async fn init_inner() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match fetch_ddragon_version().await {
        Some(version) => {
            if let Some((champions, spells, loaded_at)) = load_disk_version(version.clone()).await {
                install_disk_bundle(&version, champions, spells, loaded_at, "disk")?;
                return Ok(());
            }

            log::info!("[StaticCatalog] 磁盘未命中 version={version}，改为网络拉取");
            match load_from_network(&version).await {
                Ok(()) => Ok(()),
                Err(net_err) => {
                    log::warn!("[StaticCatalog] 网络拉取失败，尝试磁盘回退: {net_err}");
                    if let Some((v, champions, spells, loaded_at)) = load_newest_disk_bundle().await {
                        install_disk_bundle(&v, champions, spells, loaded_at, "disk-stale")?;
                        Ok(())
                    } else {
                        Err(net_err)
                    }
                }
            }
        }
        None => {
            log::warn!("[StaticCatalog] 无法探测版本，尝试加载磁盘最新缓存");
            if let Some((v, champions, spells, loaded_at)) = load_newest_disk_bundle().await {
                install_disk_bundle(&v, champions, spells, loaded_at, "disk-offline")?;
                Ok(())
            } else {
                Err("无法获取游戏版本，且本地无可用静态目录缓存".into())
            }
        }
    }
}

/// 确保静态目录已加载（启动与 IPC 共用，只执行一次首载）
pub async fn ensure_static_catalogs() -> Result<(), String> {
    INIT.get_or_try_init(|| async { init_inner().await.map_err(|e| e.to_string()) })
        .await
        .map(|_| ())
}

async fn refresh_static_catalogs_inner() -> RefreshResult {
    let Some(latest) = fetch_ddragon_version().await else {
        if champions_loaded() && spells_loaded() {
            log::info!("[StaticCatalog] 离线且内存目录已就绪，跳过刷新");
            return Ok(false);
        }
        if let Some((v, champions, spells, loaded_at)) = load_newest_disk_bundle().await {
            install_disk_bundle(&v, champions, spells, loaded_at, "disk-offline")?;
            return Ok(true);
        }
        return Err("离线且无可用静态目录".to_string());
    };

    let current = get_static_meta().map(|m| m.version);
    if current.as_deref() == Some(latest.as_str()) && champions_loaded() && spells_loaded() {
        log::info!("[StaticCatalog] 版本未变化 ({latest})，保持缓存");
        return Ok(false);
    }

    log::info!("[StaticCatalog] 版本变化 {:?} → {}，刷新静态目录", current, latest);

    if let Some((champions, spells, loaded_at)) = load_disk_version(latest.clone()).await {
        install_disk_bundle(&latest, champions, spells, loaded_at, "disk")?;
        return Ok(true);
    }

    match load_from_network(&latest).await {
        Ok(()) => Ok(true),
        Err(e) => {
            if champions_loaded() && spells_loaded() {
                log::warn!("[StaticCatalog] 刷新失败但保留旧内存目录: {e}");
                Ok(false)
            } else if let Some((v, champions, spells, loaded_at)) = load_newest_disk_bundle().await {
                install_disk_bundle(&v, champions, spells, loaded_at, "disk-stale")?;
                Ok(true)
            } else {
                Err(e.to_string())
            }
        }
    }
}

async fn run_refresh_work() -> RefreshResult {
    #[cfg(test)]
    {
        let hook = TEST_REFRESH_HOOK.lock().await.clone();
        if let Some(hook) = hook {
            hook.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let _ = hook.started.send(());
            hook.release.notified().await;
            return hook.result.clone();
        }
    }
    refresh_static_catalogs_inner().await
}

/// 版本变化时强制刷新。并发调用共享同一次执行结果（true singleflight）。
///
/// coordinator 在独立 task 中运行：调用方取消不会中断刷新，也不会把 flight 卡在 `Running`。
pub async fn refresh_static_catalogs_if_stale() -> Result<bool, String> {
    let rx = {
        let mut flight = REFRESH_FLIGHT.lock().await;
        match &mut *flight {
            RefreshFlight::Idle => {
                let (tx, rx) = oneshot::channel();
                *flight = RefreshFlight::Running { waiters: vec![tx] };
                tokio::spawn(async move {
                    let result = run_refresh_work().await;
                    let waiters = {
                        let mut flight = REFRESH_FLIGHT.lock().await;
                        match std::mem::replace(&mut *flight, RefreshFlight::Idle) {
                            RefreshFlight::Running { waiters } => waiters,
                            RefreshFlight::Idle => Vec::new(),
                        }
                    };
                    for waiter in waiters {
                        let _ = waiter.send(result.clone());
                    }
                });
                rx
            }
            RefreshFlight::Running { waiters } => {
                let (tx, rx) = oneshot::channel();
                waiters.push(tx);
                rx
            }
        }
    };

    rx.await.unwrap_or_else(|_| Err("静态目录刷新被取消".to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex as StdMutex;

    /// 测试间串行化 TEST_CACHE_ROOT，避免并行污染
    static CACHE_ROOT_TEST_LOCK: StdMutex<()> = StdMutex::new(());

    struct TestCacheRootGuard {
        dir: PathBuf,
    }

    impl TestCacheRootGuard {
        fn new(label: &str) -> Self {
            let dir =
                std::env::temp_dir().join(format!("nidalee-static-{}-{}-{}", label, std::process::id(), now_ms()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            *TEST_CACHE_ROOT.write().unwrap() = Some(dir.clone());
            Self { dir }
        }
    }

    impl Drop for TestCacheRootGuard {
        fn drop(&mut self) {
            *TEST_CACHE_ROOT.write().unwrap() = None;
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn sample_champion(id: i32) -> ChampionInfo {
        ChampionInfo {
            id,
            name: format!("Champ{id}"),
            description: String::new(),
            alias: format!("champ{id}"),
            content_id: String::new(),
            square_portrait_path: String::new(),
            roles: vec![],
        }
    }

    fn sample_spell(id: i64) -> SummonerSpellInfo {
        SummonerSpellInfo {
            id,
            name: format!("Spell{id}"),
            description: String::new(),
            summoner_level: 1,
            cooldown: 0,
            game_modes: vec![],
            icon_path: String::new(),
        }
    }

    fn write_complete_bundle(version: &str, loaded_at: i64) {
        let dir = version_dir(version);
        std::fs::create_dir_all(&dir).unwrap();
        write_json_file_atomic(&dir.join(CHAMPIONS_FILE), &vec![sample_champion(1)]).unwrap();
        write_json_file_atomic(&dir.join(SPELLS_FILE), &vec![sample_spell(1)]).unwrap();
        write_json_file_atomic(
            &dir.join(META_FILE),
            &DiskMeta {
                version: version.to_string(),
                loaded_at,
            },
        )
        .unwrap();
    }

    #[test]
    fn disk_meta_roundtrip_shape() {
        let meta = DiskMeta {
            version: "16.1.1".into(),
            loaded_at: 123,
        };
        let bytes = serde_json::to_vec(&meta).unwrap();
        let back: DiskMeta = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(back.version, "16.1.1");
        assert_eq!(back.loaded_at, 123);
    }

    #[test]
    fn replace_file_overwrites_existing_target() {
        let dir = std::env::temp_dir().join(format!("nidalee-static-replace-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let path = dir.join("champions.json");
        let tmp = dir.join("champions.json.tmp");
        std::fs::write(&path, b"old").unwrap();
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(b"new").unwrap();
        }

        replace_file(&tmp, &path).expect("replace should succeed on existing target");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
        assert!(!tmp.exists());
        assert!(!path.with_extension("json.bak").exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn incomplete_disk_bundle_is_rejected() {
        let _lock = CACHE_ROOT_TEST_LOCK.lock().unwrap();
        let _root = TestCacheRootGuard::new("incomplete");

        let version = "16.2.1";
        let dir = version_dir(version);
        std::fs::create_dir_all(&dir).unwrap();
        write_json_file_atomic(&dir.join(CHAMPIONS_FILE), &vec![sample_champion(1)]).unwrap();
        write_json_file_atomic(
            &dir.join(META_FILE),
            &DiskMeta {
                version: version.to_string(),
                loaded_at: 100,
            },
        )
        .unwrap();
        // 缺 summoner-spells.json → 不完整包
        assert!(try_load_from_disk(version).is_none());
    }

    #[test]
    fn corrupted_champions_json_is_rejected() {
        let _lock = CACHE_ROOT_TEST_LOCK.lock().unwrap();
        let _root = TestCacheRootGuard::new("corrupt");

        let version = "16.2.2";
        let dir = version_dir(version);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(CHAMPIONS_FILE), b"{not-json").unwrap();
        write_json_file_atomic(&dir.join(SPELLS_FILE), &vec![sample_spell(1)]).unwrap();
        write_json_file_atomic(
            &dir.join(META_FILE),
            &DiskMeta {
                version: version.to_string(),
                loaded_at: 100,
            },
        )
        .unwrap();
        assert!(try_load_from_disk(version).is_none());
    }

    #[test]
    fn find_newest_disk_bundle_picks_latest_loaded_at() {
        let _lock = CACHE_ROOT_TEST_LOCK.lock().unwrap();
        let _root = TestCacheRootGuard::new("newest");

        write_complete_bundle("15.1.1", 100);
        write_complete_bundle("16.3.1", 300);
        write_complete_bundle("16.1.1", 200);

        let (version, _, _, loaded_at) = find_newest_disk_bundle().expect("bundle");
        assert_eq!(version, "16.3.1");
        assert_eq!(loaded_at, 300);
    }
}

#[cfg(test)]
mod flight_tests {
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
}
