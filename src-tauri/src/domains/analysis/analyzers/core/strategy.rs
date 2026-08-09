/// 分析深度枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisDepth.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisDepth {
    /// 简单分析 - 2层分析（基础 + 简单胜负）
    Simple,
    /// 深度分析 - 5层分析（基础 + 深度 + 位置 + 分布 + 胜负）
    Deep,
}

/// 分析模式（由前端用户选择）
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ts_rs::TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisMode.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisMode {
    /// 单排分析 - 只分析单排对局（420）
    SoloRanked,
    /// 灵活组排分析 - 只分析灵活组排对局（440）
    FlexRanked,
    /// 混合排位分析 - 分析单排+灵活组排对局（420+440）
    MixedRanked,
    /// 大乱斗分析 - 只分析大乱斗对局（450）；保留兼容，新产品用 Normals
    Aram,
    /// 普通模式 - 排除排位（420/440），含匹配 / 大乱斗 / 娱乐局等
    Normals,
    /// 全部模式分析 - 分析所有对局
    AllModes,
}

impl AnalysisMode {
    /// 获取模式描述
    pub fn description(&self) -> &'static str {
        match self {
            AnalysisMode::SoloRanked => "单排分析",
            AnalysisMode::FlexRanked => "灵活组排分析",
            AnalysisMode::MixedRanked => "混合排位分析",
            AnalysisMode::Aram => "大乱斗分析",
            AnalysisMode::Normals => "普通模式分析",
            AnalysisMode::AllModes => "全部模式分析",
        }
    }

    /// 是否排除排位队列（普通模式）
    pub fn excludes_ranked(&self) -> bool {
        matches!(self, AnalysisMode::Normals)
    }

    /// 获取对应的分析策略（基于深度设置）
    pub fn to_analysis_strategy(&self, depth: AnalysisDepth) -> AnalysisStrategy {
        match (self, depth) {
            // 深度分析模式
            (AnalysisMode::SoloRanked, AnalysisDepth::Deep) => AnalysisStrategy::SoloRanked,
            (AnalysisMode::FlexRanked, AnalysisDepth::Deep) => AnalysisStrategy::FlexRanked,
            (AnalysisMode::MixedRanked, AnalysisDepth::Deep) => AnalysisStrategy::MixedRanked,

            // 简单分析模式（所有模式都使用简化策略）
            (AnalysisMode::SoloRanked, AnalysisDepth::Simple) => AnalysisStrategy::Other,
            (AnalysisMode::FlexRanked, AnalysisDepth::Simple) => AnalysisStrategy::Other,
            (AnalysisMode::MixedRanked, AnalysisDepth::Simple) => AnalysisStrategy::Other,
            (AnalysisMode::Aram, _) => AnalysisStrategy::Other,
            (AnalysisMode::Normals, _) => AnalysisStrategy::Other,
            (AnalysisMode::AllModes, _) => AnalysisStrategy::Other,
        }
    }

    /// 获取默认分析策略（保持向后兼容）
    pub fn to_analysis_strategy_default(&self) -> AnalysisStrategy {
        self.to_analysis_strategy(AnalysisDepth::Deep)
    }

    /// 获取需要过滤的队列ID列表（允许列表）；空 = 不过滤或由 excludes_ranked 决定
    pub fn queue_ids(&self) -> Vec<i64> {
        match self {
            AnalysisMode::SoloRanked => vec![420],
            AnalysisMode::FlexRanked => vec![440],
            AnalysisMode::MixedRanked => vec![420, 440],
            AnalysisMode::Aram => vec![450],
            AnalysisMode::Normals => vec![], // 语义为排除排位，见 AnalysisPolicy::exclude_ranked
            AnalysisMode::AllModes => vec![], // 空列表表示不过滤
        }
    }
}

use super::parser::ParsedGame;

