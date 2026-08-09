//! 对局数据获取层的最窄公共门面
//!
//! `infrastructure` 保持 crate 私有，外部（集成测试）只能通过这里访问统一获取层，
//! 不暴露 LCU 连接、命令注册等其余基础设施细节。
//!
//! 这里只做重导出，不定义任何新语义。

pub use crate::infrastructure::match_management::matches::fetch_types::{
    BundleQuality, DetailSource, FetchDiagnostic, FetchStage, FetchStats, MatchBundle, MatchFetchOutcome,
    TimelineStatus,
};
pub use crate::infrastructure::match_management::matches::fetcher::{
    MatchDataSource, MatchFetcher, DEFAULT_MATCH_LIST_COUNT, DEFAULT_TIMELINE_CACHE_TTL, LCU_MATCH_LIST_MAX_COUNT,
    MAX_CONCURRENT_MATCH_FETCHES,
};
pub use crate::infrastructure::match_management::matches::timeline_cache::{Clock, SystemClock, TimelineCache};
