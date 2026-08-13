use crate::infrastructure::data_services::summoner::service::get_summoner_by_id;
use crate::infrastructure::data_services::summoner::service::get_summoners_by_names;
use crate::infrastructure::match_management::matches::fetcher::fetch_match_list;
use crate::shared::types::{
    AdvicePerspective, MatchPerformance, PlayerAnalysisBasis, PlayerAnalysisData, PlayerAnalysisStatus,
    PlayerMatchStats, PlayerRankSummary, TeamAnalysisData,
};
use futures_util::stream::{self, StreamExt};
use std::collections::HashMap;
use thiserror::Error;

const PLAYER_ENRICH_CONCURRENCY: usize = 4;
const HISTORY_FETCH_CONCURRENCY: usize = 2;
const REALTIME_HISTORY_FETCH_COUNT: usize = 50;
const REALTIME_HISTORY_FETCH_ATTEMPTS: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct MatchStatsCacheKey {
    puuid: String,
    queue_id: i64,
    is_custom_game: bool,
    perspective: AdvicePerspective,
}

impl MatchStatsCacheKey {
    pub(crate) fn new(
        puuid: impl Into<String>,
        queue_id: i64,
        is_custom_game: bool,
        perspective: AdvicePerspective,
    ) -> Self {
        Self {
            puuid: puuid.into(),
            queue_id,
            is_custom_game,
            perspective,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CachedPlayerAnalysis {
    pub(crate) stats: PlayerMatchStats,
    pub(crate) basis: PlayerAnalysisBasis,
    pub(crate) recent_matches: Vec<MatchPerformance>,
}

#[derive(Debug, Error)]
pub(crate) enum RealtimePlayerAnalysisError {
    #[error("战绩服务暂不可用: {0}")]
    Unavailable(String),
    #[error("最近 50 场中没有可用于实时分析的完整对局")]
    InsufficientData,
}

impl RealtimePlayerAnalysisError {
    pub(crate) fn status(&self) -> PlayerAnalysisStatus {
        match self {
            Self::Unavailable(_) => PlayerAnalysisStatus::Unavailable,
            Self::InsufficientData => PlayerAnalysisStatus::InsufficientData,
        }
    }

    pub(crate) fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable(_))
    }
}

pub(crate) type MatchStatsCache = HashMap<MatchStatsCacheKey, CachedPlayerAnalysis>;

/// Only fields that change the player roster or account identity belong in this key.
/// Champion intent, spells, position, actions and timer are volatile ChampSelect state and
/// must not trigger another round of summoner/rank/history requests.
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

fn canonical_assigned_position(value: &serde_json::Value) -> Option<String> {
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

pub(crate) fn refresh_ranked_rating(player: &mut PlayerAnalysisData, queue_id: i64) {
    player.ranked_rating = player
        .match_stats
        .as_ref()
        .zip(player.analysis_basis.as_ref())
        .and_then(|(stats, basis)| {
            crate::domains::analysis::realtime::build_ranked_player_rating(
                queue_id,
                player.solo_rank.as_ref(),
                player.flex_rank.as_ref(),
                stats,
                basis,
            )
        });
}

/// 从 ChampSelect 会话构建完整的分析数据
///
/// 核心业务逻辑：
/// 1. 解析选人会话，提取玩家基础信息
/// 2. 批量获取召唤师详细信息（enrichment）
/// 3. 批量获取战绩数据（使用缓存优化，避免重复请求）
/// 4. 生成完整的 PlayerAnalysisData
pub async fn build_team_analysis_from_session(
    session: &serde_json::Value,
    http_client: &reqwest::Client,
    match_stats_cache: &mut MatchStatsCache,
) -> Result<TeamAnalysisData, Box<dyn std::error::Error + Send + Sync>> {
    let local_player_cell_id = session["localPlayerCellId"].as_i64().unwrap_or(0) as i32;
    let queue_id = session["queueId"].as_i64().unwrap_or(0);
    let is_custom_game = session["isCustomGame"].as_bool().unwrap_or(false);

    log::info!(
        target: "analysis_data::service",
        "Building team analysis data: localPlayerCellId={}, queueId={}, isCustom={}",
        local_player_cell_id,
        queue_id,
        is_custom_game
    );

    if is_custom_game {
        log::debug!(
            target: "analysis_data::service",
            "Custom game detected, some players may be bots"
        );
    }

    let (my_team_players, enemy_team_players) = tokio::join!(
        parse_and_enrich_team(
            session["myTeam"].as_array(),
            "myTeam",
            local_player_cell_id,
            is_custom_game,
            http_client,
        ),
        parse_and_enrich_team(
            session["theirTeam"].as_array(),
            "theirTeam",
            local_player_cell_id,
            is_custom_game,
            http_client,
        ),
    );
    let mut my_team_players = my_team_players;
    let mut enemy_team_players = enemy_team_players;

    // ⭐ 分别为我方和敌方获取战绩，使用不同的建议视角

    // 1. 获取我方队友战绩（使用 Collaboration 视角）
    let my_team_real_players: Vec<_> = my_team_players
        .iter_mut()
        .filter(|p| !p.is_bot && !p.display_name.is_empty() && p.display_name != "未知召唤师")
        .collect();

    if !my_team_real_players.is_empty() {
        log::info!(
            target: "analysis_data::service",
            "Fetching match stats for {} ally players with Collaboration perspective",
            my_team_real_players.len()
        );
        match fetch_players_match_stats_with_perspective(
            my_team_real_players,
            http_client,
            queue_id,
            is_custom_game,
            match_stats_cache,
            AdvicePerspective::Collaboration, // ⭐ 队友视角
            local_player_cell_id,
        )
        .await
        {
            Ok(count) => log::info!("Ally team stats: {} fetched", count),
            Err(e) => log::warn!("Failed to fetch ally stats: {}", e),
        }
    }

    // 2. 获取敌方战绩（使用 Targeting 视角）
    let enemy_real_players: Vec<_> = enemy_team_players
        .iter_mut()
        .filter(|p| !p.is_bot && !p.display_name.is_empty() && p.display_name != "未知召唤师")
        .collect();

    if !enemy_real_players.is_empty() {
        log::info!(
            target: "analysis_data::service",
            "Fetching match stats for {} enemy players with Targeting perspective",
            enemy_real_players.len()
        );
        match fetch_players_match_stats_with_perspective(
            enemy_real_players,
            http_client,
            queue_id,
            is_custom_game,
            match_stats_cache,
            AdvicePerspective::Targeting, // ⭐ 针对敌人视角
            local_player_cell_id,
        )
        .await
        {
            Ok(count) => log::info!("Enemy team stats: {} fetched", count),
            Err(e) => log::warn!("Failed to fetch enemy stats: {}", e),
        }
    }

    // 提取 actions、bans、timer
    let actions = session
        .get("actions")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let bans = session.get("bans").and_then(|v| serde_json::from_value(v.clone()).ok());

    let timer = session
        .get("timer")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    Ok(TeamAnalysisData {
        my_team: my_team_players,
        enemy_team: enemy_team_players,
        local_player_cell_id,
        game_phase: "ChampSelect".to_string(),
        queue_id,
        is_custom_game,
        actions,
        bans,
        timer,
    })
}

async fn parse_and_enrich_team(
    team: Option<&Vec<serde_json::Value>>,
    team_name: &'static str,
    local_player_cell_id: i32,
    is_custom_game: bool,
    http_client: &reqwest::Client,
) -> Vec<PlayerAnalysisData> {
    let Some(team) = team else {
        log::warn!(target: "analysis_data::service", "Session missing '{}' array", team_name);
        return Vec::new();
    };

    let mut players = stream::iter(team.iter().cloned().enumerate())
        .map(|(index, raw_player)| async move {
            let mut player = match parse_player_from_session(&raw_player, local_player_cell_id, is_custom_game) {
                Ok(player) => player,
                Err(error) => {
                    log::warn!(
                        target: "analysis_data::service",
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
                    target: "analysis_data::service",
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
        target: "analysis_data::service",
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

        match_stats: None, // 后续填充
        recent_matches: Vec::new(),
        analysis_basis: None,
        ranked_rating: None,
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
                target: "analysis_data::service",
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
                    target: "analysis_data::service",
                    "Fetched summoner info by ID {}: displayName='{}', soloRank={:?}, flexRank={:?}",
                    summoner_id_u64,
                    player_data.display_name,
                    player_data.solo_rank,
                    player_data.flex_rank
                );
            }
            Err(e) => {
                log::warn!(
                    target: "analysis_data::service",
                    "Failed to fetch summoner info for ID {}: {}",
                    summoner_id_u64,
                    e
                );
            }
        }
    }

    Ok(())
}

/// 批量获取所有真实玩家的战绩数据（支持指定建议视角）⭐
///
/// 参数：
/// - perspective: 建议视角（Collaboration for队友，Targeting for敌人）
/// - local_cell_id: 本地玩家cellId，用于判断是否是自己
async fn fetch_players_match_stats_with_perspective(
    players: Vec<&mut PlayerAnalysisData>,
    http_client: &reqwest::Client,
    queue_id: i64,
    is_custom_game: bool,
    match_stats_cache: &mut MatchStatsCache,
    perspective: AdvicePerspective,
    local_cell_id: i32,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    println!("🎯 批量获取玩家战绩（视角：{:?}）", perspective);

    fetch_all_players_match_stats_internal(
        players,
        http_client,
        queue_id,
        is_custom_game,
        match_stats_cache,
        perspective,
        local_cell_id,
    )
    .await
}

/// 批量获取所有真实玩家的战绩数据（内部实现）
async fn fetch_all_players_match_stats_internal(
    mut players: Vec<&mut PlayerAnalysisData>,
    http_client: &reqwest::Client,
    queue_id: i64,
    is_custom_game: bool,
    match_stats_cache: &mut MatchStatsCache,
    perspective: AdvicePerspective,
    local_cell_id: i32,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    // 第一步：先从缓存恢复已有的战绩数据
    let mut cached_count = 0;
    let mut need_fetch_indices = Vec::new();

    for (idx, player) in players.iter_mut().enumerate() {
        if player.display_name.is_empty() || player.display_name == "未知召唤师" {
            continue;
        }
        let final_perspective = if player.cell_id == local_cell_id {
            AdvicePerspective::SelfImprovement
        } else {
            perspective
        };
        let cached_stats = player.puuid.as_deref().and_then(|puuid| {
            match_stats_cache.get(&MatchStatsCacheKey::new(
                puuid,
                queue_id,
                is_custom_game,
                final_perspective,
            ))
        });
        if let Some(cached_analysis) = cached_stats {
            log::debug!(
                target: "analysis_data::service",
                "Using cached stats for player '{}'",
                player.display_name
            );
            player.match_stats = Some(cached_analysis.stats.clone());
            player.recent_matches = cached_analysis.recent_matches.clone();
            player.analysis_basis = Some(cached_analysis.basis.clone());
            player.analysis_status = PlayerAnalysisStatus::Ready;
            refresh_ranked_rating(player, queue_id);

            if player.is_bot {
                player.is_bot = false;
            }

            cached_count += 1;
        } else {
            log::debug!(
                target: "analysis_data::service",
                "Player '{}' needs match stats fetch",
                player.display_name
            );
            need_fetch_indices.push(idx);
        }
    }

    log::info!(
        target: "analysis_data::service",
        "Match stats cache: {}/{} hit, {} need fetch",
        cached_count,
        players.len(),
        need_fetch_indices.len()
    );

    // 第二步：只为没有缓存的玩家获取战绩
    if need_fetch_indices.is_empty() {
        log::debug!(
            target: "analysis_data::service",
            "All match stats from cache, skipping network request"
        );
        return Ok(cached_count);
    }

    let player_names: Vec<String> = need_fetch_indices
        .iter()
        .filter(|&&idx| players[idx].puuid.as_deref().map_or(true, str::is_empty))
        .map(|&idx| players[idx].display_name.clone())
        .collect();

    log::debug!(
        target: "analysis_data::service",
        "Batch querying summoners: {:?}",
        player_names
    );

    let summoners = if player_names.is_empty() {
        Vec::new()
    } else {
        match get_summoners_by_names(http_client, player_names.clone()).await {
            Ok(summoners) => summoners,
            Err(error) => {
                log::warn!(
                    target: "analysis_data::service",
                    "Batch summoner query failed; PUUID-known players can still continue: {}",
                    error
                );
                Vec::new()
            }
        }
    };

    log::debug!(
        target: "analysis_data::service",
        "Fetched {} summoner infos",
        summoners.len()
    );

    let requests = need_fetch_indices.into_iter().filter_map(|idx| {
        let player = &players[idx];
        let puuid = player.puuid.clone().filter(|puuid| !puuid.is_empty()).or_else(|| {
            summoners
                .iter()
                .find(|summoner| {
                    let full_name = match (&summoner.game_name, &summoner.tag_line) {
                        (Some(game_name), Some(tag_line)) => format!("{game_name}#{tag_line}"),
                        _ => summoner.display_name.clone(),
                    };
                    full_name.eq_ignore_ascii_case(&player.display_name)
                })
                .map(|summoner| summoner.puuid.clone())
        });
        let Some(puuid) = puuid else {
            log::warn!(
                target: "analysis_data::service",
                "Summoner info not found for player '{}', skipping",
                player.display_name
            );
            return None;
        };
        let final_perspective = if player.cell_id == local_cell_id {
            AdvicePerspective::SelfImprovement
        } else {
            perspective
        };
        Some((idx, puuid, player.display_name.clone(), final_perspective))
    });

    let results = stream::iter(requests)
        .map(|(idx, puuid, display_name, final_perspective)| async move {
            let result = fetch_realtime_player_analysis_with_retry(http_client, &puuid, queue_id, is_custom_game).await;
            (idx, puuid, display_name, final_perspective, result)
        })
        .buffer_unordered(HISTORY_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    for (idx, puuid, display_name, final_perspective, result) in results {
        let player = &mut players[idx];
        match result {
            Ok(analysis) => {
                player.puuid = Some(puuid.clone());
                match_stats_cache.insert(
                    MatchStatsCacheKey::new(puuid, queue_id, is_custom_game, final_perspective),
                    analysis.clone(),
                );
                player.match_stats = Some(analysis.stats);
                player.recent_matches = analysis.recent_matches;
                player.analysis_basis = Some(analysis.basis);
                player.analysis_status = PlayerAnalysisStatus::Ready;
                refresh_ranked_rating(player, queue_id);
            }
            Err(error) => {
                log::warn!(
                    target: "analysis_data::service",
                    "Failed to fetch match stats for player '{}': {}, skipping",
                    display_name,
                    error
                );
                player.match_stats = None;
                player.recent_matches.clear();
                player.analysis_basis = None;
                player.ranked_rating = None;
                player.analysis_status = error.status();
            }
        }
    }

    Ok(players.iter().filter(|player| player.match_stats.is_some()).count())
}

async fn fetch_realtime_player_analysis(
    http_client: &reqwest::Client,
    puuid: &str,
    queue_id: i64,
    is_custom_game: bool,
) -> Result<CachedPlayerAnalysis, RealtimePlayerAnalysisError> {
    let response = fetch_match_list(http_client, puuid, REALTIME_HISTORY_FETCH_COUNT)
        .await
        .map_err(RealtimePlayerAnalysisError::Unavailable)?;
    let games = response
        .get("games")
        .and_then(|games| games.get("games"))
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| RealtimePlayerAnalysisError::Unavailable("LCU 战绩响应缺少 games.games".to_string()))?;
    let context = crate::domains::analysis::realtime::RealtimeMatchContext::from_lcu(queue_id, is_custom_game);
    let champion_name = |champion_id| {
        crate::infrastructure::data_services::champion_data::service::get_champion_info(champion_id)
            .map(|champion| champion.name)
    };
    let analysis =
        crate::domains::analysis::realtime::analyze_realtime_history(games, puuid, context, Some(&champion_name))
            .ok_or(RealtimePlayerAnalysisError::InsufficientData)?;

    Ok(CachedPlayerAnalysis {
        stats: analysis.stats,
        basis: analysis.basis,
        recent_matches: analysis.recent_matches,
    })
}

pub(crate) async fn fetch_realtime_player_analysis_with_retry(
    http_client: &reqwest::Client,
    puuid: &str,
    queue_id: i64,
    is_custom_game: bool,
) -> Result<CachedPlayerAnalysis, RealtimePlayerAnalysisError> {
    for attempt in 1..=REALTIME_HISTORY_FETCH_ATTEMPTS {
        let result = fetch_realtime_player_analysis(http_client, puuid, queue_id, is_custom_game).await;
        match result {
            Err(error) if error.is_unavailable() && attempt < REALTIME_HISTORY_FETCH_ATTEMPTS => {
                log::debug!(
                    target: "analysis_data::service",
                    "Realtime history request failed on attempt {attempt}; retrying once: {error}"
                );
                tokio::time::sleep(std::time::Duration::from_millis(750)).await;
            }
            result => return result,
        }
    }

    unreachable!("history retry loop always returns on its final attempt")
}

// 注意：convert_match_statistics_to_player_stats 函数已被移除
// 原因：PlayerMatchStats 已经在 get_recent_matches_by_puuid 中由通用分析器完整计算
// 包含所有增强字段（traits, today_games, dpm, cspm, vspm 等）

#[cfg(test)]
mod identity_tests {
    use super::{
        canonical_assigned_position, champ_select_analysis_key, classify_champ_select_identity, parse_lcu_id_i64,
        patch_team_analysis_from_session, ChampSelectIdentity, ChampSelectIdentityFields,
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
                match_stats: None,
                recent_matches: Vec::new(),
                analysis_basis: None,
                ranked_rating: None,
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
}
