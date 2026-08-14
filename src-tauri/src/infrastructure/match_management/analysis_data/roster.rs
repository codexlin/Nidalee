use futures_util::stream::{self, StreamExt};

use crate::infrastructure::data_services::summoner::service::get_summoner_by_id;
use crate::shared::types::{PlayerAnalysisData, PlayerAnalysisStatus, PlayerRankSummary, TeamAnalysisData};

use super::history;

const PLAYER_ENRICH_CONCURRENCY: usize = 4;

pub(crate) fn champ_select_analysis_key(session: &serde_json::Value) -> String {
    fn team_identity(team: Option<&Vec<serde_json::Value>>) -> Vec<serde_json::Value> {
        team.into_iter()
            .flatten()
            .map(|player| {
                serde_json::json!({
                    "cellId": player.get("cellId"),
                    "summonerId": player.get("summonerId"),
                    "puuid": player.get("puuid"),
                    "gameName": player.get("gameName"),
                    "tagLine": player.get("tagLine"),
                    "displayName": player.get("displayName"),
                    "playerType": player.get("playerType"),
                    "nameVisibilityType": player.get("nameVisibilityType"),
                    "isHumanoid": player.get("isHumanoid"),
                })
            })
            .collect()
    }

    serde_json::to_string(&serde_json::json!({
        "gameId": session.get("gameId"),
        "localPlayerCellId": session.get("localPlayerCellId"),
        "queueId": session.get("queueId"),
        "isCustomGame": session.get("isCustomGame"),
        "myTeam": team_identity(session.get("myTeam").and_then(serde_json::Value::as_array)),
        "theirTeam": team_identity(session.get("theirTeam").and_then(serde_json::Value::as_array)),
    }))
    .unwrap_or_default()
}

/// Merge volatile ChampSelect fields into an already enriched roster without losing ranks or
/// match history. Returns true only when the presentation data actually changed.
pub(crate) fn patch_team_analysis_from_session(analysis: &mut TeamAnalysisData, session: &serde_json::Value) -> bool {
    fn positive_i32(value: &serde_json::Value) -> Option<i32> {
        value.as_i64().filter(|id| *id > 0).map(|id| id as i32)
    }

    fn positive_i64(value: &serde_json::Value) -> Option<i64> {
        value.as_i64().filter(|id| *id > 0)
    }

    fn non_empty_string(value: &serde_json::Value) -> Option<String> {
        value.as_str().filter(|value| !value.is_empty()).map(str::to_owned)
    }

    fn patch_team(players: &mut [PlayerAnalysisData], raw_team: Option<&Vec<serde_json::Value>>) -> bool {
        let mut changed = false;
        for raw_player in raw_team.into_iter().flatten() {
            let Some(cell_id) = raw_player.get("cellId").and_then(serde_json::Value::as_i64) else {
                continue;
            };
            let Some(player) = players.iter_mut().find(|player| player.cell_id == cell_id as i32) else {
                continue;
            };

            let champion_id = positive_i32(&raw_player["championId"]);
            let champion_pick_intent = positive_i32(&raw_player["championPickIntent"]);
            let position = canonical_assigned_position(&raw_player["assignedPosition"]);
            let spell1_id = positive_i64(&raw_player["spell1Id"]);
            let spell2_id = positive_i64(&raw_player["spell2Id"]);
            let champion_name = non_empty_string(&raw_player["championName"])
                .or_else(|| {
                    champion_id.and_then(|id| {
                        crate::infrastructure::data_services::champion_data::service::get_champion_info(id)
                            .map(|champion| champion.name)
                    })
                })
                .or_else(|| {
                    (player.champion_id == champion_id)
                        .then(|| player.champion_name.clone())
                        .flatten()
                });

            changed |= player.champion_id != champion_id
                || player.champion_name != champion_name
                || player.champion_pick_intent != champion_pick_intent
                || player.position != position
                || player.spell1_id != spell1_id
                || player.spell2_id != spell2_id;
            player.champion_id = champion_id;
            player.champion_name = champion_name;
            player.champion_pick_intent = champion_pick_intent;
            player.position = position;
            player.spell1_id = spell1_id;
            player.spell2_id = spell2_id;
            history::reproject_player_analysis(player);
        }
        changed
    }

    patch_team(
        &mut analysis.my_team,
        session.get("myTeam").and_then(serde_json::Value::as_array),
    ) | patch_team(
        &mut analysis.enemy_team,
        session.get("theirTeam").and_then(serde_json::Value::as_array),
    )
}

