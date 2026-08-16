//! 对局数据的统一获取层
//!
//! 这里是「战绩列表 / 单局详情 / 时间线」三类数据的唯一入口，向上只输出
//! [`MatchBundle`]，向下把请求次数压到策略允许的最小值：
//!
//! - 列表：整个流程只请求一次；LCU 的 `begIndex`/`endIndex` 为闭区间，
//!   要 N 场则请求 `0..(N-1)`，并在本地再截断对齐
//! - 详情：战绩列表本身通常已包含完整对局，默认零额外请求；只有列表数据残缺才按需补齐
//! - 时间线：只对「策略选中 + 实际队列为 420/440 + 该局深度为 Deep + 时间线开关开启」
//!   的前 `effective_game_count` 场发起请求，娱乐局 / Simple / 开关关闭一律零请求
//!
//! 并发受 [`MAX_CONCURRENT_MATCH_FETCHES`] 约束且输出保持列表原始顺序；
//! 任何单局失败只降级该 bundle，不会让整批分析失败。
//!
//! 数据源通过 [`MatchDataSource`] 注入，因此全部行为都能在没有英雄联盟客户端的情况下测试。

use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::{self, StreamExt};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::domains::analysis::pipeline::{is_ranked_queue, AnalysisPolicy};
use crate::shared::utils::lcu_get;

use super::fetch_types::{
    DetailSource, FetchDiagnostic, FetchStage, FetchStats, MatchBundle, MatchFetchOutcome, TimelineStatus,
};
use super::timeline_cache::TimelineCache;

/// LCU 单次战绩请求的数量上限
pub const LCU_MATCH_LIST_MAX_COUNT: usize = 100;

/// 带队列过滤时扫描的最近对局候选数。
///
/// 展示与分析仍由策略限制为更小的目标数；这里仅扩大候选窗口，避免混合模式历史中
/// 目标队列样本不足，同时避免每位玩家都请求 LCU 的 100 场上限。
pub const FILTERED_MATCH_LIST_SCAN_COUNT: usize = 50;

/// count 缺省时的对局数量
pub const DEFAULT_MATCH_LIST_COUNT: usize = 20;

/// 单局补数据（详情 / 时间线）的并发上限
pub const MAX_CONCURRENT_MATCH_FETCHES: usize = 4;

/// 时间线会话缓存的默认存活时间
pub const DEFAULT_TIMELINE_CACHE_TTL: Duration = Duration::from_secs(600);
pub const DEFAULT_DETAIL_CACHE_TTL: Duration = Duration::from_secs(300);

/// 进程内共享的时间线缓存：只在内存中，不落盘，也不持有任何 LCU 认证信息
static SHARED_TIMELINE_CACHE: Lazy<Arc<TimelineCache>> =
    Lazy::new(|| Arc::new(TimelineCache::new(DEFAULT_TIMELINE_CACHE_TTL)));
static SHARED_DETAIL_CACHE: Lazy<Arc<TimelineCache>> =
    Lazy::new(|| Arc::new(TimelineCache::new(DEFAULT_DETAIL_CACHE_TTL)));
static LCU_MATCH_LIST_LIMITER: Lazy<Semaphore> = Lazy::new(|| Semaphore::new(2));

/// 原始数据源：只负责发请求，不做任何策略判断与数据整形
pub trait MatchDataSource {
    /// 拉取一次战绩列表（`end_index` 已由调用方夹紧）
    fn fetch_match_list_raw(&self, puuid: &str, end_index: usize)
        -> impl Future<Output = Result<Value, String>> + Send;

    /// 拉取单局详情
    fn fetch_game_detail_raw(&self, game_id: u64) -> impl Future<Output = Result<Value, String>> + Send;

    /// 拉取单局时间线
    fn fetch_game_timeline_raw(&self, game_id: u64) -> impl Future<Output = Result<Value, String>> + Send;
}

/// 真实 LCU 数据源
pub struct LcuMatchDataSource<'a> {
    client: &'a Client,
}

impl<'a> LcuMatchDataSource<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }
}

