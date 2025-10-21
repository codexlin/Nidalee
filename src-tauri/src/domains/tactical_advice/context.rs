use super::types::AdvicePerspective;
use crate::domains::analysis::ParsedGame;
/// 建议分析上下文
///
/// 职责：
/// - 封装所有建议分析所需的数据
/// - 在责任链中传递
/// - 提供便捷的数据访问方法
use crate::shared::types::PlayerMatchStats;

/// 建议分析上下文
#[derive(Debug, Clone)]
pub struct AdviceContext {
    /// 玩家统计数据
    pub stats: PlayerMatchStats,

    /// 解析后的对局数据（含时间线）
    pub games: Vec<ParsedGame>,

    /// 主要位置
    pub role: String,

    /// 建议视角（自己/敌人/队友）
    pub perspective: AdvicePerspective,

    /// 目标玩家名称（针对/协作时使用）
    pub target_name: Option<String>,
}

impl AdviceContext {
    /// 创建新的上下文
    pub fn new(
        stats: PlayerMatchStats,
        games: Vec<ParsedGame>,
        role: String,
        perspective: AdvicePerspective,
        target_name: Option<String>,
    ) -> Self {
        Self {
            stats,
            games,
            role,
            perspective,
            target_name,
        }
    }

    /// 获取对局数量
    pub fn game_count(&self) -> usize {
        self.games.len()
    }

    /// 是否是改进建议（对自己）
    pub fn is_self_improvement(&self) -> bool {
        matches!(self.perspective, AdvicePerspective::SelfImprovement)
    }

    /// 是否是针对建议（对敌人）
    pub fn is_targeting(&self) -> bool {
        matches!(self.perspective, AdvicePerspective::Targeting)
    }

    /// 是否是协作建议（对队友）
    pub fn is_collaboration(&self) -> bool {
        matches!(self.perspective, AdvicePerspective::Collaboration)
    }

    /// 获取目标名称（用于描述）
    pub fn target_display_name(&self) -> String {
        match &self.target_name {
            Some(name) => name.clone(),
            None => match self.perspective {
                AdvicePerspective::SelfImprovement => "你".to_string(),
                AdvicePerspective::Targeting => "对手".to_string(),
                AdvicePerspective::Collaboration => "队友".to_string(),
            },
        }
    }
}
