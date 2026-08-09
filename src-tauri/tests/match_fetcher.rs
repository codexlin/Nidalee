//! 统一对局数据获取层测试
//!
//! 覆盖：单次列表请求 / 本地截断 / 策略驱动的 timeline 拉取 /
//! 有界并发 / 顺序稳定 / 单局失败降级 / 按需 detail / timeline 会话缓存。
//!
//! 全部通过可注入的数据源完成，不依赖运行中的英雄联盟客户端。
//!
//! 运行：`cargo test --test match_fetcher`

use std::collections::{BTreeSet, HashMap, HashSet};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};

use nidalee_lib::analysis_contract::{
    resolve_analysis_policy, AnalysisDepth, AnalysisMode, AnalysisPolicy, MatchAnalysisRequest,
};
use nidalee_lib::match_fetching::{
    Clock, DetailSource, MatchDataSource, MatchFetcher, TimelineCache, TimelineStatus, LCU_MATCH_LIST_MAX_COUNT,
    MAX_CONCURRENT_MATCH_FETCHES,
};

// === 测试夹具 ===

/// 一场「列表内已含完整详情」的对局
fn complete_game(game_id: u64, queue_id: i64) -> Value {
    json!({
        "gameId": game_id,
        "queueId": queue_id,
        "gameDuration": 1800,
        "participants": [{ "participantId": 1, "teamId": 100 }],
        "participantIdentities": [{ "participantId": 1, "player": { "puuid": "me" } }],
    })
}

/// 一场列表内缺少参与者信息的对局（需要按需补详情）
fn incomplete_game(game_id: u64, queue_id: i64) -> Value {
    json!({
        "gameId": game_id,
        "queueId": queue_id,
    })
}

fn list_response(games: Vec<Value>) -> Value {
    json!({
        "games": {
            "gameCount": games.len(),
            "gameIndexBegin": 0,
            "games": games,
        }
    })
}

fn games_of(list: &Value) -> &Vec<Value> {
    list.get("games")
        .and_then(|g| g.get("games"))
        .and_then(Value::as_array)
        .expect("列表响应缺少 games.games")
}

fn policy(mode: AnalysisMode, depth: AnalysisDepth) -> AnalysisPolicy {
    resolve_analysis_policy(&MatchAnalysisRequest::new(mode, depth))
}

/// 记录数据源收到的全部请求
#[derive(Default, Debug)]
struct Calls {
    list: Vec<usize>,
    detail: Vec<u64>,
    timeline: Vec<u64>,
}

#[derive(Default)]
struct MockSource {
    list: Value,
    list_error: Option<String>,
    details: HashMap<u64, Value>,
    failing_details: HashSet<u64>,
    failing_timelines: HashSet<u64>,
    /// 这些对局的 timeline 会多让出几次执行权，用于制造乱序完成
    slow_timelines: HashSet<u64>,
    calls: Arc<Mutex<Calls>>,
    in_flight: Arc<AtomicUsize>,
    peak_in_flight: Arc<AtomicUsize>,
}

impl MockSource {
    fn new(games: Vec<Value>) -> Self {
        Self {
            list: list_response(games),
            ..Default::default()
        }
    }

    fn calls(&self) -> Arc<Mutex<Calls>> {
        self.calls.clone()
    }

    fn peak_in_flight(&self) -> Arc<AtomicUsize> {
        self.peak_in_flight.clone()
    }

    fn failing_timelines(mut self, ids: &[u64]) -> Self {
        self.failing_timelines = ids.iter().copied().collect();
        self
    }

    fn failing_details(mut self, ids: &[u64]) -> Self {
        self.failing_details = ids.iter().copied().collect();
        self
    }

    fn slow_timelines(mut self, ids: &[u64]) -> Self {
        self.slow_timelines = ids.iter().copied().collect();
        self
    }

    fn with_detail(mut self, game_id: u64, detail: Value) -> Self {
        self.details.insert(game_id, detail);
        self
    }

    fn list_error(mut self, message: &str) -> Self {
        self.list_error = Some(message.to_string());
        self
    }

