/// 统一对局分析契约（请求 / 策略 / 结果 / 能力 / 降级诊断）
///
/// 设计要点：
/// - 请求（`MatchAnalysisRequest`）只描述「用户想要什么」，不包含任何推导结论
/// - 策略（`AnalysisPolicy`）是唯一的推导产物，由 `policy::resolve_analysis_policy` 生成
/// - 结果（`MatchAnalysisResult`）复用现有统计结构（`PlayerMatchStats` / `PositionStats`），
///   不重复定义统计字段，只补充能力声明、降级诊断与后续扩展位
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::domains::ai_analysis::AiInsight;
use crate::domains::analysis::evidence::{EvidenceBundle, EvidenceConfidence};
use crate::domains::analysis::queue_config::QueueType;
use crate::shared::types::{
    AdviceCategory, AdvicePerspective, GameAdvice, MatchPerformance, PlayerMatchStats, PositionStats, SummonerTrait,
};

// 复用现有分析模式/深度枚举，避免出现第二套语义；同时对外暴露给 pipeline 使用方
pub use crate::domains::analysis::analyzers::core::strategy::{AnalysisDepth, AnalysisMode};

/// 默认分析对局数（与前端 analysisSettingsStore 默认值保持一致）
pub const DEFAULT_ANALYSIS_GAME_COUNT: u32 = 20;

/// 位置未知时的占位码（与 EvidencePosition::Unknown 一致；中文展示由前端负责）
pub const UNKNOWN_POSITION: &str = "UNKNOWN";

/// 是否为排位队列（420 单双 / 440 灵活）
///
/// 复用 `QueueType` 的判定，保证队列语义只有一处定义
pub fn is_ranked_queue(queue_id: i64) -> bool {
    QueueType::from_queue_id(queue_id as i32).is_ranked()
}

/// 分析功能开关（对应前端 AnalysisConfig 的开关项）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisFeatureFlags.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisFeatureFlags {
    /// 智能分析总开关：false 时只产出基础统计
    pub enabled: bool,
    /// 时间线分析（分阶段 CS/经济/经验差）
    pub timeline: bool,
    /// 对手分析
    pub opponent: bool,
    /// 队友分析
    pub teammate: bool,
    /// 自我提升分析
    pub self_improvement: bool,
}

impl Default for AnalysisFeatureFlags {
    fn default() -> Self {
        Self {
            enabled: true,
            timeline: true,
            opponent: true,
            teammate: true,
            self_improvement: true,
        }
    }
}

impl AnalysisFeatureFlags {
    /// 全部关闭（总开关也关闭），用于「只要基础统计」的场景
    pub fn all_disabled() -> Self {
        Self {
            enabled: false,
            timeline: false,
            opponent: false,
            teammate: false,
            self_improvement: false,
        }
    }
}

/// 统一对局分析请求
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/MatchAnalysisRequest.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct MatchAnalysisRequest {
    /// 期望拉取的对局数量
    pub count: u32,

    /// 分析模式（决定默认队列过滤）
    pub mode: AnalysisMode,

    /// 用户请求的分析深度
    pub depth: AnalysisDepth,

    /// 单队列覆盖（优先级低于 `queue_ids`）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number")]
    pub queue_id: Option<i64>,

    /// 多队列覆盖；`Some(vec![])` 表示显式不过滤
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional, type = "number[]")]
    pub queue_ids: Option<Vec<i64>>,

    /// 功能开关
    #[serde(default)]
    pub features: AnalysisFeatureFlags,

    /// 性能上限：实际参与分析的最大对局数
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub max_analysis_games: Option<u32>,

    /// 建议视角（自我改进 / 针对对手 / 协作队友）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub perspective: Option<AdvicePerspective>,

    /// 建议措辞里的目标玩家名（针对对手 / 协作队友时使用）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_player: Option<String>,
}

