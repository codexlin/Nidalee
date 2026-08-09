//! 统一数据获取层的输出类型
//!
//! `MatchBundle` 是「一场对局的全部原始素材」，后续的 Evidence 计算只消费它，
//! 不再自行发起任何请求。数据缺失通过状态 + 诊断显式表达，不用 panic / 整体失败。

use serde_json::Value;

/// 详情数据来源
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetailSource {
    /// 直接复用战绩列表内已有的完整对局（默认路径，零额外请求）
    ListEmbedded,
    /// 列表数据不完整，按需请求了 `/lol-match-history/v1/games/{id}`
    Fetched,
    /// 详情缺失（列表不完整且补齐失败）
    Unavailable,
}

/// 时间线获取状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelineStatus {
    /// 策略不允许（非排位 / Simple / 开关关闭），属于正常结果而非降级
    Skipped,
    /// 本轮真实请求获得
    Fetched,
    /// 命中会话缓存
    CacheHit,
    /// 请求失败，仅该局降级
    Failed,
}

/// 单局数据质量
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleQuality {
    /// 策略要求的数据齐全
    Complete,
    /// 存在缺失，结论需要相应收敛
    Degraded,
}

/// 出问题的环节
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchStage {
    List,
    Detail,
    Timeline,
}

/// 获取过程中的问题记录
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchDiagnostic {
    pub stage: FetchStage,
    pub game_id: u64,
    pub message: String,
}

impl FetchDiagnostic {
    pub fn new(stage: FetchStage, game_id: u64, message: impl Into<String>) -> Self {
        Self {
            stage,
            game_id,
            message: message.into(),
        }
    }
}

/// 一场对局的全部原始素材
#[derive(Debug, Clone)]
pub struct MatchBundle {
    pub game_id: u64,
    pub queue_id: i64,
    /// 战绩列表里的原始 game 节点
    pub list_game: Value,
    /// 按需补齐的单局详情；为 `None` 时应使用 `list_game`
    pub detail: Option<Value>,
    /// 时间线原始响应
    pub timeline: Option<Value>,
    pub detail_source: DetailSource,
    pub timeline_status: TimelineStatus,
    pub diagnostics: Vec<FetchDiagnostic>,
}

impl MatchBundle {
    pub fn new(game_id: u64, queue_id: i64, list_game: Value, detail_source: DetailSource) -> Self {
        Self {
            game_id,
            queue_id,
            list_game,
            detail: None,
            timeline: None,
            detail_source,
            timeline_status: TimelineStatus::Skipped,
            diagnostics: Vec::new(),
        }
    }

    /// 分析应当读取的对局 JSON：优先详情，回落列表节点
    pub fn game(&self) -> &Value {
        self.detail.as_ref().unwrap_or(&self.list_game)
    }

    pub fn has_timeline(&self) -> bool {
        self.timeline.is_some()
    }

    /// 是否发生了数据降级（策略性跳过不算降级）
    pub fn is_degraded(&self) -> bool {
        self.detail_source == DetailSource::Unavailable || self.timeline_status == TimelineStatus::Failed
    }

    pub fn quality(&self) -> BundleQuality {
        if self.is_degraded() {
            BundleQuality::Degraded
        } else {
            BundleQuality::Complete
        }
    }

    pub fn push_diagnostic(&mut self, stage: FetchStage, message: impl Into<String>) {
        self.diagnostics
            .push(FetchDiagnostic::new(stage, self.game_id, message));
    }
}

/// 一次批量获取的请求计数（可观测性 + 回归防线）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FetchStats {
    pub list_requests: usize,
    pub detail_requests: usize,
    pub timeline_requests: usize,
    pub timeline_cache_hits: usize,
    pub degraded_bundles: usize,
}

/// 批量获取结果
#[derive(Debug, Clone)]
pub struct MatchFetchOutcome {
    /// 原始列表响应（已按 count 本地截断），供既有链路继续使用
    pub raw_list: Value,
    /// 按策略队列过滤后的**全部展示对局**（受 `count` 约束）
    ///
    /// 与 `bundles` 的区别是这里不受 `effective_game_count` 截断：
    /// `max_analysis_games` 是「深度分析上限」，不是「少给用户几场战绩」的理由。
    pub display_games: Vec<Value>,
    /// 按列表原始顺序排列、已按策略过滤的对局素材（受 `effective_game_count` 约束）
    pub bundles: Vec<MatchBundle>,
    pub stats: FetchStats,
    /// 列表层面的问题（如缺少 gameId 的异常记录）
    pub diagnostics: Vec<FetchDiagnostic>,
    /// 队列过滤后展示场数仍不足用户请求量（已 over-fetch）
    pub insufficient_matches_in_scope: bool,
}
