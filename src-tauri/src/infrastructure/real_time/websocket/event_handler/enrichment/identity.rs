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
#[path = "identity_tests.rs"]
mod tests;