impl Default for MatchAnalysisRequest {
    fn default() -> Self {
        Self {
            count: DEFAULT_ANALYSIS_GAME_COUNT,
            mode: AnalysisMode::AllModes,
            depth: AnalysisDepth::Deep,
            queue_id: None,
            queue_ids: None,
            features: AnalysisFeatureFlags::default(),
            max_analysis_games: None,
            perspective: None,
            target_player: None,
        }
    }
}

impl MatchAnalysisRequest {
    /// 构造请求（其余字段取默认值）
    pub fn new(mode: AnalysisMode, depth: AnalysisDepth) -> Self {
        Self {
            mode,
            depth,
            ..Default::default()
        }
    }

    /// 请求侧的队列过滤集合
    ///
    /// 优先级：`queue_ids` > `queue_id` > 模式默认队列。
    /// 返回空切片表示「不过滤」。
    pub fn requested_queue_ids(&self) -> Vec<i64> {
        if let Some(ids) = &self.queue_ids {
            return ids.clone();
        }
        if let Some(id) = self.queue_id {
            return vec![id];
        }
        self.mode.queue_ids()
    }

    /// 受性能上限约束后的实际分析对局数
    ///
    /// 只约束**深度证据**，不约束展示场次：展示与基础统计一律用 `count`。
    pub fn effective_game_count(&self) -> u32 {
        match self.max_analysis_games {
            Some(max) => self.count.min(max),
            None => self.count,
        }
    }

    /// 建议视角（缺省为自我提升）
    pub fn effective_perspective(&self) -> AdvicePerspective {
        self.perspective.unwrap_or(AdvicePerspective::SelfImprovement)
    }
}

/// 队列范围（由最终选中的队列集合推导）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisQueueScope.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisQueueScope {
    /// 不过滤队列（全部模式且无覆盖）
    Unfiltered,
    /// 只包含排位队列（420 / 440）
    RankedOnly,
    /// 只包含非排位队列（娱乐 / 匹配等）
    NonRankedOnly,
    /// 排位与非排位混合
    Mixed,
}

impl AnalysisQueueScope {
    /// 由选中的队列集合推导范围；空集合表示不过滤
    pub fn from_queue_ids(queue_ids: &[i64]) -> Self {
        if queue_ids.is_empty() {
            return AnalysisQueueScope::Unfiltered;
        }

        let ranked = queue_ids.iter().any(|id| is_ranked_queue(*id));
        let non_ranked = queue_ids.iter().any(|id| !is_ranked_queue(*id));

        match (ranked, non_ranked) {
            (true, true) => AnalysisQueueScope::Mixed,
            (true, false) => AnalysisQueueScope::RankedOnly,
            _ => AnalysisQueueScope::NonRankedOnly,
        }
    }

    /// 是否可能包含排位对局（用于判断深度结论/AI 证据的可行性）
    pub fn may_contain_ranked(&self) -> bool {
        !matches!(self, AnalysisQueueScope::NonRankedOnly)
    }
}

/// 受降级影响的功能项
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisFeature.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisFeature {
    DeepAnalysis,
    Timeline,
    Opponent,
    Teammate,
    SelfImprovement,
    LocalAi,
    GameCount,
}

/// 降级原因码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisDegradationCode.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisDegradationCode {
    /// 智能分析总开关关闭
    AnalysisDisabled,
    /// 用户请求了简单深度
    SimpleDepthRequested,
    /// 娱乐模式不支持深度分析，已降级为简单
    FunModeDeepUnsupported,
    /// 混合/全部模式采用按局策略（排位深度、娱乐简单）
    PerGameDepthApplied,
    /// 时间线被功能开关关闭
    TimelineDisabledByFeatureFlag,
    /// 对手分析被功能开关关闭
    OpponentDisabledByFeatureFlag,
    /// 队友分析被功能开关关闭
    TeammateDisabledByFeatureFlag,
    /// 自我提升分析被功能开关关闭
    SelfImprovementDisabledByFeatureFlag,
    /// 分析对局数被性能上限截断
    GameCountCapped,
    /// 本地 AI 需要排位深度证据
    LocalAiRequiresRankedDeepEvidence,
    /// 策略选中的对局里没有一场能提取证据
    NoMatchesInScope,
    /// 队列过滤后可展示场数不足请求量（已 over-fetch 至 LCU 上限仍不够）
    InsufficientMatchesInScope,
    /// 单局证据提取失败，已跳过该局
    EvidenceExtractionFailed,
    /// 时间线不可用（请求失败 / 响应缺帧），分阶段结论已收敛
    TimelineDataUnavailable,
    /// 样本量不足以支撑结论，只保留描述
    InsufficientEvidenceSample,
}