    /// 进入被跟踪的「在途请求」区间，返回让出次数
    fn enter(&self, slow: bool) -> (Arc<AtomicUsize>, usize) {
        let current = self.in_flight.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak_in_flight.fetch_max(current, Ordering::SeqCst);
        (self.in_flight.clone(), if slow { 8 } else { 2 })
    }
}

impl MatchDataSource for MockSource {
    fn fetch_match_list_raw(
        &self,
        _puuid: &str,
        end_index: usize,
    ) -> impl Future<Output = Result<Value, String>> + Send {
        self.calls.lock().unwrap().list.push(end_index);
        let result = match &self.list_error {
            Some(message) => Err(message.clone()),
            None => Ok(self.list.clone()),
        };
        async move { result }
    }

    fn fetch_game_detail_raw(&self, game_id: u64) -> impl Future<Output = Result<Value, String>> + Send {
        self.calls.lock().unwrap().detail.push(game_id);
        let (in_flight, yields) = self.enter(false);
        let result = if self.failing_details.contains(&game_id) {
            Err(format!("详情不可用: {game_id}"))
        } else {
            Ok(self
                .details
                .get(&game_id)
                .cloned()
                .unwrap_or_else(|| complete_game(game_id, 420)))
        };

        async move {
            for _ in 0..yields {
                tokio::task::yield_now().await;
            }
            in_flight.fetch_sub(1, Ordering::SeqCst);
            result
        }
    }

    fn fetch_game_timeline_raw(&self, game_id: u64) -> impl Future<Output = Result<Value, String>> + Send {
        self.calls.lock().unwrap().timeline.push(game_id);
        let (in_flight, yields) = self.enter(self.slow_timelines.contains(&game_id));
        let result = if self.failing_timelines.contains(&game_id) {
            Err(format!("时间线不可用: {game_id}"))
        } else {
            Ok(json!({ "gameId": game_id, "frames": [] }))
        };

        async move {
            for _ in 0..yields {
                tokio::task::yield_now().await;
            }
            in_flight.fetch_sub(1, Ordering::SeqCst);
            result
        }
    }
}

/// 可手动推进的时钟，避免用 sleep 写出脆弱的过期测试
#[derive(Default, Debug)]
struct ManualClock {
    now_ms: AtomicU64,
}

impl ManualClock {
    fn advance(&self, duration: Duration) {
        self.now_ms.fetch_add(duration.as_millis() as u64, Ordering::SeqCst);
    }
}

impl Clock for ManualClock {
    fn now_ms(&self) -> u64 {
        self.now_ms.load(Ordering::SeqCst)
    }
}

// === 列表：只请求一次 + 本地截断 ===

#[tokio::test]
async fn test_match_list_requested_once_and_truncated_locally() {
    let games: Vec<Value> = (1..=30).map(|id| complete_game(id, 420)).collect();
    let source = MockSource::new(games);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    let list = fetcher.fetch_match_list("me", 10).await.expect("列表获取失败");

    let calls = calls.lock().unwrap();
    assert_eq!(calls.list.len(), 1, "列表必须只请求一次，实际 {:?}", calls.list);
    assert_eq!(calls.list[0], 9, "LCU 闭区间：要 10 场 → endIndex=9");
    assert_eq!(games_of(&list).len(), 10, "响应必须在本地截断到目标数量");
    assert_eq!(
        list["games"]["gameCount"], 10,
        "截断后必须同步 gameCount，实际 {}",
        list["games"]["gameCount"]
    );
}

#[tokio::test]
async fn test_match_list_count_is_clamped() {
    let source = MockSource::new(vec![complete_game(1, 420)]);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    fetcher.fetch_match_list("me", 0).await.expect("列表获取失败");
    fetcher.fetch_match_list("me", 500).await.expect("列表获取失败");

    let calls = calls.lock().unwrap();
    assert_eq!(calls.list[0], 19, "count=0 应回落到默认 20 场 → endIndex=19");
    assert_eq!(
        calls.list[1],
        LCU_MATCH_LIST_MAX_COUNT - 1,
        "超过 LCU 上限的请求必须被夹紧为 endIndex=99"
    );
}

