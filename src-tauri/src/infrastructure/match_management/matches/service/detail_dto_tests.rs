use serde_json::{json, Value};

use super::map_game_detail;

fn detail_fixture(participants: Vec<Value>, participant_identities: Vec<Value>) -> Value {
    json!({
        "gameId": 987654321_u64,
        "gameDuration": 1800,
        "gameCreation": 1_700_000_000_000_i64,
        "gameMode": "CLASSIC",
        "gameType": "MATCHED_GAME",
        "gameVersion": "16.16.1",
        "mapId": 11,
        "queueId": 420,
        "teams": [],
        "participants": participants,
        "participantIdentities": participant_identities
    })
}

fn participant(participant_id: i32, team_id: i32, champion_id: i32, stats: Value) -> Value {
    json!({
        "participantId": participant_id,
        "teamId": team_id,
        "championId": champion_id,
        "spell1Id": 4,
        "spell2Id": 14,
        "stats": stats
    })
}

fn identity(
    participant_id: i32,
    summoner_name: Option<&str>,
    game_name: Option<&str>,
    tag_line: Option<&str>,
    profile_icon: i64,
) -> Value {
    json!({
        "participantId": participant_id,
        "player": {
            "summonerName": summoner_name,
            "gameName": game_name,
            "tagLine": tag_line,
            "profileIcon": profile_icon
        }
    })
}

#[test]
fn participant_identity_is_joined_by_participant_id() {
    let raw = detail_fixture(
        vec![
            participant(1, 100, 2_000_000_001, json!({})),
            participant(2, 200, 2_000_000_002, json!({})),
        ],
        vec![
            identity(2, Some("second"), None, None, 22),
            identity(1, Some("first"), None, None, 11),
        ],
    );

    let detail = map_game_detail(raw).unwrap();
    let identities: Vec<_> = detail
        .participants
        .iter()
        .map(|participant| {
            (
                participant.participant_id,
                participant.summoner_name.as_str(),
                participant.profile_icon_id,
            )
        })
        .collect();

    assert_eq!(identities, vec![(1, "first", 11), (2, "second", 22)]);
}

#[test]
fn riot_id_takes_priority_over_legacy_summoner_name() {
    let raw = detail_fixture(
        vec![participant(1, 100, 2_000_000_001, json!({}))],
        vec![identity(1, Some("legacy"), Some("RiotName"), Some("CN1"), 11)],
    );

    let detail = map_game_detail(raw).unwrap();

    assert_eq!(detail.participants[0].summoner_name, "RiotName#CN1");
}

#[test]
fn incomplete_riot_id_falls_back_to_legacy_summoner_name() {
    let raw = detail_fixture(
        vec![participant(1, 100, 2_000_000_001, json!({}))],
        vec![identity(1, Some("legacy"), Some("RiotName"), Some(""), 11)],
    );

    let detail = map_game_detail(raw).unwrap();

    assert_eq!(detail.participants[0].summoner_name, "legacy");
}

#[test]
fn team_totals_and_match_leaders_are_aggregated_across_participants() {
    let raw = detail_fixture(
        vec![
            participant(
                1,
                100,
                2_000_000_001,
                json!({
                    "kills": 3,
                    "goldEarned": 10000,
                    "totalDamageDealtToChampions": 18000,
                    "totalDamageTaken": 12000,
                    "visionScore": 20,
                    "largestMultiKill": 2
                }),
            ),
            participant(
                2,
                100,
                2_000_000_002,
                json!({
                    "kills": 5,
                    "goldEarned": 9000,
                    "totalDamageDealtToChampions": 14000,
                    "totalDamageTaken": 31000,
                    "visionScore": 30,
                    "largestMultiKill": 4
                }),
            ),
            participant(
                3,
                200,
                2_000_000_003,
                json!({
                    "kills": 9,
                    "goldEarned": 12000,
                    "totalDamageDealtToChampions": 26000,
                    "totalDamageTaken": 19000,
                    "visionScore": 15,
                    "largestMultiKill": 3
                }),
            ),
        ],
        vec![],
    );

    let detail = map_game_detail(raw).unwrap();
    let team_totals = (
        detail.blue_team_stats.kills,
        detail.blue_team_stats.gold_earned,
        detail.blue_team_stats.total_damage_dealt_to_champions,
        detail.blue_team_stats.vision_score,
        detail.red_team_stats.kills,
        detail.red_team_stats.gold_earned,
        detail.red_team_stats.total_damage_dealt_to_champions,
        detail.red_team_stats.vision_score,
    );
    let leaders = (
        detail.best_player_champion_id,
        detail.max_damage,
        detail.max_tank_champion_id,
        detail.max_tank,
        detail.max_streak_champion_id,
        detail.max_streak,
    );

    assert_eq!(team_totals, (8, 19000, 32000, 50, 9, 12000, 26000, 15));
    assert_eq!(leaders, (2_000_000_003, 26000, 2_000_000_002, 31000, 2_000_000_002, 4));
}