impl MatchDataSource for LcuMatchDataSource<'_> {
    fn fetch_match_list_raw(
        &self,
        puuid: &str,
        end_index: usize,
    ) -> impl Future<Output = Result<Value, String>> + Send {
        let client = self.client;
        let url = format!(
            "/lol-match-history/v1/products/lol/{}/matches?begIndex=0&endIndex={}",
            puuid, end_index
        );
        async move {
            let _permit = LCU_MATCH_LIST_LIMITER
                .acquire()
                .await
                .map_err(|_| "LCU match-history limiter closed unexpectedly".to_owned())?;
            lcu_get::<Value>(client, &url).await
        }
    }

    fn fetch_game_detail_raw(&self, game_id: u64) -> impl Future<Output = Result<Value, String>> + Send {
        let client = self.client;
        let url = format!("/lol-match-history/v1/games/{}", game_id);
        async move { lcu_get::<Value>(client, &url).await }
    }

    fn fetch_game_timeline_raw(&self, game_id: u64) -> impl Future<Output = Result<Value, String>> + Send {
        let client = self.client;
        let url = format!("/lol-match-history/v1/game-timelines/{}", game_id);
        async move { lcu_get::<Value>(client, &url).await }
    }
}

/// 时间线数据的来源，用于区分「命中缓存」与「真实请求」
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TimelineOrigin {
    Cache,
    Network,
}

/// 统一数据获取层
pub struct MatchFetcher<S: MatchDataSource> {
    source: S,
    detail_cache: Arc<TimelineCache>,
    timeline_cache: Arc<TimelineCache>,
    concurrency: usize,
}

impl<S: MatchDataSource> MatchFetcher<S> {
    pub fn new(source: S) -> Self {
        Self::with_caches(
            source,
            Arc::new(TimelineCache::new(DEFAULT_DETAIL_CACHE_TTL)),
            Arc::new(TimelineCache::new(DEFAULT_TIMELINE_CACHE_TTL)),
        )
    }

    pub fn with_timeline_cache(source: S, timeline_cache: Arc<TimelineCache>) -> Self {
        Self::with_caches(
            source,
            Arc::new(TimelineCache::new(DEFAULT_DETAIL_CACHE_TTL)),
            timeline_cache,
        )
    }

    fn with_caches(source: S, detail_cache: Arc<TimelineCache>, timeline_cache: Arc<TimelineCache>) -> Self {
        Self {
            source,
            detail_cache,
            timeline_cache,
            concurrency: MAX_CONCURRENT_MATCH_FETCHES,
        }
    }

    pub fn timeline_cache(&self) -> &Arc<TimelineCache> {
        &self.timeline_cache
    }

    /// 单次请求战绩列表（列表获取的唯一实现）
    ///
    /// LCU `begIndex`/`endIndex` 为**闭区间**（要 30 场 → `0..29`）。
    /// 客户端会忽略非零 `begIndex`，因此始终从 0 开始；
    /// 响应多于目标数量时在本地截断，并同步 `gameCount`。
    pub async fn fetch_match_list(&self, puuid: &str, count: usize) -> Result<Value, String> {
        let target = resolve_list_count(count);
        let end_index = count_to_lcu_end_index(target);
        let mut response = self.source.fetch_match_list_raw(puuid, end_index).await?;
        let (raw_count, actual_count) = truncate_match_list(&mut response, target)?;

        log::info!(
            "[战绩列表] 单次请求完成: begIndex=0&endIndex={} → 原始返回 {} 场，本地保留 {} 场（目标 {}）",
            end_index,
            raw_count,
            actual_count,
            target
        );
        Ok(response)
    }

    /// 按需获取单局详情
    ///
    /// 批量流程默认复用列表内的完整对局，只有列表数据残缺时才会走到这里。
    pub async fn fetch_game_detail(&self, game_id: u64) -> Result<Value, String> {
        if let Some(cached) = self.detail_cache.get(game_id) {
            return Ok(cached);
        }
        let detail = self.source.fetch_game_detail_raw(game_id).await?;
        self.detail_cache.insert(game_id, detail.clone());
        Ok(detail)
    }

    /// 获取单局时间线（成功响应会写入会话缓存）
    pub async fn fetch_game_timeline(&self, game_id: u64) -> Result<Value, String> {
        self.load_timeline(game_id).await.map(|(timeline, _)| timeline)
    }

    /// 时间线的唯一读取路径：先查会话缓存，未命中才请求，且只缓存成功响应
    async fn load_timeline(&self, game_id: u64) -> Result<(Value, TimelineOrigin), String> {
        if let Some(cached) = self.timeline_cache.get(game_id) {
            return Ok((cached, TimelineOrigin::Cache));
        }

        let timeline = self.source.fetch_game_timeline_raw(game_id).await?;
        self.timeline_cache.insert(game_id, timeline.clone());
        Ok((timeline, TimelineOrigin::Network))
    }