#[tokio::test]
async fn test_list_failure_propagates() {
    let source = MockSource::new(vec![]).list_error("LCU 未连接");
    let fetcher = MatchFetcher::new(source);

    let error = fetcher
        .fetch_bundles("me", 20, &policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep))
        .await
        .expect_err("列表失败必须直接返回错误");
    assert!(error.contains("LCU 未连接"), "错误应保留原因，实际 {error}");
}

// === 批量：策略过滤 + 单次列表 ===

#[tokio::test]
async fn test_bundles_filtered_by_policy_and_capped_by_effective_count() {
    let games = vec![
        complete_game(1, 420),
        complete_game(2, 450),
        complete_game(3, 420),
        complete_game(4, 2400),
        complete_game(5, 420),
        complete_game(6, 420),
    ];
    let source = MockSource::new(games);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    let mut request = MatchAnalysisRequest::new(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    request.count = 20;
    request.max_analysis_games = Some(2);
    let policy = resolve_analysis_policy(&request);

    let outcome = fetcher.fetch_bundles("me", 20, &policy).await.expect("批量获取失败");

    assert_eq!(
        outcome.bundles.iter().map(|b| b.game_id).collect::<Vec<_>>(),
        vec![1, 3],
        "只保留策略选中的队列，并截断到 effective_game_count"
    );
    assert_eq!(outcome.stats.list_requests, 1);
    let list_calls = calls.lock().unwrap();
    assert_eq!(list_calls.list.len(), 1, "批量流程仍然只请求一次列表");
    assert_eq!(
        list_calls.list[0],
        LCU_MATCH_LIST_MAX_COUNT - 1,
        "有队列过滤时应 over-fetch 到 LCU 上限（endIndex=99）再本地过滤"
    );
    assert_eq!(games_of(&outcome.raw_list).len(), 6, "原始列表响应必须完整保留");
    assert!(
        outcome.insufficient_matches_in_scope,
        "展示目标 20、过滤后仅 4 场 420，应标记 InsufficientMatchesInScope"
    );
}

#[tokio::test]
async fn test_queue_filter_overfetches_to_lcu_cap() {
    let games: Vec<Value> = (1..=40)
        .map(|id| complete_game(id, if id % 2 == 0 { 420 } else { 450 }))
        .collect();
    let source = MockSource::new(games);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    let outcome = fetcher
        .fetch_bundles("me", 20, &policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep))
        .await
        .expect("批量获取失败");

    assert_eq!(calls.lock().unwrap().list[0], LCU_MATCH_LIST_MAX_COUNT - 1);
    assert_eq!(outcome.display_games.len(), 20, "应凑满请求的展示场数");
    assert!(!outcome.insufficient_matches_in_scope);
    assert!(outcome.display_games.iter().all(|g| g["queueId"] == 420));
}

#[tokio::test]
async fn test_queue_filter_marks_insufficient_matches_in_scope() {
    // 100 场混合历史里只有 3 场单排，请求 20 场展示
    let mut games: Vec<Value> = (1..=97).map(|id| complete_game(id, 450)).collect();
    games.push(complete_game(98, 420));
    games.push(complete_game(99, 420));
    games.push(complete_game(100, 420));
    let source = MockSource::new(games);
    let fetcher = MatchFetcher::new(source);

    let outcome = fetcher
        .fetch_bundles("me", 20, &policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep))
        .await
        .expect("批量获取失败");

    assert_eq!(outcome.display_games.len(), 3);
    assert!(outcome.insufficient_matches_in_scope);
    assert!(
        outcome
            .diagnostics
            .iter()
            .any(|d| d.message.contains("队列过滤后仅找到 3 场")),
        "应写入列表级诊断，实际 {:?}",
        outcome.diagnostics
    );
}