pub(crate) fn canonical_assigned_position(value: &serde_json::Value) -> Option<String> {
    let position = match value.as_str()? {
        "top" => "TOP",
        "jungle" => "JUNGLE",
        "middle" => "MID",
        "bottom" => "ADC",
        "utility" => "SUPPORT",
        _ => return None,
    };
    Some(position.to_owned())
}

pub(crate) fn apply_rank_summaries(player: &mut PlayerAnalysisData, summoner: &crate::shared::types::SummonerInfo) {
    player.solo_rank = summoner.solo_rank_tier.as_ref().map(|tier| PlayerRankSummary {
        tier: tier.clone(),
        division: summoner.solo_rank_division.clone(),
        league_points: summoner.solo_rank_lp,
    });
    player.flex_rank = summoner.flex_rank_tier.as_ref().map(|tier| PlayerRankSummary {
        tier: tier.clone(),
        division: summoner.flex_rank_division.clone(),
        league_points: summoner.flex_rank_lp,
    });
}

/// Builds a publishable roster without performing any network I/O.
///
/// The websocket layer publishes this snapshot first so names, slots, champions and loading
/// states are visible while summoner and history enrichment continues in the background.
pub(crate) fn build_team_roster_from_session(session: &serde_json::Value) -> TeamAnalysisData {
    let local_player_cell_id = session["localPlayerCellId"].as_i64().unwrap_or(0) as i32;
    let is_custom_game = session["isCustomGame"].as_bool().unwrap_or(false);
    let parse_team = |team: Option<&Vec<serde_json::Value>>| {
        team.into_iter()
            .flatten()
            .filter_map(|player| {
                parse_player_from_session(player, local_player_cell_id, is_custom_game)
                    .inspect_err(|error| log::warn!(target: "analysis::team", "Failed to parse roster player: {error}"))
                    .ok()
            })
            .collect()
    };

    build_team_analysis_snapshot(
        session,
        parse_team(session.get("myTeam").and_then(serde_json::Value::as_array)),
        parse_team(session.get("theirTeam").and_then(serde_json::Value::as_array)),
    )
}

pub(super) fn build_team_analysis_snapshot(
    session: &serde_json::Value,
    my_team: Vec<PlayerAnalysisData>,
    enemy_team: Vec<PlayerAnalysisData>,
) -> TeamAnalysisData {
    TeamAnalysisData {
        my_team,
        enemy_team,
        local_player_cell_id: session["localPlayerCellId"].as_i64().unwrap_or(0) as i32,
        game_phase: "ChampSelect".to_owned(),
        queue_id: session["queueId"].as_i64().unwrap_or(0),
        is_custom_game: session["isCustomGame"].as_bool().unwrap_or(false),
        actions: session
            .get("actions")
            .and_then(|value| serde_json::from_value(value.clone()).ok()),
        bans: session
            .get("bans")
            .and_then(|value| serde_json::from_value(value.clone()).ok()),
        timer: session
            .get("timer")
            .and_then(|value| serde_json::from_value(value.clone()).ok()),
    }
}
pub(super) async fn parse_and_enrich_team(
    team: Option<&Vec<serde_json::Value>>,
    team_name: &'static str,
    local_player_cell_id: i32,
    is_custom_game: bool,
    http_client: &reqwest::Client,
) -> Vec<PlayerAnalysisData> {
    let Some(team) = team else {
        log::warn!(target: "analysis::team", "Session missing '{}' array", team_name);
        return Vec::new();
    };

    let mut players = stream::iter(team.iter().cloned().enumerate())
        .map(|(index, raw_player)| async move {
            let mut player = match parse_player_from_session(&raw_player, local_player_cell_id, is_custom_game) {
                Ok(player) => player,
                Err(error) => {
                    log::warn!(
                        target: "analysis::team",
                        "Failed to parse {} player[{}]: {}",
                        team_name,
                        index,
                        error
                    );
                    return None;
                }
            };

            if let Err(error) = enrich_player_data(&mut player, &raw_player, http_client).await {
                log::warn!(
                    target: "analysis::team",
                    "Failed to enrich {} player[{}]: {}; keeping basic data",
                    team_name,
                    index,
                    error
                );
            }
            Some((index, player))
        })
        .buffer_unordered(PLAYER_ENRICH_CONCURRENCY)
        .filter_map(|player| async move { player })
        .collect::<Vec<_>>()
        .await;
    players.sort_unstable_by_key(|(index, _)| *index);
    players.into_iter().map(|(_, player)| player).collect()
}

