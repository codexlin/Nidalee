//! 队列特定的配置和阈值
//!
//! 根据不同的队列类型（排位、大乱斗等）提供不同的分析策略和阈值。
//!
//! 这里是**队列语义的唯一来源**：其他模块（含 `evidence`）不得自建队列白名单。
//! 只有 catalog 里语义明确的队列才允许进入 [`QueueType`]，语义不明的一律落到
//! [`QueueType::Other`]，宁可判成「未知」也不要猜错模式。

use serde::{Deserialize, Serialize};

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
}
