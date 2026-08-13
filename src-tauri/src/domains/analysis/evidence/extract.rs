//! 纯提取入口
//!
//! 输入只有「对局详情 JSON + 可选时间线 JSON + 目标 PUUID」，
//! 不依赖 infrastructure 的 `MatchBundle`，由 orchestrator 负责映射。
//!
//! 任何数据缺失都通过 `EvidenceQuality` + `EvidenceDiagnostic` 表达，绝不 panic；
//! 只有「定位不到目标玩家」这种彻底无法分析的情况才返回可解释的错误。

use serde_json::Value;

use super::events::{extract_event_evidence, EventExtractContext};
use super::opponent::resolve_lane_opponent;
use super::position::position_from_role_lane;
use super::timeline::{compute_phase_evidence, timeline_frames, timeline_span};
use super::types::{
    EvidenceDiagnostic, EvidenceExtractionError, EvidenceIssue, EvidenceQuality, MatchEvidence, OpponentMatchMethod,
};

/// 提取单局证据
///
/// - `game`：对局详情（或战绩列表中的完整对局节点）
/// - `timeline`：`/lol-match-history/v1/game-timelines/{id}` 的原始响应；`None` 表示本局没有时间线
/// - `target_puuid`：分析目标
pub fn extract_match_evidence(
    game: &Value,
    timeline: Option<&Value>,
    target_puuid: &str,
) -> Result<MatchEvidence, EvidenceExtractionError> {
    let identities = game
        .get("participantIdentities")
        .and_then(Value::as_array)
        .ok_or(EvidenceExtractionError::MissingParticipantIdentities)?;

    let participant_id = identities
        .iter()
        .find(|identity| {
            identity
                .get("player")
                .and_then(|p| p.get("puuid"))
                .and_then(Value::as_str)
                == Some(target_puuid)
        })
        .and_then(|identity| identity.get("participantId"))
        .and_then(Value::as_i64)
        .ok_or(EvidenceExtractionError::TargetNotFound)? as i32;

    let participants = game
        .get("participants")
        .and_then(Value::as_array)
        .ok_or(EvidenceExtractionError::MissingParticipants)?;

    let participant = participants
        .iter()
        .find(|p| p.get("participantId").and_then(Value::as_i64).unwrap_or(0) as i32 == participant_id)
        .ok_or(EvidenceExtractionError::TargetParticipantMissing)?;

    let queue_id = game.get("queueId").and_then(Value::as_i64).unwrap_or(0);
    let participant_timeline = participant.get("timeline");
    let role = participant_timeline
        .and_then(|t| t.get("role"))
        .and_then(Value::as_str)
        .unwrap_or("NONE");
    let lane = participant_timeline
        .and_then(|t| t.get("lane"))
        .and_then(Value::as_str)
        .unwrap_or("NONE");

    let mut diagnostics = Diagnostics::default();

    let opponent = resolve_lane_opponent(game, timeline, participant_id, queue_id);
    match &opponent {
        None => diagnostics.push(
            EvidenceIssue::OpponentUnidentified,
            "无法确定对线对手：敌方没有唯一的同位置玩家，空间邻近不适用或不可靠",
        ),
        Some(found) if found.method == OpponentMatchMethod::SpatialProximity => diagnostics.push(
            EvidenceIssue::OpponentFromSpatialProximity,
            format!(
                "对手 {} 来自对线期空间邻近回退，置信度 {:.2}",
                found.participant_id, found.confidence
            ),
        ),
        Some(_) => {}
    }

    let mut quality = EvidenceQuality::Full;
    let mut phases = Vec::new();
    let mut events = None;
    let mut span = None;

    match timeline {
        None => {
            quality = EvidenceQuality::TimelineMissing;
            diagnostics.push(
                EvidenceIssue::TimelineMissing,
                "本局没有时间线数据，无法给出分阶段速率与事件证据",
            );
        }
        Some(timeline) => match timeline_frames(timeline) {
            None => {
                quality = EvidenceQuality::TimelineMissing;
                diagnostics.push(EvidenceIssue::TimelineFramesMissing, "时间线响应缺少 frames 字段");
            }
            Some(frames) if frames.is_empty() => {
                quality = EvidenceQuality::TimelineMissing;
                diagnostics.push(EvidenceIssue::TimelineFramesEmpty, "时间线 frames 为空");
            }
            Some(frames) => {
                span = timeline_span(frames, participant_id);
                let team_id = participant.get("teamId").and_then(Value::as_i64).unwrap_or(0) as i32;
                events = Some(extract_event_evidence(
                    frames,
                    EventExtractContext {
                        target_participant_id: participant_id,
                        target_team_id: team_id,
                        opponent_participant_id: opponent.as_ref().map(|o| o.participant_id),
                    },
                ));

                let (computed, flags) =
                    compute_phase_evidence(frames, participant_id, opponent.as_ref().map(|o| o.participant_id));
                phases = computed;

                // 「完全没有目标帧」是「部分帧缺失」的极端情况，共用同一个缺陷码：
                // 只保留信息量更大的那条消息，不重复上报。
                if phases.is_empty() {
                    quality = EvidenceQuality::TimelinePartial;
                    diagnostics.push(
                        EvidenceIssue::TargetParticipantFramesMissing,
                        "时间线中没有任何目标玩家的帧，无法给出分阶段速率",
                    );
                } else if flags.missing_target_frames {
                    quality = EvidenceQuality::TimelinePartial;
                    diagnostics.push(
                        EvidenceIssue::TargetParticipantFramesMissing,
                        "部分帧缺少目标玩家的 participantFrame，相关阶段速率已收敛",
                    );
                }
                if flags.single_frame_phase {
                    quality = EvidenceQuality::TimelinePartial;
                    diagnostics.push(
                        EvidenceIssue::SingleFrameNoRate,
                        "存在只有单个锚点帧的阶段，该阶段不产出每分钟速率",
                    );
                }
            }
        },
    }

    let stats = participant.get("stats");

    Ok(MatchEvidence {
        game_id: game.get("gameId").and_then(Value::as_u64).unwrap_or(0),
        target_puuid: target_puuid.to_owned(),
        queue_id,
        game_duration_seconds: game.get("gameDuration").and_then(Value::as_i64).unwrap_or(0),
        game_creation: game.get("gameCreation").and_then(Value::as_i64).unwrap_or(0),
        participant_id,
        team_id: participant.get("teamId").and_then(Value::as_i64).unwrap_or(0) as i32,
        champion_id: participant.get("championId").and_then(Value::as_i64).unwrap_or(0) as i32,
        win: stats
            .and_then(|s| s.get("win"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        position: position_from_role_lane(role, lane, queue_id),
        quality,
        opponent,
        phases,
        events,
        timeline_span: span,
        diagnostics: diagnostics.into_inner(),
    })
}

/// 保证「一个缺陷码只上报一次」的诊断收集器
///
/// 同一个码出现两条消息，前端只能重复展示同一件事，还会让「问题数量」失真。
/// 首次上报的消息胜出：调用点按信息量从大到小排列。
#[derive(Default)]
struct Diagnostics(Vec<EvidenceDiagnostic>);

impl Diagnostics {
    fn push(&mut self, code: EvidenceIssue, message: impl Into<String>) {
        if self.0.iter().any(|existing| existing.code == code) {
            return;
        }
        self.0.push(EvidenceDiagnostic::new(code, message));
    }

    fn into_inner(self) -> Vec<EvidenceDiagnostic> {
        self.0
    }
}
