//! AI prompt 门槛与脱敏测试（不联网）
//!
//! 运行：`cargo test --test ai_prompt_gate`

use nidalee_lib::ai_contract::{compact_evidence_for_ai, AiPromptBundle};
use nidalee_lib::analysis_contract::{
    resolve_analysis_policy, AnalysisCapabilities, AnalysisDepth, AnalysisMode, MatchAnalysisRequest,
    MatchAnalysisResult,
};
use nidalee_lib::analysis_evidence::{
    EvidenceBundle, EvidenceConfidence, EvidenceEventRates, EvidencePosition, EvidenceQuality, EvidenceSummary,
    MatchEvidence,
};
use serde_json::Value;

fn base_result() -> MatchAnalysisResult {
    let req = MatchAnalysisRequest::new(AnalysisMode::SoloRanked, AnalysisDepth::Deep);
    MatchAnalysisResult::empty(resolve_analysis_policy(&req))
}

fn ranked_match(game_id: u64, queue_id: i64) -> MatchEvidence {
    MatchEvidence {
        game_id,
        target_puuid: "secret-puuid-should-not-leak".to_owned(),
        queue_id,
        game_duration_seconds: 1800,
        game_creation: 0,
        participant_id: 1,
        team_id: 100,
        champion_id: 64,
        win: true,
        position: EvidencePosition::Top,
        quality: EvidenceQuality::Full,
        opponent: None,
        phases: Vec::new(),
        events: None,
        timeline_span: None,
        diagnostics: Vec::new(),
    }
}

fn empty_event_rates() -> EvidenceEventRates {
    EvidenceEventRates {
        sample_size: 0,
        kills_per_game: 0.0,
        deaths_per_game: 0.0,
        assists_per_game: 0.0,
        dragon_takedowns_per_game: 0.0,
        herald_takedowns_per_game: 0.0,
        baron_takedowns_per_game: 0.0,
        horde_takedowns_per_game: 0.0,
        building_takedowns_per_game: 0.0,
        games_with_dragon_rate: 0.0,
        games_with_baron_rate: 0.0,
    }
}

fn ranked_summary(queue_id: i64) -> EvidenceSummary {
    EvidenceSummary {
        queue_id,
        position: EvidencePosition::Top,
        sample_size: 2,
        wins: 1,
        win_rate: 50.0,
        confidence: EvidenceConfidence::Medium,
        supports_conclusion: true,
        opponent_identified_games: 0,
        phase_averages: Vec::new(),
        event_rates: empty_event_rates(),
        evidence_game_ids: vec![1, 2],
    }
}

fn result_with_evidence(
    matches: Vec<MatchEvidence>,
    summaries: Vec<EvidenceSummary>,
    local_ai: bool,
) -> MatchAnalysisResult {
    let mut result = base_result();
    result.capabilities = AnalysisCapabilities {
        basic_stats: true,
        position_breakdown: true,
        deep_analysis: true,
        timeline: true,
        opponent: true,
        teammate: false,
        self_improvement: false,
        ranked_deep_evidence: local_ai,
        local_ai,
    };
    let match_count = matches.len() as u32;
    result.evidence = Some(EvidenceBundle {
        target_puuid: "secret-puuid-should-not-leak".to_string(),
        match_count,
        matches,
        summaries,
        diagnostics: Vec::new(),
    });
    result
}

#[test]
fn test_compact_rejects_when_local_ai_capability_false() {
    let result = result_with_evidence(vec![ranked_match(1, 420)], vec![ranked_summary(420)], false);
    assert!(compact_evidence_for_ai(&result).is_none());
}

#[test]
fn test_compact_rejects_when_no_ranked_games_even_if_capability_true() {
    // 故意构造不一致载荷：capability 为 true 但证据全是娱乐局
    let result = result_with_evidence(
        vec![ranked_match(1, 450), ranked_match(2, 450)],
        vec![ranked_summary(450)],
        true,
    );
    assert!(compact_evidence_for_ai(&result).is_none());
}

#[test]
fn test_compact_accepts_ranked_deep_evidence() {
    let result = result_with_evidence(
        vec![ranked_match(1, 420), ranked_match(2, 440)],
        vec![ranked_summary(420), ranked_summary(440)],
        true,
    );
    let bundle = compact_evidence_for_ai(&result).expect("排位深度证据应可压缩");
    assert_eq!(bundle.ranked_games, 2);
    assert!(bundle.summaries.iter().all(|s| s.queue_id == 420 || s.queue_id == 440));
}

#[test]
fn test_compact_prompt_never_contains_target_puuid() {
    let result = result_with_evidence(vec![ranked_match(1, 420)], vec![ranked_summary(420)], true);
    let bundle = compact_evidence_for_ai(&result).expect("应产出 bundle");
    let json = serde_json::to_string(&bundle).expect("序列化");
    assert!(
        !json.contains("secret-puuid-should-not-leak"),
        "AI prompt 不得泄漏 targetPuuid，实际: {json}"
    );
    assert!(!json.to_ascii_lowercase().contains("puuid"));

    let value: Value = serde_json::to_value(&bundle).unwrap();
    assert!(value.get("targetPuuid").is_none());
    let _typed: AiPromptBundle = bundle;
}
