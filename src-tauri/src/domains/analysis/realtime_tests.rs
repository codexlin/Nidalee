use serde_json::{json, Value};

use super::{analyze_realtime_history, build_ranked_player_rating, RealtimeMatchContext};
use crate::shared::types::{
    PlayerAnalysisBasis, PlayerAnalysisConfidence, PlayerAnalysisScope, PlayerMatchStats, PlayerRankSummary,
    RankedRatingGrade, RankedRatingQueue,
};

const PUUID: &str = "test-puuid";

fn game(game_id: u64, queue_id: i64) -> Value {
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
            "championId": 11,
            "teamId": 100,
            "stats": {
                "win": true,
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
            "timeline": { "role": "SOLO", "lane": "MIDDLE" }
        }]
    })
}

#[test]
fn custom_game_should_use_ranked_history_without_filtering_custom_queue() {
    let mut games = (0..6).map(|id| game(id, 420)).collect::<Vec<_>>();
    games.extend((6..9).map(|id| game(id, 430)));

    let analysis = analyze_realtime_history(&games, PUUID, RealtimeMatchContext::Custom, None).unwrap();

    assert_eq!(analysis.basis.primary_scope, PlayerAnalysisScope::Ranked);
    assert_eq!(analysis.basis.primary_games, 6);
    assert_eq!(analysis.basis.supporting_games, 0);
    assert_eq!(analysis.stats.total_games, 6);
    assert!(analysis
        .stats
        .recent_performance
        .iter()
        .all(|game| game.queue_id == Some(420)));
    assert_eq!(
        analysis
            .recent_matches
            .iter()
            .map(|game| game.queue_id)
            .collect::<Vec<_>>(),
        vec![Some(430), Some(430), Some(430), Some(420), Some(420), Some(420)]
    );
}

#[test]
fn custom_game_without_ranked_sample_should_not_guess_the_map() {
    let games = vec![game(1, 430), game(2, 450), game(3, 900)];

    let analysis = analyze_realtime_history(&games, PUUID, RealtimeMatchContext::Custom, None).unwrap();

    assert_eq!(analysis.basis.primary_scope, PlayerAnalysisScope::RecentOverall);
    assert!(analysis.basis.fallback_used);
    assert_eq!(analysis.stats.total_games, 3);
}

#[test]
fn ranked_game_should_fallback_to_combined_ranked_sample() {
    let mut games = (0..3).map(|id| game(id, 420)).collect::<Vec<_>>();
    games.extend((3..7).map(|id| game(id, 440)));

    let analysis = analyze_realtime_history(&games, PUUID, RealtimeMatchContext::SoloRanked, None).unwrap();

    assert_eq!(analysis.basis.primary_scope, PlayerAnalysisScope::Ranked);
    assert!(analysis.basis.fallback_used);
    assert_eq!(analysis.stats.total_games, 7);
    assert_eq!(analysis.basis.supporting_games, 4);
}

#[test]
fn aram_game_should_not_mix_ranked_matches_into_primary_stats() {
    let mut games = (0..5).map(|id| game(id, 450)).collect::<Vec<_>>();
    games.extend((5..10).map(|id| game(id, 420)));

    let analysis = analyze_realtime_history(&games, PUUID, RealtimeMatchContext::Aram, None).unwrap();

    assert_eq!(analysis.basis.primary_scope, PlayerAnalysisScope::Aram);
    assert_eq!(analysis.stats.total_games, 5);
    assert!(analysis
        .stats
        .recent_performance
        .iter()
        .all(|game| game.queue_id == Some(450)));
    assert_eq!(
        analysis
            .recent_matches
            .iter()
            .map(|game| game.game_id)
            .collect::<Vec<_>>(),
        vec![Some(9), Some(8), Some(7), Some(6), Some(5), Some(4)]
    );
    assert_eq!(
        analysis
            .recent_matches
            .iter()
            .map(|game| game.queue_id)
            .collect::<Vec<_>>(),
        vec![Some(420), Some(420), Some(420), Some(420), Some(420), Some(450)]
    );
}