/// 降级诊断（用于向用户解释「为什么没有这个结论」）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisDiagnostic.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisDiagnostic {
    /// 原因码（前端可据此本地化/分组）
    pub code: AnalysisDegradationCode,
    /// 人类可读原因
    pub message: String,
    /// 受影响的功能项
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub feature: Option<AnalysisFeature>,
}

impl AnalysisDiagnostic {
    pub fn new(code: AnalysisDegradationCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            feature: None,
        }
    }

    pub fn with_feature(code: AnalysisDegradationCode, feature: AnalysisFeature, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            feature: Some(feature),
        }
    }
}

/// 分析策略（唯一推导产物）
///
/// 由 `policy::resolve_analysis_policy` 生成，下游 fetcher / orchestrator / AI
/// 只读取该结构，不再各自判断模式与深度。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisPolicy.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisPolicy {
    /// 请求的分析模式
    pub mode: AnalysisMode,

    /// 用户请求的深度
    pub requested_depth: AnalysisDepth,

    /// 实际生效的深度（可能因娱乐模式/开关而降级）
    pub effective_depth: AnalysisDepth,

    /// 最终选中的队列集合；空表示不过滤
    #[ts(type = "number[]")]
    pub selected_queue_ids: Vec<i64>,

    /// 队列范围
    pub queue_scope: AnalysisQueueScope,

    /// 实际参与分析的对局数（已应用性能上限）
    pub effective_game_count: u32,

    /// 只做基础统计（不做深度层）
    pub basic_only: bool,

    /// 按局决定深度（排位 deep / 娱乐 simple）
    pub per_game_depth: bool,

    /// 深度结论只能来自排位对局
    pub deep_conclusions_ranked_only: bool,

    pub enable_timeline: bool,
    pub enable_opponent: bool,
    pub enable_teammate: bool,
    pub enable_self_improvement: bool,

    /// 本地 AI 是否具备可用前提（仍需运行期存在排位深度证据）
    pub local_ai_eligible: bool,

    /// 降级诊断
    pub diagnostics: Vec<AnalysisDiagnostic>,
}

impl AnalysisPolicy {
    /// 是否包含某个诊断码
    pub fn has_diagnostic(&self, code: AnalysisDegradationCode) -> bool {
        self.diagnostics.iter().any(|d| d.code == code)
    }

    /// 该队列是否参与分析
    pub fn includes_queue(&self, queue_id: i64) -> bool {
        self.selected_queue_ids.is_empty() || self.selected_queue_ids.contains(&queue_id)
    }

    /// 单局适用的深度
    pub fn depth_for_queue(&self, queue_id: i64) -> AnalysisDepth {
        if self.basic_only {
            return AnalysisDepth::Simple;
        }
        if self.per_game_depth && !is_ranked_queue(queue_id) {
            return AnalysisDepth::Simple;
        }
        self.effective_depth
    }

    /// 单局是否启用时间线（时间线只在深度局生效）
    pub fn timeline_enabled_for_queue(&self, queue_id: i64) -> bool {
        self.enable_timeline && self.depth_for_queue(queue_id) == AnalysisDepth::Deep
    }
}

/// 结果能力声明（前端据此决定展示什么、隐藏什么）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AnalysisCapabilities.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisCapabilities {
    /// 基础统计始终可用
    pub basic_stats: bool,
    /// 位置维度的深度拆分是否可用
    pub position_breakdown: bool,
    /// 深度层结论是否可用
    pub deep_analysis: bool,
    pub timeline: bool,
    pub opponent: bool,
    pub teammate: bool,
    pub self_improvement: bool,
    /// 是否存在（或可能存在）排位深度证据
    pub ranked_deep_evidence: bool,
    /// 本地 AI 能力是否可用
    pub local_ai: bool,
}