#[tokio::test]
async fn test_all_modes_does_not_overfetch_list() {
    let games: Vec<Value> = (1..=30).map(|id| complete_game(id, 420)).collect();
    let source = MockSource::new(games);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    fetcher
        .fetch_bundles("me", 20, &policy(AnalysisMode::AllModes, AnalysisDepth::Simple))
        .await
        .expect("批量获取失败");

    assert_eq!(
        calls.lock().unwrap().list[0],
        19,
        "无队列过滤时：展示 20 场 → endIndex=19"
    );
}

// === timeline：只在 420/440 且 Deep + 开关开启时拉取 ===

#[tokio::test]
async fn test_deep_ranked_fetches_timeline_for_420_and_440() {
    let games = vec![complete_game(1, 420), complete_game(2, 440)];
    let source = MockSource::new(games);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    let outcome = fetcher
        .fetch_bundles("me", 20, &policy(AnalysisMode::MixedRanked, AnalysisDepth::Deep))
        .await
        .expect("批量获取失败");

    assert_eq!(calls.lock().unwrap().timeline.clone(), vec![1, 2]);
    assert_eq!(outcome.stats.timeline_requests, 2);
    for bundle in &outcome.bundles {
        assert_eq!(bundle.timeline_status, TimelineStatus::Fetched);
        assert!(bundle.timeline.is_some(), "深度排位局必须带 timeline");
        assert!(!bundle.is_degraded());
    }
}

#[tokio::test]
async fn test_fun_queues_get_zero_timeline_requests() {
    let games = vec![complete_game(1, 450), complete_game(2, 2400), complete_game(3, 420)];
    let source = MockSource::new(games);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    // 全部模式 + Deep：按局策略，娱乐局降级为 Simple
    let outcome = fetcher
        .fetch_bundles("me", 20, &policy(AnalysisMode::AllModes, AnalysisDepth::Deep))
        .await
        .expect("批量获取失败");

    assert_eq!(
        calls.lock().unwrap().timeline.clone(),
        vec![3],
        "只有排位局才允许请求 timeline"
    );
    assert_eq!(outcome.bundles[0].timeline_status, TimelineStatus::Skipped);
    assert_eq!(outcome.bundles[1].timeline_status, TimelineStatus::Skipped);
    assert!(outcome.bundles[0].timeline.is_none());
    assert!(!outcome.bundles[0].is_degraded(), "策略跳过不属于数据降级");
}

#[tokio::test]
async fn test_aram_deep_gets_zero_timeline_requests() {
    let source = MockSource::new(vec![complete_game(1, 450)]);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    fetcher
        .fetch_bundles("me", 20, &policy(AnalysisMode::Aram, AnalysisDepth::Deep))
        .await
        .expect("批量获取失败");

    assert!(
        calls.lock().unwrap().timeline.is_empty(),
        "娱乐模式深度会降级为简单，不应请求 timeline"
    );
}

#[tokio::test]
async fn test_simple_depth_gets_zero_timeline_requests() {
    let source = MockSource::new(vec![complete_game(1, 420), complete_game(2, 440)]);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    fetcher
        .fetch_bundles("me", 20, &policy(AnalysisMode::MixedRanked, AnalysisDepth::Simple))
        .await
        .expect("批量获取失败");

    assert!(calls.lock().unwrap().timeline.is_empty(), "Simple 深度不做时间线");
}

