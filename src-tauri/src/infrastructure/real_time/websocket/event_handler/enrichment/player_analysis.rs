use tauri::Emitter;

use super::super::WsEventHandler;
use super::identity::{live_position, same_riot_id};
use crate::infrastructure::champion_selection::summoner_spells;
use crate::infrastructure::data_services::champion_data;
use crate::shared::Result;

pub(super) type MatchStatsCacheEntry = (
    crate::infrastructure::match_management::analysis_data::service::MatchStatsCacheKey,
    crate::infrastructure::match_management::analysis_data::service::CachedPlayerAnalysis,
);

impl WsEventHandler {
    pub(crate) async fn retry_player_analysis(
        &self,
        puuid: &str,
    ) -> Result<crate::shared::types::PlayerAnalysisStatus> {
        let puuid = puuid.trim();
        if puuid.is_empty() {
            return Err("Player PUUID is required".into());
        }

        let (is_champ_select, generation, queue_id, is_custom_game, perspective) = {
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
            let perspective = if let Some(player) = analysis
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
                if player.is_local {
                    crate::shared::types::AdvicePerspective::SelfImprovement
                } else {
                    crate::shared::types::AdvicePerspective::Collaboration
                }
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
                crate::shared::types::AdvicePerspective::Targeting
            } else {
                return Err("Player is no longer part of the active team analysis".into());
            };

            let updated_data = analysis.clone();
            let _ = self.app.emit("team-analysis-data", &updated_data);
            (is_champ_select, generation, queue_id, is_custom_game, perspective)
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
                player.match_stats = Some(player_analysis.stats.clone());
                player.recent_matches = player_analysis.recent_matches.clone();
                player.analysis_basis = Some(player_analysis.basis.clone());
                player.analysis_status = crate::shared::types::PlayerAnalysisStatus::Ready;
                crate::infrastructure::match_management::analysis_data::service::refresh_ranked_rating(
                    player, queue_id,
                );
                (
                    Some(player_analysis),
                    Ok(crate::shared::types::PlayerAnalysisStatus::Ready),
                )
            }
            Err(error) => {
                player.match_stats = None;
                player.recent_matches.clear();
                player.analysis_basis = None;
                player.ranked_rating = None;
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
                    perspective,
                ),
                player_analysis,
            );
        }
        let _ = self.app.emit("team-analysis-data", &updated_data);
        command_result
    }

    /// 辅助方法：从 LiveClient 玩家数据构建 PlayerAnalysisData
    pub(super) async fn build_player_data(
        &self,
        live_player: &crate::shared::types::LiveClientPlayer,
        cell_id: i32,
        summoners_info: &[crate::shared::types::SummonerInfo],
        queue_id: i64,
        is_custom_game: bool,
        perspective: crate::shared::types::AdvicePerspective,
    ) -> Result<(crate::shared::types::PlayerAnalysisData, Option<MatchStatsCacheEntry>)> {
        // 获取英雄 ID
        let champion_id = champion_data::get_champion_id_by_name(&live_player.champion_name)
            .ok_or_else(|| format!("Could not find champion ID for '{}'", live_player.champion_name))?;

        // 查找召唤师信息
        let summoner_info = summoners_info.iter().find(|s| {
            let full_name = if let (Some(game_name), Some(tag_line)) = (&s.game_name, &s.tag_line) {
                format!("{}#{}", game_name, tag_line)
            } else {
                s.display_name.clone()
            };
            same_riot_id(&full_name, &live_player.summoner_name)
        });

        let mut player_data = crate::shared::types::PlayerAnalysisData {
            cell_id,
            display_name: live_player.summoner_name.clone(),
            summoner_id: None,
            puuid: None,
            is_local: false,
            is_bot: false,
            analysis_status: crate::shared::types::PlayerAnalysisStatus::Loading,
            champion_id: Some(champion_id),
            champion_name: Some(live_player.champion_name.clone()),
            champion_pick_intent: None,
            position: live_position(&live_player.position),
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
        };
        let mut cache_entry = None;

        // 解析召唤师技能
        if let Some(spells) = live_player.summoner_spells.as_object() {
            if let Some(spell_one) = spells.get("summonerSpellOne") {
                if let Some(spell_name) = spell_one.get("displayName").and_then(|v| v.as_str()) {
                    if let Some(spell_id) = summoner_spells::get_spell_id_by_name(spell_name) {
                        player_data.spell1_id = Some(spell_id);
                    }
                }
            }

            if let Some(spell_two) = spells.get("summonerSpellTwo") {
                if let Some(spell_name) = spell_two.get("displayName").and_then(|v| v.as_str()) {
                    if let Some(spell_id) = summoner_spells::get_spell_id_by_name(spell_name) {
                        player_data.spell2_id = Some(spell_id);
                    }
                }
            }
        }

        // 填充召唤师详细信息
        if let Some(info) = summoner_info {
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
                            perspective,
                        ),
                        analysis.clone(),
                    ));
                    player_data.match_stats = Some(analysis.stats);
                    player_data.recent_matches = analysis.recent_matches;
                    player_data.analysis_basis = Some(analysis.basis);
                    player_data.analysis_status = crate::shared::types::PlayerAnalysisStatus::Ready;
                    crate::infrastructure::match_management::analysis_data::service::refresh_ranked_rating(
                        &mut player_data,
                        queue_id,
                    );

                    log::debug!(
                        target: "ws::event_handler",
                        "Fetched match data for player '{}'",
                        live_player.summoner_name
                    );
                }
                Err(e) => {
                    player_data.analysis_status = e.status();
                    log::warn!(
                        target: "ws::event_handler",
                        "Failed to get match history for player '{}': {}",
                        live_player.summoner_name,
                        e
                    );
                }
            }
        } else {
            player_data.analysis_status = crate::shared::types::PlayerAnalysisStatus::Unavailable;
            log::warn!(
                target: "ws::event_handler",
                "Could not find detailed summoner info for '{}'",
                live_player.summoner_name
            );
        }

        Ok((player_data, cache_entry))
    }
}
