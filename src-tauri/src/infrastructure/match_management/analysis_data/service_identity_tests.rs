use super::{
    build_team_roster_from_session, canonical_assigned_position, champ_select_analysis_key,
    classify_champ_select_identity, parse_lcu_id_i64, patch_team_analysis_from_session, ChampSelectIdentity,
    ChampSelectIdentityFields,
};
use crate::shared::types::{PlayerAnalysisData, PlayerAnalysisStatus, PlayerRankSummary, TeamAnalysisData};
use serde_json::json;

fn classify(
    player_type: &str,
    summoner_id_num: i64,
    game_name: &str,
    puuid: &str,
    name_visibility: &str,
    is_custom_game: bool,
    is_humanoid: bool,
) -> ChampSelectIdentity {
    classify_champ_select_identity(ChampSelectIdentityFields {
        player_type,
        summoner_id_num,
        game_name,
        puuid,
        name_visibility,
        is_custom_game,
        is_humanoid,
    })
}

#[test]
fn ranked_hidden_enemy_is_obfuscated_human_not_bot() {
    let id = classify("HUMAN", 0, "", "", "HIDDEN", false, false);
    assert!(!id.is_bot);
    assert!(id.is_obfuscated);
}

#[test]
fn custom_game_empty_identity_is_bot() {
    let id = classify("", 0, "", "", "", true, false);
    assert!(id.is_bot);
    assert!(!id.is_obfuscated);
}

#[test]
fn custom_visible_humanoid_with_synthetic_puuid_is_bot() {
    let id = classify("", 0, "", "4d68d840-35b8-4fb6-b76a-98840f3352fa", "VISIBLE", true, true);
    assert!(id.is_bot);
    assert!(!id.is_obfuscated);
}

#[test]
fn custom_hidden_human_without_visible_account_id_is_not_bot() {
    let id = classify("HUMAN", 0, "", "", "HIDDEN", true, false);
    assert!(!id.is_bot);
    assert!(id.is_obfuscated);
}

#[test]
fn custom_hidden_slot_without_identity_remains_awaiting_identity() {
    let id = classify("", 0, "", "", "HIDDEN", true, false);
    assert!(!id.is_bot);
    assert!(id.is_obfuscated);
}

#[test]
fn explicit_bot_player_type_is_bot() {
    let id = classify("BOT", 0, "", "", "HIDDEN", false, false);
    assert!(id.is_bot);
    assert!(!id.is_obfuscated);
}

#[test]
fn normal_human_keeps_identity() {
    let id = classify(
        "HUMAN",
        12345,
        "SummonerOne",
        "e21feb65-440d-566c-bab4-c9d5af9497c0",
        "VISIBLE",
        false,
        false,
    );
    assert!(!id.is_bot);
    assert!(!id.is_obfuscated);
}

#[test]
fn string_summoner_id_keeps_custom_hidden_player_human() {
    // LCU 偶发把 summonerId 序列化为字符串；旧逻辑 as_i64()→0 会误判自定义局隐名人为 bot
    let summoner_id_num = parse_lcu_id_i64(&json!("12345"));
    assert_eq!(summoner_id_num, 12345);
    let id = classify("", summoner_id_num, "", "", "HIDDEN", true, false);
    assert!(!id.is_bot);
    assert!(id.is_obfuscated);
}

#[test]
fn parse_lcu_id_accepts_number_and_string() {
    assert_eq!(parse_lcu_id_i64(&json!(42)), 42);
    assert_eq!(parse_lcu_id_i64(&json!("42")), 42);
    assert_eq!(parse_lcu_id_i64(&json!(0)), 0);
    assert_eq!(parse_lcu_id_i64(&json!("0")), 0);
    assert_eq!(parse_lcu_id_i64(&json!(null)), 0);
}

