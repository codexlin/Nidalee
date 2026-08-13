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
        let dir = std::env::temp_dir().join(format!("nidalee-static-{}-{}-{}", label, std::process::id(), now_ms()));
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
