use std::collections::HashSet;

use crate::infrastructure::champion_selection::summoner_spells;
use crate::infrastructure::data_services::champion_data;

pub(super) fn same_riot_id(left: &str, right: &str) -> bool {
    let left = left.trim();
    let right = right.trim();
    if left.eq_ignore_ascii_case(right) {
        return true;
    }

    let (left_name, left_has_tag) = left.split_once('#').map_or((left, false), |(name, _)| (name, true));
    let (right_name, right_has_tag) = right.split_once('#').map_or((right, false), |(name, _)| (name, true));

    // Only fall back to the game name when one source omitted the tag line.
    (!left_has_tag || !right_has_tag) && left_name.eq_ignore_ascii_case(right_name)
}

pub(super) fn resolve_local_live_team<'a>(
    live_players: &'a [crate::shared::types::LiveClientPlayer],
    team_analysis: &crate::shared::types::TeamAnalysisData,
) -> Option<&'a str> {
    let local_player = team_analysis.my_team.iter().find(|player| player.is_local)?;

    live_players
        .iter()
        .find(|player| !player.is_bot && same_riot_id(&local_player.display_name, &player.summoner_name))
        .or_else(|| {
            let local_champion_id = local_player.champion_id?;
            live_players.iter().find(|player| {
                !player.is_bot
                    && champion_data::get_champion_id_by_name(&player.champion_name) == Some(local_champion_id)
            })
        })
        .map(|player| player.team.as_str())
}

pub(super) fn apply_confirmed_bot_identity(
    cached_player: &mut crate::shared::types::PlayerAnalysisData,
    live_player: &crate::shared::types::LiveClientPlayer,
    champion_id: i32,
) {
    cached_player.display_name = live_player.summoner_name.clone();
    cached_player.is_bot = true;
    cached_player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Bot;
    cached_player.champion_id = Some(champion_id);
    cached_player.champion_name = Some(live_player.champion_name.clone());
    cached_player.position = live_position(&live_player.position);
    cached_player.spell1_id = live_spell_id(&live_player.summoner_spells, "summonerSpellOne");
    cached_player.spell2_id = live_spell_id(&live_player.summoner_spells, "summonerSpellTwo");
    cached_player.match_stats = None;
    cached_player.recent_matches.clear();
    cached_player.analysis_basis = None;
}

pub(super) fn live_spell_id(spells: &serde_json::Value, slot: &str) -> Option<i64> {
    spells
        .get(slot)?
        .get("displayName")?
        .as_str()
        .and_then(summoner_spells::get_spell_id_by_name)
}

pub(super) fn live_position(position: &str) -> Option<String> {
    let position = position.trim();
    (!position.is_empty() && !position.eq_ignore_ascii_case("NONE")).then(|| position.to_owned())
}

pub(super) fn select_enemy_slot(
    enemy_team: &[crate::shared::types::PlayerAnalysisData],
    occupied_slots: &HashSet<usize>,
    live_player: &crate::shared::types::LiveClientPlayer,
    champion_id: i32,
) -> Option<usize> {
    let available = |index: &usize| !occupied_slots.contains(index);

    enemy_team
        .iter()
        .enumerate()
        .filter(|(index, _)| available(index))
        .find(|(_, player)| same_riot_id(&player.display_name, &live_player.summoner_name))
        .or_else(|| {
            enemy_team
                .iter()
                .enumerate()
                .filter(|(index, _)| available(index))
                .find(|(_, player)| player.champion_id == Some(champion_id))
        })
        .or_else(|| {
            enemy_team
                .iter()
                .enumerate()
                .filter(|(index, _)| available(index))
                .find(|(_, player)| {
                    player.analysis_status == crate::shared::types::PlayerAnalysisStatus::AwaitingIdentity
                })
        })
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::{live_position, same_riot_id, select_enemy_slot};
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
            match_stats: None,
            recent_matches: Vec::new(),
            analysis_basis: None,
            ranked_rating: None,
        }
    }

    fn live_player(name: &str, champion_name: &str) -> LiveClientPlayer {
        LiveClientPlayer {
            summoner_name: name.to_owned(),
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
    fn live_position_omits_none_sentinel() {
        assert_eq!(live_position("NONE"), None);
        assert_eq!(live_position("JUNGLE").as_deref(), Some("JUNGLE"));
    }

    #[test]
    fn anonymous_enemy_slots_are_assigned_once_in_stable_order() {
        let enemies = vec![anonymous_enemy(0), anonymous_enemy(1)];
        let first = live_player("first#CN1", "Katarina");
        let second = live_player("second#CN1", "Ezreal");
        let mut occupied = HashSet::new();

        let first_index = select_enemy_slot(&enemies, &occupied, &first, 55).unwrap();
        occupied.insert(first_index);
        let second_index = select_enemy_slot(&enemies, &occupied, &second, 81).unwrap();

        assert_eq!(first_index, 0);
        assert_eq!(second_index, 1);
    }

    #[test]
    fn known_enemy_identity_wins_over_anonymous_slot() {
        let mut known = anonymous_enemy(1);
        known.display_name = "Known#CN1".to_owned();
        known.analysis_status = PlayerAnalysisStatus::Loading;
        let enemies = vec![anonymous_enemy(0), known];

        let index = select_enemy_slot(&enemies, &HashSet::new(), &live_player("known#CN1", "Ezreal"), 81);

        assert_eq!(index, Some(1));
    }
}