#[test]
fn volatile_champ_select_updates_keep_the_same_analysis_key() {
    let base = json!({
        "gameId": 123,
        "localPlayerCellId": 0,
        "queueId": 440,
        "isCustomGame": false,
        "myTeam": [{
            "cellId": 0,
            "summonerId": 42,
            "puuid": "player-a",
            "gameName": "Player",
            "tagLine": "CN1",
            "playerType": "HUMAN",
            "nameVisibilityType": "VISIBLE",
            "championId": 67,
            "championPickIntent": 67,
            "assignedPosition": "jungle",
            "spell1Id": 4,
            "spell2Id": 11
        }],
        "theirTeam": [],
        "timer": { "adjustedTimeLeftInPhase": 10000 }
    });
    let mut updated = base.clone();
    updated["myTeam"][0]["championId"] = json!(22);
    updated["myTeam"][0]["championPickIntent"] = json!(22);
    updated["myTeam"][0]["assignedPosition"] = json!("bottom");
    updated["myTeam"][0]["spell2Id"] = json!(7);
    updated["timer"]["adjustedTimeLeftInPhase"] = json!(5000);

    assert_eq!(champ_select_analysis_key(&base), champ_select_analysis_key(&updated));

    updated["myTeam"][0]["puuid"] = json!("player-b");
    assert_ne!(champ_select_analysis_key(&base), champ_select_analysis_key(&updated));
}

#[test]
fn initial_roster_is_publishable_before_network_enrichment() {
    let roster = build_team_roster_from_session(&json!({
        "localPlayerCellId": 0,
        "queueId": 420,
        "isCustomGame": false,
        "myTeam": [{
            "cellId": 0,
            "displayName": "Player#CN1",
            "summonerId": 42,
            "puuid": "player-a",
            "playerType": "HUMAN",
            "nameVisibilityType": "VISIBLE",
            "championId": 67,
            "assignedPosition": "jungle"
        }],
        "theirTeam": [{
            "cellId": 5,
            "summonerId": 0,
            "puuid": "",
            "playerType": "HUMAN",
            "nameVisibilityType": "HIDDEN"
        }]
    }));

    assert_eq!(roster.my_team[0].analysis_status, PlayerAnalysisStatus::Loading);
    assert_eq!(
        roster.enemy_team[0].analysis_status,
        PlayerAnalysisStatus::AwaitingIdentity
    );
}

#[test]
fn volatile_session_patch_preserves_enriched_player_data() {
    let mut analysis = TeamAnalysisData {
        my_team: vec![PlayerAnalysisData {
            cell_id: 0,
            display_name: "Player#CN1".to_string(),
            summoner_id: Some("42".to_string()),
            puuid: Some("player-a".to_string()),
            is_local: true,
            is_bot: false,
            analysis_status: PlayerAnalysisStatus::Ready,
            champion_id: Some(67),
            champion_name: None,
            champion_pick_intent: Some(67),
            position: Some("JUNGLE".to_string()),
            solo_rank: Some(PlayerRankSummary {
                tier: "EMERALD".to_string(),
                division: Some("I".to_string()),
                league_points: Some(19),
            }),
            flex_rank: None,
            profile_icon_id: Some(4977),
            tag_line: Some("CN1".to_string()),
            spell1_id: Some(4),
            spell2_id: Some(11),
            analysis: None,
        }],
        enemy_team: Vec::new(),
        local_player_cell_id: 0,
        game_phase: "ChampSelect".to_string(),
        queue_id: 440,
        is_custom_game: false,
        actions: None,
        bans: None,
        timer: None,
    };
    let session = json!({
        "myTeam": [{
            "cellId": 0,
            "championId": 22,
            "championPickIntent": 22,
            "assignedPosition": "bottom",
            "spell1Id": 4,
            "spell2Id": 7
        }],
        "theirTeam": []
    });

    assert!(patch_team_analysis_from_session(&mut analysis, &session));
    let player = &analysis.my_team[0];
    assert_eq!(player.champion_id, Some(22));
    assert_eq!(player.position.as_deref(), Some("ADC"));
    assert_eq!(player.spell2_id, Some(7));
    assert_eq!(player.display_name, "Player#CN1");
    assert_eq!(
        player.solo_rank.as_ref().map(|rank| rank.tier.as_str()),
        Some("EMERALD")
    );
    assert_eq!(player.analysis_status, PlayerAnalysisStatus::Ready);
}

#[test]
fn assigned_position_is_exposed_as_canonical_rune_position() {
    assert_eq!(canonical_assigned_position(&json!("top")).as_deref(), Some("TOP"));
    assert_eq!(canonical_assigned_position(&json!("jungle")).as_deref(), Some("JUNGLE"));
    assert_eq!(canonical_assigned_position(&json!("middle")).as_deref(), Some("MID"));
    assert_eq!(canonical_assigned_position(&json!("bottom")).as_deref(), Some("ADC"));
    assert_eq!(
        canonical_assigned_position(&json!("utility")).as_deref(),
        Some("SUPPORT")
    );
    assert_eq!(canonical_assigned_position(&json!("unknown")), None);
}
