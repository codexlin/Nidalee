use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use crate::analysis_evidence::EvidencePosition;
use crate::domains::analysis::{EvidenceQuality, MatchEvidence};

use super::resolve_match_evidence;

fn evidence(game_id: u64, quality: EvidenceQuality) -> MatchEvidence {
    MatchEvidence {
        game_id,
        target_puuid: "target-puuid".to_owned(),
        queue_id: 420,
        game_duration_seconds: 1_800,
        game_creation: 0,
        participant_id: 1,
        team_id: 100,
        champion_id: 59,
        win: true,
        position: EvidencePosition::Jungle,
        quality,
        opponent: None,
        phases: Vec::new(),
        events: None,
        timeline_span: None,
        diagnostics: Vec::new(),
    }
}

#[tokio::test]
async fn full_cache_skips_fetch() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_for_fetch = Arc::clone(&calls);

    let (resolved, from_cache) =
        resolve_match_evidence("target-puuid", 7, Some(evidence(7, EvidenceQuality::Full)), move || {
            calls_for_fetch.fetch_add(1, Ordering::SeqCst);
            async { Err("fetch should not run".to_owned()) }
        })
        .await
        .expect("full cache should resolve");

    assert_eq!(resolved.game_id, 7);
    assert!(from_cache);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn partial_cache_is_refreshed_when_fetch_succeeds() {
    let calls = Arc::new(AtomicUsize::new(0));
    let calls_for_fetch = Arc::clone(&calls);

    let (resolved, from_cache) = resolve_match_evidence(
        "target-puuid",
        7,
        Some(evidence(7, EvidenceQuality::TimelineMissing)),
        move || {
            calls_for_fetch.fetch_add(1, Ordering::SeqCst);
            async { Ok(evidence(7, EvidenceQuality::Full)) }
        },
    )
    .await
    .expect("refresh should resolve");

    assert_eq!(resolved.quality, EvidenceQuality::Full);
    assert!(!from_cache);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn failed_refresh_falls_back_to_matching_partial_cache() {
    let (resolved, from_cache) = resolve_match_evidence(
        "target-puuid",
        7,
        Some(evidence(7, EvidenceQuality::TimelinePartial)),
        || async { Err("timeline unavailable".to_owned()) },
    )
    .await
    .expect("matching partial cache should be a valid fallback");

    assert_eq!(resolved.quality, EvidenceQuality::TimelinePartial);
    assert!(from_cache);
}

#[tokio::test]
async fn wrong_game_cache_does_not_mask_fetch_error() {
    let error = resolve_match_evidence("target-puuid", 7, Some(evidence(8, EvidenceQuality::Full)), || async {
        Err("detail unavailable".to_owned())
    })
    .await
    .expect_err("wrong-game cache must be ignored");

    assert_eq!(error, "detail unavailable");
}

#[tokio::test]
async fn wrong_player_cache_does_not_mask_fetch_error() {
    let error = resolve_match_evidence(
        "different-puuid",
        7,
        Some(evidence(7, EvidenceQuality::Full)),
        || async { Err("detail unavailable".to_owned()) },
    )
    .await
    .expect_err("same-game evidence for another player must be ignored");

    assert_eq!(error, "detail unavailable");
}