/// 分析策略（决定分析深度）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisStrategy {
    /// 单排模式 - 完整深度分析
    /// - 5层分析：基础 → 深度 → 位置 → 分布 → 胜负
    /// - 位置识别
    /// - 稳定性分析
    /// - 趋势分析
    /// - 高光/崩盘识别
    SoloRanked,

    /// 灵活组排模式 - 完整深度分析
    /// - 5层分析：基础 → 深度 → 位置 → 分布 → 胜负
    /// - 位置识别
    /// - 稳定性分析
    /// - 趋势分析
    /// - 高光/崩盘识别
    FlexRanked,

    /// 混合排位模式 - 完整深度分析（单排+灵活）
    /// - 5层分析：基础 → 深度 → 位置 → 分布 → 胜负
    /// - 位置识别
    /// - 稳定性分析
    /// - 趋势分析
    /// - 高光/崩盘识别
    MixedRanked,

    /// 其他模式 - 简化快速分析
    /// - 2层分析：基础 → 简单胜负
    /// - 基础统计（胜率、KDA、连胜）
    /// - 简单评价
    Other,
}

impl AnalysisStrategy {
    /// 根据队列ID自动选择策略
    pub fn from_queue_id(queue_id: i64) -> Self {
        match queue_id {
            420 => AnalysisStrategy::SoloRanked, // 单排 - 核心模式
            440 => AnalysisStrategy::FlexRanked, // 灵活组排 - 核心模式
            _ => AnalysisStrategy::Other,        // 其他所有模式 - 简化分析
        }
    }

    /// 根据对局列表自动推断策略（取主要模式）
    pub fn from_games(games: &[ParsedGame]) -> Self {
        if games.is_empty() {
            return AnalysisStrategy::Other;
        }

        // 统计各队列的场次
        let mut queue_counts: std::collections::HashMap<i64, u32> = std::collections::HashMap::new();
        for game in games {
            *queue_counts.entry(game.queue_id).or_insert(0) += 1;
        }

        // 检查是否包含排位赛数据
        let solo_games = queue_counts.get(&420).copied().unwrap_or(0);
        let flex_games = queue_counts.get(&440).copied().unwrap_or(0);
        // let total_ranked_games = solo_games + flex_games; // 暂时注释掉未使用的变量

        // 如果同时包含单排和灵活组排，使用混合模式
        if solo_games > 0 && flex_games > 0 {
            return AnalysisStrategy::MixedRanked;
        }

        // 如果只有一种排位模式，使用对应策略
        if solo_games > 0 {
            return AnalysisStrategy::SoloRanked;
        }
        if flex_games > 0 {
            return AnalysisStrategy::FlexRanked;
        }

        // 其他情况，找出最多的队列
        if let Some((main_queue, _)) = queue_counts.iter().max_by_key(|(_, count)| *count) {
            Self::from_queue_id(*main_queue)
        } else {
            AnalysisStrategy::Other
        }
    }

    /// 是否启用深度分析（参团率、伤害占比、稳定性、趋势）
    pub fn enable_advanced_analysis(&self) -> bool {
        matches!(
            self,
            AnalysisStrategy::SoloRanked | AnalysisStrategy::FlexRanked | AnalysisStrategy::MixedRanked
        )
    }

    /// 是否启用位置分析（位置识别、位置专项评价）
    pub fn enable_role_analysis(&self) -> bool {
        matches!(
            self,
            AnalysisStrategy::SoloRanked | AnalysisStrategy::FlexRanked | AnalysisStrategy::MixedRanked
        )
    }

    /// 是否启用分布分析（高光时刻、崩盘场次、稳定性分布）
    pub fn enable_distribution_analysis(&self) -> bool {
        matches!(
            self,
            AnalysisStrategy::SoloRanked | AnalysisStrategy::FlexRanked | AnalysisStrategy::MixedRanked
        )
    }

    /// 获取特征数量限制
    pub fn max_traits(&self) -> usize {
        match self {
            AnalysisStrategy::SoloRanked | AnalysisStrategy::FlexRanked | AnalysisStrategy::MixedRanked => 12, // 排位模式：展示更多特征
            AnalysisStrategy::Other => 6, // 其他模式：精简特征
        }
    }

