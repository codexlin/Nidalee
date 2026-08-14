use serde_json::{json, Value};

use super::{analyze_realtime_history, build_ranked_player_rating, project_ranked_context, RealtimeMatchContext};
use crate::shared::types::{
    PlayerAnalysisBasis, PlayerAnalysisConfidence, PlayerAnalysisDepth, PlayerAnalysisScope, PlayerMatchStats,
    PlayerRankSummary, RankedRatingGrade, RankedRatingQueue,
};

const PUUID: &str = "test-puuid";

fn game(game_id: u64, queue_id: i64) -> Value {
    game_with(game_id, queue_id, 11, "SOLO", "MIDDLE", true)
}

fn game_with(game_id: u64, queue_id: i64, champion_id: i32, role: &str, lane: &str, win: bool) -> Value {
    json!({
        "gameId": game_id,
        "queueId": queue_id,
        "gameDuration": 1800,
        "gameCreation": 1_700_000_000_000_i64 + game_id as i64,
        "participantIdentities": [{
            "participantId": 1,
            "player": { "puuid": PUUID }
        }],
        "participants": [{
            "participantId": 1,
            "championId": champion_id,
            "teamId": 100,
            "stats": {
                "win": win,
                "kills": 8,
                "deaths": 4,
                "assists": 10,
                "totalDamageDealtToChampions": 20_000,
                "totalDamageTaken": 18_000,
                "goldEarned": 12_000,
                "visionScore": 25,
                "wardsPlaced": 8,
                "wardsKilled": 2,
                "totalMinionsKilled": 150,
                "neutralMinionsKilled": 20
            },
            "timeline": { "role": role, "lane": lane }
        }]
    })
}

fn analyze(games: &[Value], queue_id: i64) -> crate::shared::types::PlayerAnalysisResult {
    analyze_realtime_history(
        games,
        PUUID,
        RealtimeMatchContext::from_lcu(queue_id, false),
        None,
        None,
        None,
    )
    .unwrap()
}

#[test]
fn solo_ranked_should_use_only_solo_matches_without_fallback() {
    let mut games = (0..3).map(|id| game(id, 420)).collect::<Vec<_>>();
    games.extend((3..12).map(|id| game(id, 440)));
    games.extend((12..20).map(|id| game(id, 430)));

    let analysis = analyze(&games, 420);

    assert_eq!(analysis.depth, PlayerAnalysisDepth::Ranked);
    assert_eq!(analysis.basis.primary_scope, PlayerAnalysisScope::SoloRanked);
    assert_eq!(analysis.basis.primary_games, 3);
    assert_eq!(analysis.basis.supporting_games, 0);
    assert!(!analysis.basis.fallback_used);
    assert!(analysis
        .stats
        .recent_performance
        .iter()
        .all(|game| game.queue_id == Some(420)));
}

#[test]
fn flex_ranked_should_use_only_flex_matches_without_fallback() {
    let mut games = (0..8).map(|id| game(id, 420)).collect::<Vec<_>>();
    games.extend((8..12).map(|id| game(id, 440)));

    let analysis = analyze(&games, 440);

    assert_eq!(analysis.basis.primary_scope, PlayerAnalysisScope::FlexRanked);
    assert_eq!(analysis.stats.total_games, 4);
    assert!(analysis
        .stats
        .recent_performance
        .iter()
        .all(|game| game.queue_id == Some(440)));
}

#[test]
fn ranked_analysis_should_keep_zero_sample_instead_of_borrowing_other_queues() {
    let games = (0..8).map(|id| game(id, 440)).collect::<Vec<_>>();

    let analysis = analyze(&games, 420);

    assert_eq!(analysis.depth, PlayerAnalysisDepth::Ranked);
    assert_eq!(analysis.basis.primary_games, 0);
    assert_eq!(analysis.stats.total_games, 0);
    assert_eq!(analysis.basis.confidence, PlayerAnalysisConfidence::Low);
    assert!(analysis.ranked.unwrap().position_profile.positions.is_empty());
}

#[test]
fn ranked_analysis_should_cap_exact_queue_sample_at_twenty() {
    let games = (0..30).map(|id| game(id, 440)).collect::<Vec<_>>();

    let analysis = analyze(&games, 440);

    assert_eq!(analysis.basis.scanned_games, 30);
    assert_eq!(analysis.basis.primary_games, 20);
    assert_eq!(analysis.stats.total_games, 20);
    assert_eq!(analysis.recent_matches.len(), 6);
    assert_eq!(
        analysis.stats.recent_performance.first().and_then(|game| game.game_id),
        Some(29)
    );
}

#[test]
fn non_ranked_analysis_should_use_recent_twenty_without_ranked_depth() {
    let games = (0..30)
        .map(|id| game(id, if id % 2 == 0 { 420 } else { 450 }))
        .collect::<Vec<_>>();

    let analysis = analyze(&games, 450);

    assert_eq!(analysis.depth, PlayerAnalysisDepth::Simple);
    assert_eq!(analysis.basis.primary_scope, PlayerAnalysisScope::RecentOverall);
    assert_eq!(analysis.stats.total_games, 20);
    assert!(analysis.ranked.is_none());
    assert!(analysis.stats.traits.is_empty());
}