/// 选人阶段身份分类结果
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChampSelectIdentity {
    pub is_bot: bool,
    pub is_obfuscated: bool,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ChampSelectIdentityFields<'a> {
    pub player_type: &'a str,
    pub summoner_id_num: i64,
    pub game_name: &'a str,
    pub puuid: &'a str,
    pub name_visibility: &'a str,
    pub is_custom_game: bool,
    pub is_humanoid: bool,
}

/// 机器人 vs 选人隐名：
/// - 征召敌方 summonerId=0 / 空 puuid / HIDDEN 是 LCU 匿名，**不是** bot
/// - 自定义我方机器人会带合成 puuid，不能用 puuid 是否为空判断真人
/// - 实测可见机器人是 isHumanoid=true + 非正 summonerId；HIDDEN 敌方留给 LiveClient 最终确认
pub(crate) fn classify_champ_select_identity(fields: ChampSelectIdentityFields<'_>) -> ChampSelectIdentity {
    let is_explicit_bot =
        fields.player_type.eq_ignore_ascii_case("BOT") || fields.player_type.eq_ignore_ascii_case("NPC");
    let is_explicit_human = fields.player_type.eq_ignore_ascii_case("HUMAN");
    let is_hidden = fields.name_visibility.eq_ignore_ascii_case("HIDDEN");
    let is_visible_custom_bot = fields.is_custom_game
        && !is_explicit_human
        && !is_hidden
        && fields.summoner_id_num <= 0
        && (fields.is_humanoid || (fields.game_name.is_empty() && fields.puuid.is_empty()));
    let is_custom_bot = is_visible_custom_bot;
    let is_bot = is_explicit_bot || is_custom_bot;
    let is_obfuscated = !is_bot && (is_hidden || (fields.game_name.is_empty() && fields.puuid.is_empty()));
    ChampSelectIdentity { is_bot, is_obfuscated }
}

/// 兼容 LCU JSON 里 id 可能是 number 或 string（含超大 u64）。
/// 分类只关心是否为正数账号 ID；无法解析时按 0（匿名/缺失）。
pub(crate) fn parse_lcu_id_i64(value: &serde_json::Value) -> i64 {
    if let Some(n) = value.as_i64() {
        return n;
    }
    if let Some(n) = value.as_u64() {
        return i64::try_from(n).unwrap_or(i64::MAX);
    }
    if let Some(s) = value.as_str() {
        if let Ok(n) = s.parse::<i64>() {
            return n;
        }
        if let Ok(n) = s.parse::<u64>() {
            return i64::try_from(n).unwrap_or(i64::MAX);
        }
    }
    0
}

fn lcu_id_as_string(value: &serde_json::Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        if s.is_empty() {
            return None;
        }
        return Some(s.to_string());
    }
    if let Some(n) = value.as_i64() {
        return Some(n.to_string());
    }
    if let Some(n) = value.as_u64() {
        return Some(n.to_string());
    }
    None
}

