use tauri::Emitter;

use super::super::WsEventHandler;
use super::identity::{canonical_riot_id, live_position, live_spell_id, same_riot_id};
use crate::infrastructure::data_services::champion_data;
use crate::shared::Result;

pub(super) type MatchStatsCacheEntry = (
    crate::infrastructure::match_management::analysis_data::service::MatchStatsCacheKey,
    crate::infrastructure::match_management::analysis_data::service::CachedPlayerAnalysis,
);

pub(super) fn base_player_data(
    live_player: &crate::shared::types::LiveClientPlayer,
    cell_id: i32,
    champion_id: Option<i32>,
) -> crate::shared::types::PlayerAnalysisData {
    crate::shared::types::PlayerAnalysisData {
        cell_id,
        display_name: live_player.summoner_name.clone(),
        summoner_id: None,
        puuid: None,
        is_local: false,
        is_bot: live_player.is_bot,
        analysis_status: if live_player.is_bot {
            crate::shared::types::PlayerAnalysisStatus::Bot
        } else {
            crate::shared::types::PlayerAnalysisStatus::Loading
        },
        champion_id,
        champion_name: Some(live_player.champion_name.clone()),
        champion_pick_intent: None,
        position: live_position(&live_player.position),
        solo_rank: None,
        flex_rank: None,
        profile_icon_id: None,
        tag_line: None,
        spell1_id: live_spell_id(&live_player.summoner_spells, "summonerSpellOne"),
        spell2_id: live_spell_id(&live_player.summoner_spells, "summonerSpellTwo"),
        analysis: None,
    }
}

impl WsEventHandler {
    pub(crate) async fn retry_player_analysis(
        &self,
        puuid: &str,
    ) -> Result<crate::shared::types::PlayerAnalysisStatus> {
        let puuid = puuid.trim();
        if puuid.is_empty() {
            return Err("Player PUUID is required".into());
        }

        let (is_champ_select, generation, queue_id, is_custom_game) = {
            let mut cache = self.cache.write().await;
            let (is_champ_select, generation) = match cache.gameflow_phase.as_deref() {
                Some("ChampSelect") => (true, cache.champ_select_analysis_generation),
                Some("InProgress") => (false, cache.in_game_recovery_generation),
                _ => return Err("Player analysis can only be retried during champion select or an active game".into()),
            };
            let analysis = cache
                .team_analysis_data
                .as_mut()
                .ok_or("No active team analysis data is available")?;
            let queue_id = analysis.queue_id;
            let is_custom_game = analysis.is_custom_game;
            if let Some(player) = analysis
                .my_team
                .iter_mut()
                .find(|player| player.puuid.as_deref() == Some(puuid))
            {
                if player.is_bot {
                    return Err("Bot players do not require match analysis".into());
                }
                if player.analysis_status == crate::shared::types::PlayerAnalysisStatus::Loading {
                    return Ok(crate::shared::types::PlayerAnalysisStatus::Loading);
                }
                player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Loading;
            } else if let Some(player) = analysis
                .enemy_team
                .iter_mut()
                .find(|player| player.puuid.as_deref() == Some(puuid))
            {
                if player.is_bot {
                    return Err("Bot players do not require match analysis".into());
                }
                if player.analysis_status == crate::shared::types::PlayerAnalysisStatus::Loading {
                    return Ok(crate::shared::types::PlayerAnalysisStatus::Loading);
                }
                player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Loading;
            } else {
                return Err("Player is no longer part of the active team analysis".into());
            }

            let updated_data = analysis.clone();
            let _ = self.app.emit("team-analysis-data", &updated_data);
            (is_champ_select, generation, queue_id, is_custom_game)
        };

        let result =
            crate::infrastructure::match_management::analysis_data::service::fetch_realtime_player_analysis_with_retry(
                &self.client,
                puuid,
                queue_id,
                is_custom_game,
            )
            .await;

        let mut cache = self.cache.write().await;
        let can_commit = if is_champ_select {
            cache.gameflow_phase.as_deref() == Some("ChampSelect") && cache.can_commit_champ_select_analysis(generation)
        } else {
            cache.can_commit_in_game_recovery(generation)
        };
        if !can_commit {
            return Err("The active game changed while the player was being analyzed".into());
        }

        let analysis = cache
            .team_analysis_data
            .as_mut()
            .ok_or("Team analysis data was cleared while retrying the player")?;
        let player = analysis
            .my_team
            .iter_mut()
            .chain(analysis.enemy_team.iter_mut())
            .find(|player| player.puuid.as_deref() == Some(puuid))
            .ok_or("Player left the active team analysis while the retry was running")?;

        let (cache_entry, command_result) = match result {
            Ok(player_analysis) => {
                crate::infrastructure::match_management::analysis_data::service::apply_player_analysis(
                    player,
                    player_analysis.clone(),
                    queue_id,
                );
                (
                    Some(player_analysis),
                    Ok(crate::shared::types::PlayerAnalysisStatus::Ready),
                )
            }
            Err(error) => {
                crate::infrastructure::match_management::analysis_data::service::clear_player_analysis(player);
                player.analysis_status = error.status();
                let status = error.status();
                let command_result = if error.is_unavailable() {
                    Err(error.to_string().into())
                } else {
                    Ok(status)
                };
                (None, command_result)
            }
        };

        let updated_data = analysis.clone();
        if let Some(player_analysis) = cache_entry {
            cache.match_stats_cache.insert(
                crate::infrastructure::match_management::analysis_data::service::MatchStatsCacheKey::new(
                    puuid,
                    queue_id,
                    is_custom_game,
                ),
                player_analysis,
            );
        }
        let _ = self.app.emit("team-analysis-data", &updated_data);
        command_result
    }