    /// 按策略批量获取分析所需的全部素材
    ///
    /// `list_count` 是用户期望的**展示**场数。若策略带队列过滤，会扫描最近 50 场
    /// 候选再本地过滤，尽量凑满 `list_count`；仍不足则写入诊断。
    /// 深度 bundle 数量另受 `effective_game_count` 约束。
    pub async fn fetch_bundles(
        &self,
        puuid: &str,
        list_count: usize,
        policy: &AnalysisPolicy,
    ) -> Result<MatchFetchOutcome, String> {
        let display_target = resolve_list_count(list_count);
        let fetch_count = resolve_list_fetch_count(policy, display_target);
        let raw_list = self.fetch_match_list(puuid, fetch_count).await?;
        let (display_games, bundles, mut diagnostics) = build_bundles(&raw_list, policy, display_target);

        // 仅在有队列过滤时报告「范围内不足」；无过滤时场数不够只是历史本身偏少
        let insufficient_matches_in_scope = policy.has_queue_filter() && display_games.len() < display_target;
        if insufficient_matches_in_scope {
            diagnostics.push(FetchDiagnostic::new(
                FetchStage::List,
                0,
                format!(
                    "队列过滤后仅找到 {} 场符合条件的对局（期望 {} 场，已扫描最近 {} 场）",
                    display_games.len(),
                    display_target,
                    fetch_count
                ),
            ));
        }

        let bundles: Vec<MatchBundle> = stream::iter(bundles.into_iter().map(|bundle| self.enrich(bundle, policy)))
            .buffered(self.concurrency)
            .collect()
            .await;

        let stats = collect_stats(&bundles);
        Ok(MatchFetchOutcome {
            raw_list,
            display_games,
            bundles,
            stats,
            diagnostics,
            insufficient_matches_in_scope,
        })
    }

    /// 单局补数据：先补详情（仅在列表残缺时），再按策略补时间线
    async fn enrich(&self, mut bundle: MatchBundle, policy: &AnalysisPolicy) -> MatchBundle {
        if bundle.detail_source == DetailSource::Unavailable {
            match self.fetch_game_detail(bundle.game_id).await {
                Ok(detail) => {
                    bundle.detail = Some(detail);
                    bundle.detail_source = DetailSource::Fetched;
                }
                Err(error) => bundle.push_diagnostic(FetchStage::Detail, error),
            }
        }

        if !timeline_wanted(policy, bundle.queue_id) {
            bundle.timeline_status = TimelineStatus::Skipped;
            return bundle;
        }

        match self.load_timeline(bundle.game_id).await {
            Ok((timeline, origin)) => {
                bundle.timeline = Some(timeline);
                bundle.timeline_status = match origin {
                    TimelineOrigin::Cache => TimelineStatus::CacheHit,
                    TimelineOrigin::Network => TimelineStatus::Fetched,
                };
            }
            Err(error) => {
                bundle.timeline_status = TimelineStatus::Failed;
                bundle.push_diagnostic(FetchStage::Timeline, error);
            }
        }

        bundle
    }
}

/// 基于真实 LCU 客户端的获取层（复用进程内共享的时间线缓存）
pub fn lcu_fetcher(client: &Client) -> MatchFetcher<LcuMatchDataSource<'_>> {
    MatchFetcher::with_caches(
        LcuMatchDataSource::new(client),
        SHARED_DETAIL_CACHE.clone(),
        SHARED_TIMELINE_CACHE.clone(),
    )
}

/// 单次请求战绩列表（面向既有调用方的自由函数入口）
pub async fn fetch_match_list(client: &Client, puuid: &str, count: usize) -> Result<Value, String> {
    lcu_fetcher(client).fetch_match_list(puuid, count).await
}

/// 按策略批量获取对局素材（集成测试 / 门面入口；生产路径走 `MatchFetcher::fetch_bundles`）
#[allow(dead_code)]
pub async fn fetch_match_bundles(
    client: &Client,
    puuid: &str,
    list_count: usize,
    policy: &AnalysisPolicy,
) -> Result<MatchFetchOutcome, String> {
    lcu_fetcher(client).fetch_bundles(puuid, list_count, policy).await
}

/// 时间线只对排位深度局有意义：非 420/440 一律不请求
fn timeline_wanted(policy: &AnalysisPolicy, queue_id: i64) -> bool {
    is_ranked_queue(queue_id) && policy.timeline_enabled_for_queue(queue_id)
}

fn resolve_list_count(count: usize) -> usize {
    if count == 0 {
        DEFAULT_MATCH_LIST_COUNT
    } else {
        count.min(LCU_MATCH_LIST_MAX_COUNT)
    }
}