/// 从选人会话的玩家数据中解析基础信息
fn parse_player_from_session(
    player: &serde_json::Value,
    local_cell_id: i32,
    is_custom_game: bool,
) -> Result<PlayerAnalysisData, Box<dyn std::error::Error + Send + Sync>> {
    let cell_id = player["cellId"].as_i64().unwrap_or(0) as i32;
    let display_name = player["displayName"].as_str().unwrap_or("").to_string();
    let summoner_id = lcu_id_as_string(&player["summonerId"]);
    let summoner_id_num = parse_lcu_id_i64(&player["summonerId"]);
    let game_name = player["gameName"].as_str().unwrap_or("");
    let puuid = player["puuid"].as_str().unwrap_or("");
    let name_visibility = player["nameVisibilityType"].as_str().unwrap_or("");
    let player_type = player["playerType"].as_str().unwrap_or("");
    let is_humanoid = player["isHumanoid"].as_bool().unwrap_or(false);

    let identity = classify_champ_select_identity(ChampSelectIdentityFields {
        player_type,
        summoner_id_num,
        game_name,
        puuid,
        name_visibility,
        is_custom_game,
        is_humanoid,
    });
    let is_bot = identity.is_bot;
    let resolved_display_name = if identity.is_obfuscated {
        String::new()
    } else {
        display_name
    };
    let is_obfuscated = identity.is_obfuscated;

    log::debug!(
        target: "analysis::team",
        "Parsed player: cellId={}, displayName='{}', gameName='{}', tagLine='{}', summonerId={}, isBot={}, obfuscated={}, puuid='{}'",
        cell_id,
        resolved_display_name,
        game_name,
        player["tagLine"].as_str().unwrap_or(""),
        summoner_id_num,
        is_bot,
        is_obfuscated,
        puuid
    );

    Ok(PlayerAnalysisData {
        cell_id,
        display_name: resolved_display_name,
        summoner_id,
        puuid: player["puuid"]
            .as_str()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty()),
        is_local: cell_id == local_cell_id,
        is_bot,
        analysis_status: if is_bot {
            PlayerAnalysisStatus::Bot
        } else if is_obfuscated {
            PlayerAnalysisStatus::AwaitingIdentity
        } else {
            PlayerAnalysisStatus::Loading
        },

        champion_id: player["championId"].as_i64().map(|id| id as i32),
        champion_name: player["championName"].as_str().map(|s| s.to_string()),
        champion_pick_intent: player["championPickIntent"].as_i64().map(|id| id as i32),
        position: canonical_assigned_position(&player["assignedPosition"]),

        solo_rank: None,
        flex_rank: None,
        profile_icon_id: player["profileIconId"].as_i64().map(|id| id as i32),
        tag_line: player["tagLine"].as_str().map(|s| s.to_string()),
        spell1_id: player["spell1Id"].as_i64(),
        spell2_id: player["spell2Id"].as_i64(),

        analysis: None,
    })
}

/// 从 LCU API 获取并填充玩家的召唤师信息（displayName, tier等）
async fn enrich_player_data(
    player_data: &mut PlayerAnalysisData,
    raw_player: &serde_json::Value,
    http_client: &reqwest::Client,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 优先从 raw_player 中提取 displayName
    // ChampSelect session 中通常有 gameName + tagLine 组合
    let game_name = raw_player["gameName"].as_str();
    let tag_line = raw_player["tagLine"].as_str();

    // 如果有 gameName，构建完整的显示名称
    if let Some(name) = game_name {
        if !name.is_empty() {
            if let Some(tag) = tag_line {
                if !tag.is_empty() {
                    player_data.display_name = format!("{}#{}", name, tag);
                } else {
                    player_data.display_name = name.to_string();
                }
            } else {
                player_data.display_name = name.to_string();
            }
            player_data.tag_line = tag_line.map(|s| s.to_string());
            log::debug!(
                target: "analysis::team",
                "Extracted display name from session: {}",
                player_data.display_name
            );
        }
    }

    // 降级方案：通过 summonerId 查询（兼容 string 或 number）
    let summoner_id_u64 = if let Some(s) = raw_player["summonerId"].as_str() {
        s.parse().unwrap_or(0)
    } else {
        raw_player["summonerId"].as_u64().unwrap_or_default()
    };
    if summoner_id_u64 > 0 {
        match get_summoner_by_id(http_client, summoner_id_u64).await {
            Ok(summoner_info) => {
                apply_rank_summaries(player_data, &summoner_info);
                if !summoner_info.display_name.trim().is_empty() {
                    player_data.display_name = summoner_info.display_name;
                }
                player_data.profile_icon_id = Some(summoner_info.profile_icon_id as i32);
                player_data.tag_line = summoner_info.tag_line;
                log::debug!(
                    target: "analysis::team",
                    "Fetched summoner info by ID {}: displayName='{}', soloRank={:?}, flexRank={:?}",
                    summoner_id_u64,
                    player_data.display_name,
                    player_data.solo_rank,
                    player_data.flex_rank
                );
            }
            Err(e) => {
                log::warn!(
                    target: "analysis::team",
                    "Failed to fetch summoner info for ID {}: {}",
                    summoner_id_u64,
                    e
                );
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[path = "service_identity_tests.rs"]
mod identity_tests;
