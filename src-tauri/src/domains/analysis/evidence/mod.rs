//! 确定性 Evidence 证据层
//!
//! 职责边界：
//! - `types`：证据数据结构的唯一定义（可 serde / ts-rs）
//! - `position`：统一位置枚举与 LCU role/lane 归一化
//! - `timeline`：帧级纯计算（真实经过时间、阶段、归一化优势）
//! - `opponent`：对线对手识别（lane-role 优先，空间邻近回退）
//! - `events`：事件参与证据
//! - `extract`：单局提取入口
//! - `aggregate`：按 (队列, 位置) 的多局聚合
//!
//! DDD 约束：本模块**不**依赖 `infrastructure`，输入只有
//! 「对局详情 JSON + 可选时间线 JSON + 目标 PUUID」，
//! `MatchBundle → EvidenceInput` 的映射由 orchestrator 负责。
//!
//! 这一层只描述事实，不做阈值判断、不生成结论、不调用 AI。

pub mod activity;
pub mod aggregate;
pub mod events;
pub mod extract;
pub mod opponent;
pub mod position;
pub mod timeline;
pub mod types;

use serde_json::Value;

// 证据层先落地，crate 内的消费者（orchestrator / advice）在后续任务接入
#[allow(unused_imports)]
pub use aggregate::aggregate_match_evidence;
#[allow(unused_imports)]
pub use activity::ActivityContext;
#[allow(unused_imports)]
pub use events::{extract_event_evidence, EventExtractContext};
#[allow(unused_imports)]
pub use extract::extract_match_evidence;
#[allow(unused_imports)]
pub use opponent::resolve_lane_opponent;
#[allow(unused_imports)]
pub use position::{is_aram_queue, position_from_role_lane, EvidencePosition};
#[allow(unused_imports)]
pub use timeline::{
    compute_phase_evidence, frame_snapshot, laning_opponent_diff, timeline_frames, FrameSnapshot, PhaseComputationFlags,
};
#[allow(unused_imports)]
pub use types::{
    advantage_to_percent, normalized_share_advantage, DeathCause, EventEvidence, EventInvolvement, EvidenceBundle,
    EvidenceConfidence, EvidenceDiagnostic, EvidenceEventKind, EvidenceEventRates, EvidenceExtractionError,
    EvidenceIssue, EvidenceQuality, EvidenceSummary, GamePhase, KeyEventEvidence, MatchEvidence, OpponentEvidence,
    OpponentMatchMethod, PhaseAverages, PhaseEvidence, PhaseOpponentDiff, TimelineSpan, UnknownTimelineEvent,
    ADVANTAGE_PERCENT_SCALE, ADVANTAGE_WEIGHT_CS, ADVANTAGE_WEIGHT_GOLD, ADVANTAGE_WEIGHT_XP,
    MIN_SAMPLE_FOR_CONCLUSION, PHASE_EARLY_END_MS, PHASE_MID_END_MS,
};

/// 单局证据的纯输入
///
/// 刻意只持有 JSON 引用：领域层不认识 `MatchBundle`，
/// orchestrator 负责把 `bundle.game()` / `bundle.timeline` 映射过来。
#[derive(Debug, Clone, Copy)]
pub struct MatchEvidenceInput<'a> {
    pub game: &'a Value,
    pub timeline: Option<&'a Value>,
}

impl<'a> MatchEvidenceInput<'a> {
    pub fn new(game: &'a Value, timeline: Option<&'a Value>) -> Self {
        Self { game, timeline }
    }

    /// 只有详情、没有时间线
    pub fn detail_only(game: &'a Value) -> Self {
        Self { game, timeline: None }
    }
}

/// 提取并聚合多局证据
///
/// 单局提取失败不会中断整体：失败原因记录在 `bundle.diagnostics`，
/// 其余对局照常产出证据。输入顺序即 `matches` 顺序，聚合顺序与输入无关。
pub fn build_evidence_bundle(target_puuid: &str, inputs: &[MatchEvidenceInput<'_>]) -> EvidenceBundle {
    let mut matches = Vec::with_capacity(inputs.len());
    let mut diagnostics = Vec::new();

    for input in inputs {
        match extract_match_evidence(input.game, input.timeline, target_puuid) {
            Ok(evidence) => matches.push(evidence),
            Err(error) => diagnostics.push(EvidenceDiagnostic::new(
                EvidenceIssue::MatchSkipped,
                format!("跳过一局：{}", error.message()),
            )),
        }
    }

    let summaries = aggregate_match_evidence(&matches);

    EvidenceBundle {
        target_puuid: target_puuid.to_string(),
        match_count: matches.len() as u32,
        matches,
        summaries,
        diagnostics,
    }
}