/// 将「期望场数」转为 LCU 闭区间 `endIndex`（N 场 → N-1）。
fn count_to_lcu_end_index(count: usize) -> usize {
    count.saturating_sub(1)
}

/// 有队列过滤时扫描最近 50 场候选，尽量在本地凑满展示场数；无过滤则按目标场数请求。
fn resolve_list_fetch_count(policy: &AnalysisPolicy, display_target: usize) -> usize {
    if policy.has_queue_filter() {
        display_target
            .max(FILTERED_MATCH_LIST_SCAN_COUNT)
            .min(LCU_MATCH_LIST_MAX_COUNT)
    } else {
        display_target
    }
}

/// 本地截断到目标数量并同步 `gameCount`，返回（原始数量，保留数量）
fn truncate_match_list(response: &mut Value, target: usize) -> Result<(usize, usize), String> {
    let games = response
        .get_mut("games")
        .and_then(|games| games.get_mut("games"))
        .and_then(Value::as_array_mut)
        .ok_or_else(|| "LCU 战绩响应缺少 games.games".to_string())?;

    let raw_count = games.len();
    games.truncate(target);
    let actual_count = games.len();

    if let Some(games_obj) = response.get_mut("games").and_then(Value::as_object_mut) {
        games_obj.insert("gameCount".to_string(), Value::from(actual_count as u64));
    }

    Ok((raw_count, actual_count))
}

/// 按策略过滤队列，产出「展示对局」与「受深度上限约束的分析素材」
///
/// 两者都保持列表原始顺序；`display_games` 最多 `display_limit` 场，
/// `bundles` 是其前缀子集（受 `effective_game_count` 约束）。
fn build_bundles(
    raw_list: &Value,
    policy: &AnalysisPolicy,
    display_limit: usize,
) -> (Vec<Value>, Vec<MatchBundle>, Vec<FetchDiagnostic>) {
    let mut display_games = Vec::new();
    let mut bundles = Vec::new();
    let mut diagnostics = Vec::new();
    let deep_limit = policy.effective_game_count as usize;

    let Some(games) = raw_list
        .get("games")
        .and_then(|games| games.get("games"))
        .and_then(Value::as_array)
    else {
        diagnostics.push(FetchDiagnostic::new(
            FetchStage::List,
            0,
            "LCU 战绩响应缺少 games.games",
        ));
        return (display_games, bundles, diagnostics);
    };

    for game in games {
        if display_games.len() >= display_limit && bundles.len() >= deep_limit {
            break;
        }

        let queue_id = game.get("queueId").and_then(Value::as_i64).unwrap_or_default();
        if !policy.includes_queue(queue_id) {
            continue;
        }

        // 展示不受深度上限影响，但受用户请求的 count 约束
        if display_games.len() < display_limit {
            display_games.push(game.clone());
        }

        if bundles.len() >= deep_limit {
            continue;
        }

        let Some(game_id) = game.get("gameId").and_then(Value::as_u64).filter(|id| *id > 0) else {
            diagnostics.push(FetchDiagnostic::new(FetchStage::List, 0, "对局记录缺少有效 gameId"));
            continue;
        };

        let detail_source = if list_game_has_full_detail(game) {
            DetailSource::ListEmbedded
        } else {
            DetailSource::Unavailable
        };

        bundles.push(MatchBundle::new(game_id, queue_id, game.clone(), detail_source));
    }

    (display_games, bundles, diagnostics)
}

/// 列表内的 game 是否已经等价于单局详情
fn list_game_has_full_detail(game: &Value) -> bool {
    let non_empty_array = |key: &str| {
        game.get(key)
            .and_then(Value::as_array)
            .is_some_and(|items| !items.is_empty())
    };

    non_empty_array("participants") && non_empty_array("participantIdentities")
}

fn collect_stats(bundles: &[MatchBundle]) -> FetchStats {
    let mut stats = FetchStats {
        list_requests: 1,
        ..FetchStats::default()
    };

    for bundle in bundles {
        if bundle.detail_source != DetailSource::ListEmbedded {
            stats.detail_requests += 1;
        }
        match bundle.timeline_status {
            TimelineStatus::Fetched | TimelineStatus::Failed => stats.timeline_requests += 1,
            TimelineStatus::CacheHit => stats.timeline_cache_hits += 1,
            TimelineStatus::Skipped => {}
        }
        if bundle.is_degraded() {
            stats.degraded_bundles += 1;
        }
    }

    stats
}