    /// 获取分析层次描述（用于日志）
    pub fn analysis_layers(&self) -> &str {
        match self {
            AnalysisStrategy::SoloRanked => "5层完整分析（基础→深度→位置→分布→胜负）",
            AnalysisStrategy::FlexRanked => "5层完整分析（基础→深度→位置→分布→胜负）",
            AnalysisStrategy::MixedRanked => "5层完整分析（基础→深度→位置→分布→胜负）",
            AnalysisStrategy::Other => "2层简化分析（基础→简单胜负）",
        }
    }

    /// 获取策略描述
    pub fn description(&self) -> &str {
        match self {
            AnalysisStrategy::SoloRanked => "单排模式 - 核心深度分析",
            AnalysisStrategy::FlexRanked => "灵活组排模式 - 核心深度分析",
            AnalysisStrategy::MixedRanked => "混合排位模式 - 核心深度分析（单排+灵活）",
            AnalysisStrategy::Other => "娱乐模式 - 快速简化分析",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::analysis::analyzers::core::parser::{ParsedPlayerData, ParsedTeamData};

    /// 只关心 `queue_id` 的最小 `ParsedGame`
    ///
    /// 刻意不给 `ParsedPlayerData` / `ParsedTeamData` 加 `derive(Default)`：
    /// 「一场全零的对局」在生产语义里没有意义，能被 `Default` 造出来只会让
    /// 忘记赋值的 bug 变成静默的错误统计。测试需要占位值就在测试里显式写。
    fn game_with_queue(game_id: u64, queue_id: i64) -> ParsedGame {
        ParsedGame {
            game_id,
            queue_id,
            game_duration: 1800,
            game_creation: 1234567890,
            player_data: ParsedPlayerData {
                participant_id: 1,
                champion_id: 0,
                team_id: 100,
                win: true,
                kills: 0,
                deaths: 0,
                assists: 0,
                kda: 0.0,
                damage_to_champions: 0,
                damage_taken: 0,
                gold_earned: 0,
                vision_score: 0,
                wards_placed: 0,
                wards_killed: 0,
                cs: 0,
                jungle_cs: 0,
                role: "SOLO".to_string(),
                lane: "TOP".to_string(),
                timeline_data: None,
            },
            team_data: ParsedTeamData {
                team_id: 100,
                total_kills: 0,
                total_deaths: 0,
                total_assists: 0,
                total_damage_to_champions: 0,
                total_damage_taken: 0,
                total_gold_earned: 0,
                total_vision_score: 0,
                total_cs: 0,
            },
        }
    }

    #[test]
    fn test_solo_ranked_strategy() {
        let strategy = AnalysisStrategy::from_queue_id(420);
        assert_eq!(strategy, AnalysisStrategy::SoloRanked);
        assert!(strategy.enable_advanced_analysis());
        assert!(strategy.enable_role_analysis());
        assert!(strategy.enable_distribution_analysis());
        assert_eq!(strategy.max_traits(), 12);
    }

    #[test]
    fn test_flex_ranked_strategy() {
        let strategy = AnalysisStrategy::from_queue_id(440);
        assert_eq!(strategy, AnalysisStrategy::FlexRanked);
        assert!(strategy.enable_advanced_analysis());
        assert!(strategy.enable_role_analysis());
        assert!(strategy.enable_distribution_analysis());
        assert_eq!(strategy.max_traits(), 12);
    }

    #[test]
    fn test_mixed_ranked_strategy() {
        // 创建包含单排和灵活组排的游戏数据
        let games = vec![game_with_queue(1, 420), game_with_queue(2, 440)];
        let strategy = AnalysisStrategy::from_games(&games);
        assert_eq!(strategy, AnalysisStrategy::MixedRanked);
        assert!(strategy.enable_advanced_analysis());
        assert!(strategy.enable_role_analysis());
        assert!(strategy.enable_distribution_analysis());
        assert_eq!(strategy.max_traits(), 12);
    }

    #[test]
    fn test_other_strategy() {
        let strategy = AnalysisStrategy::from_queue_id(450); // ARAM
        assert_eq!(strategy, AnalysisStrategy::Other);
        assert!(!strategy.enable_advanced_analysis());
        assert!(!strategy.enable_role_analysis());
        assert!(!strategy.enable_distribution_analysis());
        assert_eq!(strategy.max_traits(), 6);
    }
}
