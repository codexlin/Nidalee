//! 多局聚合
//!
//! 按 `(queue_id, position)` 分组，只产出「样本量 + 均值 + 事件率 + 证据 gameId」，
//! 不产出任何结论。样本量不足时 `supports_conclusion = false`，
//! 保证一场对局无法触发全局强结论。
//!
//! 分组用 `BTreeMap`，输出顺序永远是队列升序 + 位置枚举声明序。

use std::collections::BTreeMap;

use super::position::EvidencePosition;
use super::types::{
    EvidenceConfidence, EvidenceEventRates, EvidenceSummary, GamePhase, MatchEvidence, PhaseAverages, PhaseEvidence,
    MIN_SAMPLE_FOR_CONCLUSION,
};

/// 按 (队列, 位置) 聚合多局证据
pub fn aggregate_match_evidence(matches: &[MatchEvidence]) -> Vec<EvidenceSummary> {
    let mut grouped: BTreeMap<(i64, EvidencePosition), Vec<&MatchEvidence>> = BTreeMap::new();
    for evidence in matches {
        grouped
            .entry((evidence.queue_id, evidence.position))
            .or_default()
            .push(evidence);
    }

    grouped
        .into_iter()
        .map(|((queue_id, position), group)| summarize(queue_id, position, &group))
        .collect()
}

fn summarize(queue_id: i64, position: EvidencePosition, group: &[&MatchEvidence]) -> EvidenceSummary {
    let sample_size = group.len() as u32;
    let wins = group.iter().filter(|m| m.win).count() as u32;
    let win_rate = if sample_size > 0 {
        wins as f64 / sample_size as f64 * 100.0
    } else {
        0.0
    };

    let mut evidence_game_ids: Vec<u64> = group.iter().map(|m| m.game_id).collect();
    evidence_game_ids.sort_unstable();
    evidence_game_ids.dedup();

    let phase_averages = GamePhase::ALL
        .iter()
        .filter_map(|phase| average_phase(*phase, group))
        .collect();

    let confidence = EvidenceConfidence::from_sample_size(sample_size);

    EvidenceSummary {
        queue_id,
        position,
        sample_size,
        wins,
        win_rate,
        confidence,
        supports_conclusion: sample_size >= MIN_SAMPLE_FOR_CONCLUSION,
        opponent_identified_games: group.iter().filter(|m| m.opponent.is_some()).count() as u32,
        phase_averages,
        event_rates: average_events(group),
        evidence_game_ids,
    }
}

/// 某阶段的均值；该阶段在所有对局里都不存在时返回 `None`
fn average_phase(phase: GamePhase, group: &[&MatchEvidence]) -> Option<PhaseAverages> {
    let phases: Vec<&PhaseEvidence> = group.iter().filter_map(|m| m.phase(phase)).collect();
    if phases.is_empty() {
        return None;
    }

    // 只有真正算出速率的阶段才计入样本（零时长阶段不污染均值）
    let rated: Vec<&&PhaseEvidence> = phases.iter().filter(|p| p.cs_per_min.is_some()).collect();
    let with_opponent: Vec<&&PhaseEvidence> = phases.iter().filter(|p| p.opponent_diff.is_some()).collect();

    Some(PhaseAverages {
        phase,
        sample_size: rated.len() as u32,
        opponent_sample_size: with_opponent.len() as u32,
        avg_lane_cs_per_min: mean(rated.iter().filter_map(|p| p.lane_cs_per_min)),
        avg_jungle_cs_per_min: mean(rated.iter().filter_map(|p| p.jungle_cs_per_min)),
        avg_cs_per_min: mean(rated.iter().filter_map(|p| p.cs_per_min)),
        avg_gold_per_min: mean(rated.iter().filter_map(|p| p.gold_per_min)),
        avg_xp_per_min: mean(rated.iter().filter_map(|p| p.xp_per_min)),
        avg_cs_diff: mean(
            with_opponent
                .iter()
                .filter_map(|p| p.opponent_diff.as_ref())
                .map(|d| d.cs_diff as f64),
        ),
        avg_gold_diff: mean(
            with_opponent
                .iter()
                .filter_map(|p| p.opponent_diff.as_ref())
                .map(|d| d.gold_diff as f64),
        ),
        avg_xp_diff: mean(
            with_opponent
                .iter()
                .filter_map(|p| p.opponent_diff.as_ref())
                .map(|d| d.xp_diff as f64),
        ),
        avg_overall_advantage: mean(
            with_opponent
                .iter()
                .filter_map(|p| p.opponent_diff.as_ref())
                .map(|d| d.overall_advantage),
        ),
    })
}

/// 场均事件频率
///
/// 分母只包含「时间线里真的有目标玩家帧」的对局：
/// 有 `frames` 但目标完全不在其中时，抽出来的事件必然是空的，
/// 把它算进分母只会把场均击杀/参团稀释成假的低值。
fn average_events(group: &[&MatchEvidence]) -> EvidenceEventRates {
    let events: Vec<_> = group
        .iter()
        .filter(|m| m.timeline_span.map(|span| span.target_frame_count > 0).unwrap_or(false))
        .filter_map(|m| m.events.as_ref())
        .collect();
    let sample_size = events.len() as u32;
    if sample_size == 0 {
        return EvidenceEventRates::default();
    }

    let per_game = |total: u32| total as f64 / sample_size as f64;
    let rate_of = |count: usize| count as f64 / sample_size as f64;

    EvidenceEventRates {
        sample_size,
        kills_per_game: per_game(events.iter().map(|e| e.kills).sum()),
        deaths_per_game: per_game(events.iter().map(|e| e.deaths).sum()),
        assists_per_game: per_game(events.iter().map(|e| e.assists).sum()),
        dragon_takedowns_per_game: per_game(events.iter().map(|e| e.dragon_takedowns).sum()),
        herald_takedowns_per_game: per_game(events.iter().map(|e| e.herald_takedowns).sum()),
        baron_takedowns_per_game: per_game(events.iter().map(|e| e.baron_takedowns).sum()),
        horde_takedowns_per_game: per_game(events.iter().map(|e| e.horde_takedowns).sum()),
        building_takedowns_per_game: per_game(events.iter().map(|e| e.building_takedowns).sum()),
        games_with_dragon_rate: rate_of(events.iter().filter(|e| e.dragon_takedowns > 0).count()),
        games_with_baron_rate: rate_of(events.iter().filter(|e| e.baron_takedowns > 0).count()),
    }
}

fn mean(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut total = 0.0;
    let mut count = 0u32;
    for value in values {
        total += value;
        count += 1;
    }
    (count > 0).then(|| total / count as f64)
}
