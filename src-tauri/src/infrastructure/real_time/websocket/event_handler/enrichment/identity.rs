use std::collections::HashSet;

use crate::infrastructure::champion_selection::summoner_spells;
use crate::infrastructure::data_services::champion_data;

use crate::infrastructure::real_time::identity::{live_name_match, riot_id_rank, unique_best};
use crate::shared::types::SummonerInfo;

pub(super) fn match_summoner<'a>(summoners: &'a [SummonerInfo], name: &str) -> Option<&'a SummonerInfo> {
    unique_best(summoners, |s| {
        riot_id_rank(
            &canonical_riot_id(&s.display_name, s.game_name.as_deref(), s.tag_line.as_deref()),
            name,
        )
    })
    .ok()
    .flatten()
}

pub(super) fn canonical_riot_id(display_name: &str, game_name: Option<&str>, tag_line: Option<&str>) -> String {
    match (
        game_name.map(str::trim).filter(|value| !value.is_empty()),
        tag_line.map(str::trim).filter(|value| !value.is_empty()),
    ) {
        (Some(game_name), Some(tag_line)) => format!("{game_name}#{tag_line}"),
        (Some(game_name), None) => game_name.to_owned(),
        _ => display_name.trim().to_owned(),
    }
}

pub(super) fn resolve_local_live_team<'a>(
    live_players: &'a [crate::shared::types::LiveClientPlayer],
    team_analysis: &crate::shared::types::TeamAnalysisData,
) -> Option<&'a str> {
    let local_player = team_analysis.my_team.iter().find(|player| player.is_local)?;

    let player = match live_name_match(live_players, &local_player.display_name).ok()? {
        Some(player) => player,
        None => {
            let champion = local_player.champion_id?;
            unique_best(live_players.iter().filter(|p| !p.is_bot), |p| {
                u8::from(champion_data::get_champion_id_by_name(&p.champion_name) == Some(champion))
            })
            .ok()??
        }
    };
    Some(player.team.as_str())
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
    cached_player.analysis = None;
}

pub(super) fn apply_live_player_identity(
    cached_player: &mut crate::shared::types::PlayerAnalysisData,
    live_player: &crate::shared::types::LiveClientPlayer,
    champion_id: i32,
) {
    cached_player.display_name = live_player.summoner_name.clone();
    cached_player.is_bot = false;
    cached_player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Loading;
    cached_player.champion_id = Some(champion_id);
    cached_player.champion_name = Some(live_player.champion_name.clone());
    cached_player.position = live_position(&live_player.position);
    cached_player.spell1_id = live_spell_id(&live_player.summoner_spells, "summonerSpellOne");
    cached_player.spell2_id = live_spell_id(&live_player.summoner_spells, "summonerSpellTwo");
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

pub(super) fn select_team_slot(
    team: &[crate::shared::types::PlayerAnalysisData],
    occupied_slots: &HashSet<usize>,
    live_player: &crate::shared::types::LiveClientPlayer,
    champion_id: i32,
) -> Option<usize> {
    let available = || {
        team.iter()
            .enumerate()
            .filter(|(index, _)| !occupied_slots.contains(index))
    };
    if let Some((index, _)) = unique_best(available(), |(_, p)| {
        riot_id_rank(&p.display_name, &live_player.summoner_name)
    })
    .ok()?
    {
        return Some(index);
    }
    if let Some((index, _)) = unique_best(available(), |(_, p)| u8::from(p.champion_id == Some(champion_id))).ok()? {
        return Some(index);
    }
    available()
        .find(|(_, p)| {
            p.champion_id.is_none() && p.analysis_status == crate::shared::types::PlayerAnalysisStatus::AwaitingIdentity
        })
        .map(|(index, _)| index)
}

#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;
