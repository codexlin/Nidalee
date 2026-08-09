//! 对局分析应用服务的最窄公共门面
//!
//! `infrastructure` 保持 crate 私有，外部（集成测试）只能通过这里访问
//! 「请求 → 策略 → 获取 → 编排」这条唯一执行路径，不暴露 LCU 连接、命令注册等细节。
//!
//! 这里只做重导出，不定义任何新语义。

pub use crate::infrastructure::match_management::matches::analysis_service::{
    analyze_matches_with_fetcher, legacy_analysis_request, legacy_overview_request, tactical_advice_request,
    to_legacy_advice, to_multi_position_analysis, to_player_match_stats, LEGACY_MAX_ANALYSIS_GAMES,
    TACTICAL_ADVICE_GAME_COUNT,
};
