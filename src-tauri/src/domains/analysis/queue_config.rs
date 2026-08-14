//! 队列特定的配置和阈值
//!
//! 根据不同的队列类型（排位、大乱斗等）提供不同的分析策略和阈值。
//!
//! 这里是**队列语义的唯一来源**：其他模块（含 `evidence`）不得自建队列白名单。
//! 只有 catalog 里语义明确的队列才允许进入 [`QueueType`]，语义不明的一律落到
//! [`QueueType::Other`]，宁可判成「未知」也不要猜错模式。

use serde::{Deserialize, Serialize};

use crate::shared::types::RankedRatingQueue;

/// 实时对局卡片的分析策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisProfile {
    /// 仅使用该排位队列的历史进行深度分析。
    Ranked(RankedRatingQueue),
    /// 非竞技队列只产出近期轻量摘要。
    Simple,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QueueType {
    SoloRanked,
    FlexRanked,
    Aram,
    Urf,
    Normal,
    Other,
}

impl QueueType {
    pub fn from_queue_id(queue_id: i32) -> Self {
        match queue_id {
            420 => Self::SoloRanked,
            440 => Self::FlexRanked,
            450 => Self::Aram,
            900 => Self::Urf,
            400 | 430 | 490 => Self::Normal,
            _ => Self::Other,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::SoloRanked => "单双排",
            Self::FlexRanked => "灵活组排",
            Self::Aram => "大乱斗",
            Self::Urf => "无限火力",
            Self::Normal => "匹配模式",
            Self::Other => "其他模式",
        }
    }

    pub fn is_ranked(self) -> bool {
        matches!(self, Self::SoloRanked | Self::FlexRanked)
    }

    pub fn is_aram(self) -> bool {
        matches!(self, Self::Aram)
    }

    pub fn analysis_profile(self) -> AnalysisProfile {
        match self {
            Self::SoloRanked => AnalysisProfile::Ranked(RankedRatingQueue::SoloRanked),
            Self::FlexRanked => AnalysisProfile::Ranked(RankedRatingQueue::FlexRanked),
            Self::Aram | Self::Urf | Self::Normal | Self::Other => AnalysisProfile::Simple,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AnalysisProfile, QueueType};
    use crate::shared::types::RankedRatingQueue;

    #[test]
    fn only_solo_and_flex_should_use_ranked_analysis() {
        assert_eq!(
            QueueType::from_queue_id(420).analysis_profile(),
            AnalysisProfile::Ranked(RankedRatingQueue::SoloRanked)
        );
        assert_eq!(
            QueueType::from_queue_id(440).analysis_profile(),
            AnalysisProfile::Ranked(RankedRatingQueue::FlexRanked)
        );
    }

    #[test]
    fn non_ranked_queues_should_use_simple_analysis() {
        for queue_id in [0, 400, 430, 450, 490, 900, 2400] {
            assert_eq!(
                QueueType::from_queue_id(queue_id).analysis_profile(),
                AnalysisProfile::Simple,
                "queue {queue_id} unexpectedly enabled ranked analysis"
            );
        }
    }
}