#[tokio::test]
async fn test_timeline_flag_off_gets_zero_timeline_requests() {
    let source = MockSource::new(vec![complete_game(1, 420)]);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    let mut request = MatchAnalysisRequest::new(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    request.features.timeline = false;

    let outcome = fetcher
        .fetch_bundles("me", 20, &resolve_analysis_policy(&request))
        .await
        .expect("批量获取失败");

    assert!(calls.lock().unwrap().timeline.is_empty(), "时间线开关关闭时零请求");
    assert_eq!(outcome.stats.timeline_requests, 0);
    assert_eq!(outcome.bundles[0].timeline_status, TimelineStatus::Skipped);
}

#[tokio::test]
async fn test_timeline_limited_to_effective_game_count() {
    let games: Vec<Value> = (1..=10).map(|id| complete_game(id, 420)).collect();
    let source = MockSource::new(games);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    let mut request = MatchAnalysisRequest::new(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    request.count = 10;
    request.max_analysis_games = Some(3);

    let outcome = fetcher
        .fetch_bundles("me", 10, &resolve_analysis_policy(&request))
        .await
        .expect("批量获取失败");

    assert_eq!(calls.lock().unwrap().timeline.clone(), vec![1, 2, 3]);
    assert_eq!(outcome.bundles.len(), 3);
}

// === 有界并发与顺序 ===

#[tokio::test]
async fn test_bounded_concurrency_never_exceeds_limit() {
    let games: Vec<Value> = (1..=12).map(|id| complete_game(id, 420)).collect();
    let source = MockSource::new(games);
    let peak = source.peak_in_flight();
    let fetcher = MatchFetcher::new(source);

    fetcher
        .fetch_bundles("me", 12, &policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep))
        .await
        .expect("批量获取失败");

    let peak = peak.load(Ordering::SeqCst);
    assert!(
        peak <= MAX_CONCURRENT_MATCH_FETCHES,
        "并发数不得超过 {MAX_CONCURRENT_MATCH_FETCHES}，实际峰值 {peak}"
    );
    assert!(peak > 1, "应当真正并发执行，实际峰值 {peak}");
}

#[tokio::test]
async fn test_bundle_order_matches_list_order() {
    let games: Vec<Value> = (1..=8).map(|id| complete_game(id, 420)).collect();
    // 偶数局的时间线返回更慢，保证完成顺序与列表顺序不一致
    let source = MockSource::new(games).slow_timelines(&[2, 4, 6, 8]);
    let fetcher = MatchFetcher::new(source);

    let outcome = fetcher
        .fetch_bundles("me", 8, &policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep))
        .await
        .expect("批量获取失败");

    assert_eq!(
        outcome.bundles.iter().map(|b| b.game_id).collect::<Vec<_>>(),
        (1..=8).collect::<Vec<u64>>(),
        "输出顺序必须与列表原始顺序一致"
    );
}

// === 单局失败只降级该局 ===

#[tokio::test]
async fn test_single_timeline_failure_degrades_only_that_bundle() {
    let games: Vec<Value> = (1..=4).map(|id| complete_game(id, 420)).collect();
    let source = MockSource::new(games).failing_timelines(&[2]);
    let fetcher = MatchFetcher::new(source);

    let outcome = fetcher
        .fetch_bundles("me", 4, &policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep))
        .await
        .expect("单局失败不得让整批失败");

    assert_eq!(outcome.bundles.len(), 4);
    let failed = &outcome.bundles[1];
    assert_eq!(failed.game_id, 2);
    assert_eq!(failed.timeline_status, TimelineStatus::Failed);
    assert!(failed.timeline.is_none());
    assert!(failed.is_degraded());
    assert!(!failed.diagnostics.is_empty(), "失败必须留下诊断");

    for bundle in [&outcome.bundles[0], &outcome.bundles[2], &outcome.bundles[3]] {
        assert_eq!(bundle.timeline_status, TimelineStatus::Fetched);
        assert!(!bundle.is_degraded());
    }
    assert_eq!(outcome.stats.degraded_bundles, 1);
}

// === detail：默认复用列表内数据，按需才请求 ===

#[tokio::test]
async fn test_complete_list_games_skip_detail_requests() {
    let games: Vec<Value> = (1..=5).map(|id| complete_game(id, 420)).collect();
    let source = MockSource::new(games);
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    let outcome = fetcher
        .fetch_bundles("me", 5, &policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep))
        .await
        .expect("批量获取失败");

    assert!(
        calls.lock().unwrap().detail.is_empty(),
        "列表已含完整对局时不得再请求 /games/{{id}}"
    );
    assert_eq!(outcome.stats.detail_requests, 0);
    for bundle in &outcome.bundles {
        assert_eq!(bundle.detail_source, DetailSource::ListEmbedded);
        assert_eq!(bundle.game()["gameId"], json!(bundle.game_id));
    }
}