#[test]
fn realtime_summary_exposes_measured_traits() {
    let games = (0..6).map(|id| game(id, 440)).collect::<Vec<_>>();
    let analysis = analyze_realtime_history(&games, PUUID, RealtimeMatchContext::FlexRanked, None).unwrap();

    assert!(analysis.stats.traits.iter().any(|item| item.name == "近期状态强势"));
    assert!(analysis.stats.traits.iter().any(|item| item.name == "生存稳定"));
    assert!(analysis.stats.traits.iter().all(|item| !item.description.is_empty()));
}

#[test]
fn realtime_analysis_scans_the_window_but_caps_the_selected_sample_at_twenty() {
    let games = (0..30).map(|id| game(id, 440)).collect::<Vec<_>>();
    let analysis = analyze_realtime_history(&games, PUUID, RealtimeMatchContext::FlexRanked, None).unwrap();

    assert_eq!(analysis.basis.scanned_games, 30);
    assert_eq!(analysis.basis.primary_games, 20);
    assert_eq!(analysis.stats.total_games, 20);
    assert_eq!(analysis.recent_matches.len(), 6);
    assert_eq!(
        analysis.stats.recent_performance.first().and_then(|game| game.game_id),
        Some(29)
    );
    assert_eq!(
        analysis.stats.recent_performance.last().and_then(|game| game.game_id),
        Some(10)
    );
}

#[test]
fn flex_rating_uses_flex_rank_instead_of_stronger_solo_rank() {
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
    assert!(rating.summary.contains("近期表现小幅上调"));
}

#[test]
fn ranked_rating_requires_matching_rank_but_not_five_recent_games() {
    let solo = rank("EMERALD", "II", 40);
    let enough = stats(12, 55.0, 3.5);
    let too_small = stats(4, 75.0, 8.0);
    let ranked_basis = basis(PlayerAnalysisScope::SoloRanked, PlayerAnalysisConfidence::Medium, 12);
    let small_basis = basis(PlayerAnalysisScope::SoloRanked, PlayerAnalysisConfidence::Low, 4);

    assert!(build_ranked_player_rating(440, Some(&solo), None, &enough, &ranked_basis).is_none());
    let rank_only = build_ranked_player_rating(420, Some(&solo), None, &too_small, &small_basis).unwrap();
    assert_eq!(rank_only.grade, RankedRatingGrade::B);
    assert!(rank_only.summary.contains("近期样本不足，未进行状态修正"));
}

#[test]
fn custom_ranked_sample_uses_the_stronger_available_rank() {
    let solo = rank("PLATINUM", "I", 80);
    let flex = rank("EMERALD", "III", 20);
    let rating = build_ranked_player_rating(
        0,
        Some(&solo),
        Some(&flex),
        &stats(16, 50.0, 3.0),
        &basis(PlayerAnalysisScope::Ranked, PlayerAnalysisConfidence::High, 16),
    )
    .unwrap();

    assert_eq!(rating.queue, RankedRatingQueue::FlexRanked);
    assert!(rating.summary.contains("流光翡翠"));
}

#[test]
fn non_ranked_sample_keeps_the_stronger_rank_as_an_unadjusted_baseline() {
    let solo = rank("EMERALD", "IV", 10);
    let flex = rank("GOLD", "IV", 56);
    let rating = build_ranked_player_rating(
        0,
        Some(&solo),
        Some(&flex),
        &stats(44, 34.0, 3.04),
        &basis(PlayerAnalysisScope::RecentOverall, PlayerAnalysisConfidence::High, 44),
    )
    .unwrap();

    assert_eq!(rating.queue, RankedRatingQueue::SoloRanked);
    assert_eq!(rating.grade, RankedRatingGrade::B);
    assert!(rating.summary.contains("当前样本并非排位，仅展示段位基础评级"));
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