    /// Builds the best available player card from the authoritative LiveClient roster.
    pub(super) async fn build_player_data(
        &self,
        live_player: &crate::shared::types::LiveClientPlayer,
        cell_id: i32,
        summoners_info: &[crate::shared::types::SummonerInfo],
        queue_id: i64,
        is_custom_game: bool,
    ) -> (crate::shared::types::PlayerAnalysisData, Option<MatchStatsCacheEntry>) {
        let champion_id = champion_data::get_champion_id_by_name(&live_player.champion_name);
        if champion_id.is_none() {
            log::warn!(
                target: "ws::event",
                "Champion '{}' is absent from the current static catalog; keeping the player with an unresolved champion ID",
                live_player.champion_name
            );
        }

        let mut player_data = base_player_data(live_player, cell_id, champion_id);
        if live_player.is_bot {
            return (player_data, None);
        }

        // 查找召唤师信息
        let summoner_info = summoners_info.iter().find(|s| {
            let full_name = if let (Some(game_name), Some(tag_line)) = (&s.game_name, &s.tag_line) {
                format!("{}#{}", game_name, tag_line)
            } else {
                s.display_name.clone()
            };
            same_riot_id(&full_name, &live_player.summoner_name)
        });

        let mut cache_entry = None;

        // 填充召唤师详细信息
        if let Some(info) = summoner_info {
            player_data.display_name =
                canonical_riot_id(&info.display_name, info.game_name.as_deref(), info.tag_line.as_deref());
            player_data.summoner_id = Some(info.summoner_id.clone());
            player_data.puuid = Some(info.puuid.clone());
            crate::infrastructure::match_management::analysis_data::service::apply_rank_summaries(
                &mut player_data,
                info,
            );
            player_data.profile_icon_id = Some(info.profile_icon_id as i32);
            player_data.tag_line = info.tag_line.clone();

            // 获取战绩数据
            match crate::infrastructure::match_management::analysis_data::service::fetch_realtime_player_analysis_with_retry(
                &self.client,
                &info.puuid,
                queue_id,
                is_custom_game,
            )
            .await
            {
                Ok(analysis) => {
                    cache_entry = Some((
                        crate::infrastructure::match_management::analysis_data::service::MatchStatsCacheKey::new(
                            &info.puuid,
                            queue_id,
                            is_custom_game,
                        ),
                        analysis.clone(),
                    ));
                    crate::infrastructure::match_management::analysis_data::service::apply_player_analysis(
                        &mut player_data,
                        analysis,
                        queue_id,
                    );

                    log::debug!(
                        target: "ws::event",
                        "Fetched match data for player '{}'",
                        live_player.summoner_name
                    );
                }
                Err(e) => {
                    player_data.analysis_status = e.status();
                    log::warn!(
                        target: "ws::event",
                        "Failed to get match history for player '{}': {}",
                        live_player.summoner_name,
                        e
                    );
                }
            }
        } else {
            player_data.analysis_status = crate::shared::types::PlayerAnalysisStatus::Unavailable;
            log::warn!(
                target: "ws::event",
                "Could not find detailed summoner info for '{}'",
                live_player.summoner_name
            );
        }

        (player_data, cache_entry)
    }
}

#[cfg(test)]
mod tests {
    use super::base_player_data;
    use crate::shared::types::{LiveClientPlayer, PlayerAnalysisStatus};

    fn live_player(is_bot: bool) -> LiveClientPlayer {
        LiveClientPlayer {
            summoner_name: "Player#CN1".to_owned(),
            riot_id: None,
            riot_id_game_name: None,
            riot_id_tag_line: None,
            champion_name: "Catalog Miss".to_owned(),
            is_bot,
            is_dead: false,
            items: Vec::new(),
            level: 1,
            position: "MIDDLE".to_owned(),
            raw_champion_name: String::new(),
            respawn_timer: 0.0,
            runes: serde_json::Value::Null,
            scores: serde_json::Value::Null,
            skin_id: 0,
            summoner_spells: serde_json::Value::Null,
            team: "ORDER".to_owned(),
        }
    }

    #[test]
    fn base_player_data_keeps_player_when_champion_id_is_unresolved() {
        let player = base_player_data(&live_player(false), 3, None);

        assert_eq!(
            (player.display_name.as_str(), player.champion_id, player.analysis_status),
            ("Player#CN1", None, PlayerAnalysisStatus::Loading)
        );
    }

    #[test]
    fn base_player_data_marks_bot_without_requesting_identity() {
        let player = base_player_data(&live_player(true), 103, Some(55));

        assert_eq!(
            (player.is_bot, player.analysis_status, player.champion_id),
            (true, PlayerAnalysisStatus::Bot, Some(55))
        );
    }
}