#[tokio::test]
async fn test_incomplete_list_game_triggers_detail_fetch() {
    let games = vec![complete_game(1, 420), incomplete_game(2, 420)];
    let source = MockSource::new(games).with_detail(2, complete_game(2, 420));
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    let outcome = fetcher
        .fetch_bundles("me", 2, &policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep))
        .await
        .expect("批量获取失败");

    assert_eq!(calls.lock().unwrap().detail.clone(), vec![2], "只补齐缺失详情的对局");
    assert_eq!(outcome.bundles[0].detail_source, DetailSource::ListEmbedded);
    assert_eq!(outcome.bundles[1].detail_source, DetailSource::Fetched);
    assert!(outcome.bundles[1].game()["participants"].is_array());
}

#[tokio::test]
async fn test_detail_failure_degrades_bundle_without_failing_batch() {
    let games = vec![incomplete_game(1, 420), complete_game(2, 420)];
    let source = MockSource::new(games).failing_details(&[1]);
    let fetcher = MatchFetcher::new(source);

    let outcome = fetcher
        .fetch_bundles("me", 2, &policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep))
        .await
        .expect("详情失败不得让整批失败");

    assert_eq!(outcome.bundles[0].detail_source, DetailSource::Unavailable);
    assert!(outcome.bundles[0].is_degraded());
    assert!(!outcome.bundles[0].diagnostics.is_empty());
    assert!(!outcome.bundles[1].is_degraded());
}

#[tokio::test]
async fn test_on_demand_detail_api() {
    let source = MockSource::new(vec![]).with_detail(77, complete_game(77, 440));
    let calls = source.calls();
    let fetcher = MatchFetcher::new(source);

    let detail = fetcher.fetch_game_detail(77).await.expect("按需详情获取失败");

    assert_eq!(detail["gameId"], json!(77));
    assert_eq!(calls.lock().unwrap().detail.clone(), vec![77]);
}

// === timeline 会话缓存 ===

#[tokio::test]
async fn test_timeline_cache_hit_avoids_second_request() {
    let games: Vec<Value> = (1..=3).map(|id| complete_game(id, 420)).collect();
    let source = MockSource::new(games);
    let calls = source.calls();
    let cache = Arc::new(TimelineCache::new(Duration::from_secs(300)));
    let fetcher = MatchFetcher::with_timeline_cache(source, cache.clone());
    let policy = policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep);

    let first = fetcher.fetch_bundles("me", 3, &policy).await.expect("首轮获取失败");
    let second = fetcher.fetch_bundles("me", 3, &policy).await.expect("次轮获取失败");

    assert_eq!(calls.lock().unwrap().timeline.len(), 3, "第二轮必须全部命中缓存");
    assert_eq!(first.stats.timeline_requests, 3);
    assert_eq!(second.stats.timeline_requests, 0);
    assert_eq!(second.stats.timeline_cache_hits, 3);
    assert_eq!(cache.len(), 3);
    for bundle in &second.bundles {
        assert_eq!(bundle.timeline_status, TimelineStatus::CacheHit);
        assert!(bundle.timeline.is_some());
    }
}

#[tokio::test]
async fn test_timeline_cache_expires_after_ttl() {
    let source = MockSource::new(vec![complete_game(1, 420)]);
    let calls = source.calls();
    let clock = Arc::new(ManualClock::default());
    let cache = Arc::new(TimelineCache::with_clock(Duration::from_secs(60), clock.clone()));
    let fetcher = MatchFetcher::with_timeline_cache(source, cache.clone());
    let policy = policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep);

    fetcher.fetch_bundles("me", 1, &policy).await.expect("首轮获取失败");

    clock.advance(Duration::from_secs(59));
    assert!(cache.get(1).is_some(), "未过期时必须命中");

    clock.advance(Duration::from_secs(2));
    assert!(cache.get(1).is_none(), "超过 TTL 后必须失效");

    let after = fetcher.fetch_bundles("me", 1, &policy).await.expect("次轮获取失败");
    assert_eq!(calls.lock().unwrap().timeline.clone(), vec![1, 1], "过期后必须重新请求");
    assert_eq!(after.bundles[0].timeline_status, TimelineStatus::Fetched);
}