impl AnalysisCapabilities {
    /// 由策略推导「潜在能力」
    ///
    /// 对于全部/混合模式，`ranked_deep_evidence` 属于潜在值，
    /// 需在拿到真实对局后调用 `refined_with_observed_queues` 收敛。
    pub fn from_policy(policy: &AnalysisPolicy) -> Self {
        let deep_analysis = !policy.basic_only && policy.effective_depth == AnalysisDepth::Deep;
        let ranked_deep_evidence = deep_analysis && policy.queue_scope.may_contain_ranked();

        Self {
            basic_stats: true,
            // 位置拆分（英雄池/趋势）在 Simple 也会产出；最终由编排器用实际 positionStats 收敛
            position_breakdown: true,
            deep_analysis,
            // 深度子能力：开关打开且具备深度前提；最终由编排器用实际证据收敛
            timeline: policy.enable_timeline && deep_analysis,
            opponent: policy.enable_opponent && deep_analysis,
            teammate: policy.enable_teammate && deep_analysis,
            self_improvement: policy.enable_self_improvement && deep_analysis,
            ranked_deep_evidence,
            local_ai: policy.local_ai_eligible && ranked_deep_evidence,
        }
    }

    /// 用实际观察到的队列收敛能力（本地 AI 只对排位深度证据可用）
    pub fn refined_with_observed_queues(mut self, observed_queue_ids: &[i64]) -> Self {
        let has_ranked = observed_queue_ids.iter().any(|id| is_ranked_queue(*id));
        self.ranked_deep_evidence = self.ranked_deep_evidence && has_ranked;
        self.local_ai = self.local_ai && self.ranked_deep_evidence;
        self
    }
}

/// 特征倾向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/TraitSentiment.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub enum TraitSentiment {
    Good,
    Bad,
    /// 处在中间带，或样本量不足以定性
    Neutral,
}

impl TraitSentiment {
    /// 旧 `SummonerTrait.type` 的取值（前端仍按 good/bad 上色）
    fn legacy_type(&self) -> &'static str {
        match self {
            TraitSentiment::Good => "good",
            TraitSentiment::Bad => "bad",
            TraitSentiment::Neutral => "neutral",
        }
    }

    /// 旧 `SummonerTrait.score`（1-10）
    fn legacy_score(&self) -> i32 {
        match self {
            TraitSentiment::Good => 8,
            TraitSentiment::Bad => 4,
            TraitSentiment::Neutral => 6,
        }
    }
}

/// 确定性特征
///
/// 由特征策略产出（排位 Evidence / 娱乐 ParsedGame / 模式身份）。
/// 每条都必须能回指对局：`sample_count` / `frequency` / `confidence` / `evidence_game_ids` 必填。
/// 样本量不足时 `supports_conclusion == false`，此时只允许描述事实，不允许定性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/DeterministicTrait.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct DeterministicTrait {
    /// 稳定标识（前端本地化 / 分组的键）
    pub key: String,
    pub name: String,
    pub description: String,
    pub sentiment: TraitSentiment,
    /// 参与该特征计算的对局数
    pub sample_count: u32,
    /// 样本中符合该结论的占比，取值 `0.0..=1.0`
    pub frequency: f64,
    pub confidence: EvidenceConfidence,
    /// 样本量是否足以支撑结论
    pub supports_conclusion: bool,
    /// 支撑该特征的对局 ID（升序）
    #[ts(type = "number[]")]
    pub evidence_game_ids: Vec<u64>,
    /// 位置维度（`None` 表示全局）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub position: Option<String>,
}

impl DeterministicTrait {
    /// 映射为旧的 `SummonerTrait`（兼容存量前端）
    pub fn to_legacy_trait(&self) -> SummonerTrait {
        SummonerTrait {
            name: self.name.clone(),
            description: self.description.clone(),
            score: self.sentiment.legacy_score(),
            trait_type: self.sentiment.legacy_type().to_string(),
        }
    }
}

