use super::{canonical_riot_id, live_position, same_riot_id, select_team_slot};
use crate::shared::types::{LiveClientPlayer, PlayerAnalysisData, PlayerAnalysisStatus};
use std::collections::HashSet;

fn anonymous_enemy(cell_id: i32) -> PlayerAnalysisData {
    PlayerAnalysisData {
        cell_id,
        display_name: "敌方选手".to_owned(),
        summoner_id: None,
        puuid: None,
        is_local: false,
        is_bot: false,
        analysis_status: PlayerAnalysisStatus::AwaitingIdentity,
        champion_id: None,
        champion_name: None,
        champion_pick_intent: None,
        position: None,
        solo_rank: None,
        flex_rank: None,
        profile_icon_id: None,
        tag_line: None,
        spell1_id: None,
        spell2_id: None,
        analysis: None,
    }
}

fn live_player(name: &str, champion_name: &str) -> LiveClientPlayer {
    LiveClientPlayer {
        summoner_name: name.to_owned(),
        riot_id: None,
        riot_id_game_name: None,
        riot_id_tag_line: None,
        champion_name: champion_name.to_owned(),
        is_bot: false,
        is_dead: false,
        items: Vec::new(),
        level: 1,
        position: String::new(),
        raw_champion_name: String::new(),
        respawn_timer: 0.0,
        runes: serde_json::Value::Null,
        scores: serde_json::Value::Null,
        skin_id: 0,
        summoner_spells: serde_json::Value::Null,
        team: "CHAOS".to_owned(),
    }
}

#[test]
fn riot_id_matching_preserves_tag_when_both_sources_have_one() {
    assert!(same_riot_id("bbs#23912", "BBS#23912"));
    assert!(!same_riot_id("bbs#23912", "bbs#other"));
    assert!(same_riot_id("bbs", "bbs#23912"));
}

#[test]
fn canonical_riot_id_prefers_game_name_and_tag_line() {
    assert_eq!(
        canonical_riot_id("legacy name", Some("bbs"), Some("23912")),
        "bbs#23912"
    );
    assert_eq!(canonical_riot_id("legacy name", Some("bbs"), None), "bbs");
    assert_eq!(canonical_riot_id(" legacy name ", None, Some("23912")), "legacy name");
}

#[test]
fn live_position_omits_none_sentinel() {
    assert_eq!(live_position("NONE"), None);
    assert_eq!(live_position("JUNGLE").as_deref(), Some("JUNGLE"));
}

#[test]
fn anonymous_team_slots_are_assigned_once_in_stable_order() {
    let enemies = vec![anonymous_enemy(0), anonymous_enemy(1)];
    let first = live_player("first#CN1", "Katarina");
    let second = live_player("second#CN1", "Ezreal");
    let mut occupied = HashSet::new();

    let first_index = select_team_slot(&enemies, &occupied, &first, 55).unwrap();
    occupied.insert(first_index);
    let second_index = select_team_slot(&enemies, &occupied, &second, 81).unwrap();

    assert_eq!(first_index, 0);
    assert_eq!(second_index, 1);
}

#[test]
fn known_team_identity_wins_over_anonymous_slot() {
    let mut known = anonymous_enemy(1);
    known.display_name = "Known#CN1".to_owned();
    known.analysis_status = PlayerAnalysisStatus::Loading;
    let enemies = vec![anonymous_enemy(0), known];

    let index = select_team_slot(&enemies, &HashSet::new(), &live_player("known#CN1", "Ezreal"), 81);

    assert_eq!(index, Some(1));
}
