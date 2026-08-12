//! 对局时间线的会话级缓存
//!
//! 约束：
//! - 只驻留内存，不落盘；进程退出即失效
//! - 只缓存成功响应，失败不写入，避免把一次网络抖动固化成长期空洞
//! - 不保存任何 LCU 认证信息（端口 / remoting token），键只有 gameId
//! - 时钟可注入，过期行为可在测试中确定性验证，无需 sleep
//! - 有界容量（默认 128）：超出时按 LRU 淘汰最久未访问条目

use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;

/// 默认容量上限（约覆盖一次选人全队深度分析的缓存需求）
pub const DEFAULT_TIMELINE_CACHE_CAPACITY: usize = 128;

/// 单调时钟抽象（毫秒），用于让 TTL 可测
pub trait Clock: Send + Sync {
    /// 自时钟创建以来经过的毫秒数
    fn now_ms(&self) -> u64;
}

/// 进程真实时钟
pub struct SystemClock {
    base: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self { base: Instant::now() }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now_ms(&self) -> u64 {
        self.base.elapsed().as_millis() as u64
    }
}

struct CacheEntry {
    timeline: Value,
    stored_at_ms: u64,
}

struct CacheState {
    entries: HashMap<u64, CacheEntry>,
    /// 最近访问在尾部
    order: VecDeque<u64>,
}

/// 按 gameId 缓存时间线的线程安全 TTL + LRU 缓存
pub struct TimelineCache {
    ttl_ms: u64,
    capacity: usize,
    clock: Arc<dyn Clock>,
    state: Mutex<CacheState>,
}

impl TimelineCache {
    pub fn new(ttl: Duration) -> Self {
        Self::with_capacity(ttl, DEFAULT_TIMELINE_CACHE_CAPACITY)
    }

    pub fn with_capacity(ttl: Duration, capacity: usize) -> Self {
        Self::with_clock_and_capacity(ttl, capacity, Arc::new(SystemClock::new()))
    }

    pub fn with_clock(ttl: Duration, clock: Arc<dyn Clock>) -> Self {
        Self::with_clock_and_capacity(ttl, DEFAULT_TIMELINE_CACHE_CAPACITY, clock)
    }

    pub fn with_clock_and_capacity(ttl: Duration, capacity: usize, clock: Arc<dyn Clock>) -> Self {
        Self {
            ttl_ms: ttl.as_millis() as u64,
            capacity: capacity.max(1),
            clock,
            state: Mutex::new(CacheState {
                entries: HashMap::new(),
                order: VecDeque::new(),
            }),
        }
    }

    /// 读取未过期的时间线；命中过期条目时顺带清除
    pub fn get(&self, game_id: u64) -> Option<Value> {
        let now = self.clock.now_ms();
        let mut state = self.lock();

        let hit = state
            .entries
            .get(&game_id)
            .filter(|entry| now.saturating_sub(entry.stored_at_ms) < self.ttl_ms)
            .map(|entry| entry.timeline.clone());

        if hit.is_some() {
            touch_order(&mut state.order, game_id);
            return hit;
        }

        if state.entries.remove(&game_id).is_some() {
            remove_from_order(&mut state.order, game_id);
        }
        None
    }

    /// 写入成功响应；超出容量时淘汰最久未访问条目
    pub fn insert(&self, game_id: u64, timeline: Value) {
        let stored_at_ms = self.clock.now_ms();
        let mut state = self.lock();

        if let Some(entry) = state.entries.get_mut(&game_id) {
            *entry = CacheEntry { timeline, stored_at_ms };
            touch_order(&mut state.order, game_id);
            return;
        }

        while state.entries.len() >= self.capacity {
            if let Some(evict_id) = state.order.pop_front() {
                state.entries.remove(&evict_id);
            } else {
                break;
            }
        }

        state.entries.insert(game_id, CacheEntry { timeline, stored_at_ms });
        state.order.push_back(game_id);
    }

    pub fn clear(&self) {
        let mut state = self.lock();
        state.entries.clear();
        state.order.clear();
    }

    pub fn len(&self) -> usize {
        self.lock().entries.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, CacheState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn touch_order(order: &mut VecDeque<u64>, game_id: u64) {
    remove_from_order(order, game_id);
    order.push_back(game_id);
}

fn remove_from_order(order: &mut VecDeque<u64>, game_id: u64) {
    if let Some(index) = order.iter().position(|id| *id == game_id) {
        order.remove(index);
    }
}

impl fmt::Debug for TimelineCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimelineCache")
            .field("ttl_ms", &self.ttl_ms)
            .field("capacity", &self.capacity)
            .field("entries", &self.len())
            .finish()
    }
}