#[tokio::test]
async fn test_timeline_cache_clear_forces_refetch() {
    let source = MockSource::new(vec![complete_game(1, 420)]);
    let calls = source.calls();
    let cache = Arc::new(TimelineCache::new(Duration::from_secs(300)));
    let fetcher = MatchFetcher::with_timeline_cache(source, cache.clone());
    let policy = policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep);

    fetcher.fetch_bundles("me", 1, &policy).await.expect("首轮获取失败");
    cache.clear();
    assert!(cache.is_empty());

    fetcher.fetch_bundles("me", 1, &policy).await.expect("次轮获取失败");
    assert_eq!(calls.lock().unwrap().timeline.len(), 2, "清空后必须重新请求");
}

#[test]
fn test_timeline_cache_evicts_least_recently_used_when_full() {
    let cache = TimelineCache::with_capacity(Duration::from_secs(300), 2);
    cache.insert(1, json!({ "frames": [] }));
    cache.insert(2, json!({ "frames": [] }));
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.capacity(), 2);

    // 访问 1，使 2 成为最久未用
    assert!(cache.get(1).is_some());
    cache.insert(3, json!({ "frames": [] }));

    assert_eq!(cache.len(), 2);
    assert!(cache.get(1).is_some(), "刚访问过的条目应保留");
    assert!(cache.get(3).is_some(), "新写入应保留");
    assert!(cache.get(2).is_none(), "最久未访问的条目应被淘汰");
}

// === 唯一入口守卫 ===

/// 递归收集 `src/` 下全部 Rust 源文件（测试进程 CWD 为 src-tauri 包根）
fn rust_sources(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("读取 {} 失败: {e}", dir.display())) {
        let path = entry.expect("遍历源码目录失败").path();
        if path.is_dir() {
            rust_sources(&path, found);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            found.push(path);
        }
    }
}

/// 出现指定片段的源文件（相对路径，统一用 `/` 分隔）
fn sources_containing(pattern: &str) -> BTreeSet<String> {
    let mut files = Vec::new();
    rust_sources(Path::new("src"), &mut files);

    files
        .into_iter()
        .filter(|path| {
            std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("读取 {} 失败: {e}", path.display()))
                .contains(pattern)
        })
        .map(|path| path.to_string_lossy().replace('\\', "/"))
        .collect()
}

#[test]
fn test_match_list_url_is_assembled_only_in_fetcher() {
    let unified_entry = "src/infrastructure/match_management/matches/fetcher.rs".to_string();

    assert_eq!(
        sources_containing("begIndex=0&endIndex="),
        BTreeSet::from([unified_entry.clone()]),
        "统一战绩列表语义（begIndex=0 + 闭区间 endIndex）只能由 fetcher 拼装，其余调用方必须走 fetch_match_list"
    );

    assert_eq!(
        sources_containing("products/lol/"),
        BTreeSet::from([
            unified_entry,
            // 分页探针命令刻意变化 begIndex 以验证 LCU 是否忽略它，无法走固定 begIndex=0 的统一入口
            "src/common/commands/data_collection.rs".to_string(),
        ]),
        "除分页探针外，不允许出现第二处战绩列表 URL 拼装"
    );
}

#[tokio::test]
async fn test_failed_timeline_is_not_cached() {
    let source = MockSource::new(vec![complete_game(1, 420)]).failing_timelines(&[1]);
    let calls = source.calls();
    let cache = Arc::new(TimelineCache::new(Duration::from_secs(300)));
    let fetcher = MatchFetcher::with_timeline_cache(source, cache.clone());
    let policy = policy(AnalysisMode::SoloRanked, AnalysisDepth::Deep);

    fetcher.fetch_bundles("me", 1, &policy).await.expect("批量获取失败");
    fetcher.fetch_bundles("me", 1, &policy).await.expect("批量获取失败");

    assert!(cache.is_empty(), "失败响应不得进入缓存");
    assert_eq!(calls.lock().unwrap().timeline.len(), 2);
}