/// 确定性建议
///
/// 只在对应特征「可下结论」时产生，因此必然带着样本量与证据对局。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/DeterministicAdvice.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct DeterministicAdvice {
    /// 与来源特征相同的稳定标识
    pub key: String,
    pub title: String,
    pub problem: String,
    /// 可核对的证据描述（含具体数值与样本量）
    pub evidence: String,
    pub suggestions: Vec<String>,
    pub priority: i32,
    pub category: AdviceCategory,
    pub perspective: AdvicePerspective,
    pub sample_count: u32,
    pub confidence: EvidenceConfidence,
    #[ts(type = "number[]")]
    pub evidence_game_ids: Vec<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub position: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub target_player: Option<String>,
}

impl DeterministicAdvice {
    /// 映射为旧的 `GameAdvice`（兼容存量前端）
    pub fn to_legacy_advice(&self) -> GameAdvice {
        GameAdvice {
            title: self.title.clone(),
            problem: self.problem.clone(),
            evidence: self.evidence.clone(),
            suggestions: self.suggestions.clone(),
            priority: self.priority,
            category: self.category,
            perspective: self.perspective,
            affected_role: self.position.clone(),
            target_player: self.target_player.clone(),
        }
    }
}

/// 统一对局分析结果
///
/// 统计部分直接复用现有结构，避免重复定义。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/MatchAnalysisResult.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct MatchAnalysisResult {
    /// 总览统计（所有参与分析的对局合计）
    pub overall_stats: PlayerMatchStats,

    /// 分位置统计
    pub position_stats: Vec<PositionStats>,

    /// 主要位置
    pub main_position: String,

    /// 深度证据覆盖的对局数（受 `maxAnalysisGames` 约束）
    pub analyzed_games: u32,

    /// 展示与基础统计覆盖的对局数（受 `count` 约束，**不**受 `maxAnalysisGames` 影响）
    pub display_games: u32,

    /// 展示用战绩列表（与 `overall_stats.recent_performance` 同一批对局）
    ///
    /// 有了它，前端一次调用就能同时渲染战绩列表与统计，不需要第二个命令。
    pub matches: Vec<MatchPerformance>,

    /// 确定性特征（唯一来源是 `evidence`）
    pub traits: Vec<DeterministicTrait>,

    /// 确定性建议（唯一来源是 `evidence`）
    pub advice: Vec<DeterministicAdvice>,

    /// 生效策略（便于前端解释结论来源）
    pub policy: AnalysisPolicy,

    /// 能力声明
    pub capabilities: AnalysisCapabilities,

    /// 降级诊断（策略级 + 运行期）
    pub diagnostics: Vec<AnalysisDiagnostic>,

    /// 确定性证据包（时间线 / 位置 / 对手 / 事件 / 聚合）
    ///
    /// 由 `domains::analysis::evidence` 产出；`None` 表示本次分析没有可用证据。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub evidence: Option<EvidenceBundle>,

    /// 本地 BYOK AI 解读（显式触发后由前端写回；编排器本身不调用 AI）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub ai_insight: Option<AiInsight>,
}

impl MatchAnalysisResult {
    /// 构造空结果（无符合条件对局时的正常业务结果）
    pub fn empty(policy: AnalysisPolicy) -> Self {
        let capabilities = AnalysisCapabilities::from_policy(&policy).refined_with_observed_queues(&[]);
        let mut diagnostics = policy.diagnostics.clone();
        diagnostics.push(AnalysisDiagnostic::new(
            AnalysisDegradationCode::NoMatchesInScope,
            "当前筛选条件下没有可分析的对局",
        ));

        Self {
            overall_stats: PlayerMatchStats::default(),
            position_stats: Vec::new(),
            main_position: UNKNOWN_POSITION.to_string(),
            analyzed_games: 0,
            display_games: 0,
            matches: Vec::new(),
            traits: Vec::new(),
            advice: Vec::new(),
            policy,
            capabilities,
            diagnostics,
            evidence: None,
            ai_insight: None,
        }
    }
}
