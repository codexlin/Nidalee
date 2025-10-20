use super::super::types::{GameAdvice, AdvicePerspective};

/// 问题数据（传递给策略）
#[derive(Debug, Clone)]
pub struct ProblemData {
    /// 问题严重程度（0.0-1.0）
    pub severity: f64,

    /// 相关数值（如 CS差、胜率等）
    pub value: f64,

    /// 位置
    pub role: String,

    /// 目标玩家名称
    pub target_name: Option<String>,

    /// 额外描述
    pub extra_info: Option<String>,
}

/// 问题类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProblemType {
    // 对线期问题
    LaningCsDeficit,      // 补刀落后
    LaningDominated,      // 被压制

    // 发育问题
    MidGameDecline,       // 中期经济下降
    PoorFarming,          // 补刀效率低

    // 团战问题
    LowKillParticipation,       // 参团率低（助攻少）
    LowTeamfightParticipation,  // 低参团率（综合）
    HighDeathRate,              // 死亡过多
    PoorPositioning,            // 站位问题

    // 视野问题
    LowVisionScore,       // 视野得分低

    // 英雄池问题
    ChampionPoolNarrow,   // 英雄池窄
    ChampionDependency,   // 过度依赖单一英雄
}

/// 建议生成策略 trait
pub trait AdviceStrategy: Send + Sync {
    /// 生成建议
    ///
    /// 根据问题类型和数据，生成对应视角的建议
    fn generate_advice(
        &self,
        problem_type: ProblemType,
        data: &ProblemData,
    ) -> Option<GameAdvice>;

    /// 获取策略名称
    fn name(&self) -> &str;

    /// 获取视角
    fn perspective(&self) -> AdvicePerspective;
}