#[test]
fn missing_optional_participant_fields_use_existing_defaults() {
    let raw = detail_fixture(
        vec![json!({
            "participantId": 1,
            "stats": {}
        })],
        vec![],
    );

    let detail = map_game_detail(raw).unwrap();
    let participant = &detail.participants[0];
    let defaults = (
        participant.champion_id,
        participant.team_id,
        participant.summoner_name.as_str(),
        participant.profile_icon_id,
        participant.spell1_id,
        participant.spell2_id,
        participant.perk_primary_style,
        participant.perk0,
        participant.stats.kills,
        participant.stats.deaths,
        participant.stats.assists,
        participant.stats.item0,
    );

    assert_eq!(defaults, (0, 0, "未知玩家", 0, None, None, None, None, 0, 0, 0, None));
}

#[test]
fn champion_name_remains_none_when_catalog_has_no_matching_id() {
    let raw = detail_fixture(vec![participant(1, 100, i32::MAX, json!({}))], vec![]);

    let detail = map_game_detail(raw).unwrap();

    assert_eq!(detail.participants[0].champion_name, None);
}

#[test]
fn unknown_team_id_does_not_pollute_team_totals_or_match_leaders() {
    let raw = detail_fixture(
        vec![participant(
            1,
            0,
            2_000_000_001,
            json!({
                "kills": 9,
                "goldEarned": 12000,
                "totalDamageDealtToChampions": 26000,
                "totalDamageTaken": 31000,
                "largestMultiKill": 4,
                "visionScore": 15
            }),
        )],
        vec![],
    );

    let detail = map_game_detail(raw).unwrap();

    assert_eq!(
        (
            detail.red_team_stats.kills,
            detail.red_team_stats.gold_earned,
            detail.red_team_stats.total_damage_dealt_to_champions,
            detail.red_team_stats.vision_score,
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(
        (
            detail.best_player_champion_id,
            detail.max_damage,
            detail.max_tank_champion_id,
            detail.max_tank,
            detail.max_streak_champion_id,
            detail.max_streak,
        ),
        (0, 0, 0, 0, 0, 0)
    );
}

#[test]
fn missing_profile_icon_uses_zero_without_rejecting_detail() {
    let raw = detail_fixture(
        vec![participant(1, 100, 2_000_000_001, json!({}))],
        vec![json!({
            "participantId": 1,
            "player": {
                "summonerName": "legacy",
                "gameName": "RiotName",
                "tagLine": "CN1"
            }
        })],
    );

    let detail = map_game_detail(raw).unwrap();

    assert_eq!(detail.participants[0].profile_icon_id, 0);
}

#[test]
fn null_or_missing_player_uses_unknown_identity_without_rejecting_detail() {
    for identity in [
        json!({ "participantId": 1, "player": null }),
        json!({ "participantId": 1 }),
    ] {
        let raw = detail_fixture(vec![participant(1, 100, 2_000_000_001, json!({}))], vec![identity]);

        let detail = map_game_detail(raw).unwrap();

        assert_eq!(
            (
                detail.participants[0].summoner_name.as_str(),
                detail.participants[0].profile_icon_id,
            ),
            ("未知玩家", 0)
        );
    }
}

#[test]
fn empty_identity_names_use_unknown_player_fallback() {
    let raw = detail_fixture(
        vec![participant(1, 100, 2_000_000_001, json!({}))],
        vec![identity(1, Some("  "), Some(""), Some("CN1"), 11)],
    );

    let detail = map_game_detail(raw).unwrap();

    assert_eq!(detail.participants[0].summoner_name, "未知玩家");
}
