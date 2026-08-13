use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::shared::types::{GameDetail, ParticipantInfo, ParticipantStats, TeamInfo, TeamStats};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiGameData {
    game_id: u64,
    game_duration: i32,
    game_creation: i64,
    game_mode: String,
    game_type: String,
    game_version: String,
    map_id: i32,
    queue_id: i32,
    teams: Vec<TeamInfo>,
    participants: Vec<ApiParticipant>,
    participant_identities: Vec<ApiParticipantIdentity>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct ApiParticipant {
    participant_id: i32,
    #[serde(default)]
    team_id: Option<i32>,
    #[serde(default)]
    champion_id: Option<i32>,
    #[serde(default)]
    spell1_id: Option<i32>,
    #[serde(default)]
    spell2_id: Option<i32>,
    stats: ApiParticipantStats,
}

#[derive(Deserialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiParticipantStats {
    #[serde(default)]
    kills: Option<i32>,
    #[serde(default)]
    deaths: Option<i32>,
    #[serde(default)]
    assists: Option<i32>,
    #[serde(default)]
    champ_level: Option<i32>,
    #[serde(default)]
    gold_earned: Option<i32>,
    #[serde(default)]
    total_damage_dealt_to_champions: Option<i32>,
    #[serde(default)]
    total_damage_taken: Option<i32>,
    #[serde(default)]
    vision_score: Option<i32>,
    /// 本场最大多杀（双杀=2 … 五杀=5）；LCU 战绩详情常见字段
    #[serde(default)]
    largest_multi_kill: Option<i32>,
    #[serde(default)]
    largest_killing_spree: Option<i32>,
    #[serde(default)]
    total_minions_killed: Option<i32>,
    #[serde(default)]
    neutral_minions_killed: Option<i32>,
    #[serde(default)]
    turret_kills: Option<i32>,
    #[serde(default)]
    inhibitor_kills: Option<i32>,
    #[serde(default)]
    wards_placed: Option<i32>,
    #[serde(default)]
    wards_killed: Option<i32>,
    #[serde(default)]
    damage_dealt_to_turrets: Option<i32>,
    #[serde(default)]
    item0: Option<i32>,
    #[serde(default)]
    item1: Option<i32>,
    #[serde(default)]
    item2: Option<i32>,
    #[serde(default)]
    item3: Option<i32>,
    #[serde(default)]
    item4: Option<i32>,
    #[serde(default)]
    item5: Option<i32>,
    #[serde(default)]
    item6: Option<i32>,
    #[serde(default)]
    perk0: Option<i32>,
    #[serde(default)]
    perk_primary_style: Option<i32>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiParticipantIdentity {
    participant_id: i32,
    #[serde(default)]
    player: Option<ApiPlayer>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ApiPlayer {
    #[serde(default)]
    summoner_name: Option<String>,
    #[serde(default)]
    game_name: Option<String>,
    #[serde(default)]
    tag_line: Option<String>,
    #[serde(default)]
    profile_icon: i64,
}

fn participant_display_name(player: Option<&ApiPlayer>) -> String {
    let Some(player) = player else {
        return "未知玩家".to_owned();
    };

    match (
        player.game_name.as_deref().map(str::trim),
        player.tag_line.as_deref().map(str::trim),
    ) {
        (Some(name), Some(tag)) if !name.is_empty() && !tag.is_empty() => format!("{name}#{tag}"),
        _ => player
            .summoner_name
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or("未知玩家")
            .to_owned(),
    }
}

pub(super) fn map_game_detail(raw_detail: Value) -> Result<GameDetail, String> {
    let api_game_data: ApiGameData =
        serde_json::from_value(raw_detail).map_err(|error| format!("解析游戏详细信息失败: {error}"))?;

    let mut blue_team_stats = TeamStats::default();
    let mut red_team_stats = TeamStats::default();
    let mut max_damage = 0;
    let mut best_player_champion_id = 0;
    let mut max_tank = 0;
    let mut max_tank_champion_id = 0;
    let mut max_streak = 0;
    let mut max_streak_champion_id = 0;

    let player_map: HashMap<i32, ApiPlayer> = api_game_data
        .participant_identities
        .into_iter()
        .filter_map(|identity| identity.player.map(|player| (identity.participant_id, player)))
        .collect();

    let api_participants = api_game_data.participants;
    let mut participants: Vec<ParticipantInfo> = Vec::with_capacity(api_participants.len());
    for p in api_participants {
        // 详情列表不展示分路；对位由过程复盘 / evidence 层单独解析
        let position = None;
        let stats = p.stats;
        let kills = stats.kills.unwrap_or(0);
        let deaths = stats.deaths.unwrap_or(0);
        let assists = stats.assists.unwrap_or(0);
        let damage = stats.total_damage_dealt_to_champions.unwrap_or(0);
        let damage_taken = stats.total_damage_taken.unwrap_or(0);
        let gold = stats.gold_earned.unwrap_or(0);
        let vision = stats.vision_score.unwrap_or(0);
        let champ_level = stats.champ_level.unwrap_or(0);
        let champion_id = p.champion_id.unwrap_or(0);
        let team_id = p.team_id.unwrap_or(0);

        match team_id {
            100 => {
                blue_team_stats.kills += kills;
                blue_team_stats.gold_earned += gold;
                blue_team_stats.total_damage_dealt_to_champions += damage;
                blue_team_stats.vision_score += vision;
            }
            200 => {
                red_team_stats.kills += kills;
                red_team_stats.gold_earned += gold;
                red_team_stats.total_damage_dealt_to_champions += damage;
                red_team_stats.vision_score += vision;
            }
            _ => {}
        }

        if matches!(team_id, 100 | 200) {
            if damage > max_damage {
                max_damage = damage;
                best_player_champion_id = champion_id;
            }
            if damage_taken > max_tank {
                max_tank = damage_taken;
                max_tank_champion_id = champion_id;
            }

            let multi_kill = stats.largest_multi_kill.unwrap_or(0);
            if multi_kill > max_streak {
                max_streak = multi_kill;
                max_streak_champion_id = champion_id;
            }
        }

        let player_identity = player_map.get(&p.participant_id);
        let summoner_name = participant_display_name(player_identity);
        let profile_icon_id = player_identity.map_or(0, |pi| pi.profile_icon);

        let champion_name =
            crate::infrastructure::data_services::champion_data::service::get_champion_info(champion_id)
                .map(|info| info.name);

        participants.push(ParticipantInfo {
            participant_id: p.participant_id,
            champion_id,
            champion_name,
            summoner_name,
            profile_icon_id,
            team_id,
            rank_tier: None,
            spell1_id: p.spell1_id.filter(|&id| id > 0),
            spell2_id: p.spell2_id.filter(|&id| id > 0),
            perk_primary_style: stats.perk_primary_style.filter(|&id| id > 0),
            perk0: stats.perk0.filter(|&id| id > 0),
            position,
            score: None,
            stats: ParticipantStats {
                kills,
                deaths,
                assists,
                champ_level,
                gold_earned: gold,
                total_damage_dealt_to_champions: damage,
                total_damage_taken: damage_taken,
                vision_score: vision,
                total_minions_killed: stats.total_minions_killed.unwrap_or(0),
                neutral_minions_killed: stats.neutral_minions_killed.unwrap_or(0),
                largest_multi_kill: stats.largest_multi_kill.unwrap_or(0),
                largest_killing_spree: stats.largest_killing_spree.unwrap_or(0),
                turret_kills: stats.turret_kills.unwrap_or(0),
                inhibitor_kills: stats.inhibitor_kills.unwrap_or(0),
                wards_placed: stats.wards_placed.unwrap_or(0),
                wards_killed: stats.wards_killed.unwrap_or(0),
                damage_dealt_to_turrets: stats.damage_dealt_to_turrets.unwrap_or(0),
                item0: stats.item0,
                item1: stats.item1,
                item2: stats.item2,
                item3: stats.item3,
                item4: stats.item4,
                item5: stats.item5,
                item6: stats.item6,
            },
        });
    }

    Ok(GameDetail {
        game_id: api_game_data.game_id,
        game_duration: api_game_data.game_duration,
        game_creation: api_game_data.game_creation,
        game_mode: api_game_data.game_mode,
        game_type: api_game_data.game_type,
        game_version: api_game_data.game_version,
        map_id: api_game_data.map_id,
        queue_id: api_game_data.queue_id,
        teams: api_game_data.teams,
        participants,
        blue_team_stats,
        red_team_stats,
        best_player_champion_id,
        max_damage,
        max_tank_champion_id,
        max_tank,
        max_streak_champion_id,
        max_streak,
    })
}

#[cfg(test)]
mod tests {
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
}