#[test]
fn custom_game_should_always_use_simple_recent_summary() {
    let games = (0..12).map(|id| game(id, 420)).collect::<Vec<_>>();
    let analysis = analyze_realtime_history(
        &games,
        PUUID,
        RealtimeMatchContext::from_lcu(420, true),
        None,
        None,
        None,
    )
    .unwrap();

    assert_eq!(analysis.depth, PlayerAnalysisDepth::Simple);
    assert_eq!(analysis.basis.primary_scope, PlayerAnalysisScope::RecentOverall);
    assert!(analysis.ranked.is_none());
}

#[test]
fn ranked_analysis_should_build_position_and_current_champion_profiles() {
    let games = vec![
        game_with(1, 420, 11, "JUNGLE", "JUNGLE", true),
        game_with(2, 420, 11, "JUNGLE", "JUNGLE", false),
        game_with(3, 420, 22, "SOLO", "TOP", true),
    ];
    let resolve = |champion_id| match champion_id {
        11 => Some("Master Yi".to_string()),
        22 => Some("Ashe".to_string()),
        _ => None,
    };

    let analysis = analyze_realtime_history(
        &games,
        PUUID,
        RealtimeMatchContext::from_lcu(420, false),
        Some("JUNGLE"),
        Some(11),
        Some(&resolve),
    )
    .unwrap();
    let ranked = analysis.ranked.unwrap();

    assert_eq!(ranked.position_profile.primary_position.as_deref(), Some("JUNGLE"));
    assert_eq!(
        ranked.position_profile.current_position.as_ref().map(|current| (
            current.position.as_str(),
            current.sample.games,
            current.is_primary
        )),
        Some(("JUNGLE", 2, true))
    );
    assert_eq!(
        ranked
            .current_champion
            .as_ref()
            .map(|champion| (champion.champion_name.as_deref(), champion.sample.games)),
        Some((Some("Master Yi"), 2))
    );
}

#[test]
fn cached_ranked_breakdown_should_reproject_without_games() {
    let games = vec![
        game_with(1, 440, 11, "JUNGLE", "JUNGLE", true),
        game_with(2, 440, 22, "DUO_CARRY", "BOTTOM", true),
    ];
    let mut analysis = analyze(&games, 440);
    let ranked = analysis.ranked.as_mut().unwrap();

    project_ranked_context(ranked, Some("BOTTOM"), Some(22), None);

    assert_eq!(
        ranked
            .position_profile
            .current_position
            .as_ref()
            .map(|current| (current.position.as_str(), current.sample.games)),
        Some(("ADC", 1))
    );
    assert_eq!(
        ranked.current_champion.as_ref().map(|champion| champion.sample.games),
        Some(1)
    );
}

#[test]
fn flex_rating_should_use_flex_rank_instead_of_stronger_solo_rank() {
    let solo = rank("DIAMOND", "I", 80);
    let flex = rank("GOLD", "IV", 20);
    let rating = build_ranked_player_rating(
        440,
        Some(&solo),
        Some(&flex),
        &stats(20, 60.0, 4.0),
        &basis(PlayerAnalysisScope::FlexRanked, PlayerAnalysisConfidence::High, 20),
    )
    .unwrap();

    assert_eq!(rating.queue, RankedRatingQueue::FlexRanked);
    assert_eq!(rating.grade, RankedRatingGrade::C);
    assert!(rating.summary.contains("灵活排位 荣耀黄金 IV"));
}

#[test]
fn non_ranked_queue_should_not_build_a_ranked_rating() {
    let solo = rank("EMERALD", "II", 40);

    assert!(build_ranked_player_rating(
        450,
        Some(&solo),
        None,
        &stats(20, 60.0, 4.0),
        &basis(PlayerAnalysisScope::RecentOverall, PlayerAnalysisConfidence::High, 20),
    )
    .is_none());
}

fn rank(tier: &str, division: &str, league_points: i32) -> PlayerRankSummary {
    PlayerRankSummary {
        tier: tier.to_string(),
        division: Some(division.to_string()),
        league_points: Some(league_points),
    }
}

fn stats(total_games: u32, win_rate: f64, avg_kda: f64) -> PlayerMatchStats {
    PlayerMatchStats {
        total_games,
        win_rate,
        avg_kda,
        ..PlayerMatchStats::default()
    }
}

fn basis(
    primary_scope: PlayerAnalysisScope,
    confidence: PlayerAnalysisConfidence,
    primary_games: u32,
) -> PlayerAnalysisBasis {
    PlayerAnalysisBasis {
        primary_scope,
        scanned_games: primary_games,
        primary_games,
        supporting_games: 0,
        confidence,
        fallback_used: false,
    }
}
