use crate::domains::analysis::analyzers::core::strategy::AnalysisMode;
use crate::domains::analysis::pipeline::{
    build_key_moments, build_opponent_compare, build_single_match_process_insight,
};
use crate::domains::analysis::{extract_match_evidence, MatchEvidence};
use crate::shared::types::{
    AdvicePerspective, GameDetail, GameProcessReview, ParticipantInfo, ParticipantStats, PlayerMatchStats, TeamInfo,
    TeamStats,
};
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

// 以下内容为原 match_history.rs 全部内容，粘贴至此
// 其余内容保持不变，全部迁移

/// 单次请求指定数量的原生 LCU 战绩。
///
/// 实现已统一到 [`super::fetcher`]，这里只做转发，避免出现第二套列表获取逻辑。
pub async fn fetch_match_list(client: &Client, puuid: &str, count: usize) -> Result<Value, String> {
    super::fetcher::fetch_match_list(client, puuid, count).await
}

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
    player: ApiPlayer,
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
    profile_icon: i64,
}

/// 获取当前玩家历史战绩统计（自动认证、统一请求、日志耗时）
/// 内部兼容入口（数据采集等）；前端请使用 `analyze_matches`。
///
/// 改为调用统一应用服务，不再自行请求战绩列表。
pub(crate) async fn get_match_history(
    client: &Client,
    end_count: usize,
    queue_id: Option<i32>,
    analysis_mode: Option<AnalysisMode>,
) -> Result<PlayerMatchStats, String> {
    let request =
        super::analysis_service::legacy_analysis_request(end_count as u32, queue_id, analysis_mode, None, None);
    let result = super::analysis_service::analyze_current_summoner(client, &request).await?;

    log::info!(
        "[战绩] 展示 {} 场，胜率 {:.1}%，深度证据 {} 场",
        result.overall_stats.total_games,
        result.overall_stats.win_rate,
        result.analyzed_games
    );

    Ok(super::analysis_service::to_player_match_stats(&result))
}

/// 单局过程复盘：优先使用前端已有的 Evidence，否则按需拉详情 + 时间线
///
/// 仅排位（420/440）。匹配/娱乐局不应请求本接口。
pub async fn get_game_process_review_logic(
    client: &Client,
    puuid: &str,
    game_id: u64,
    cached_evidence: Option<MatchEvidence>,
) -> Result<GameProcessReview, String> {
    // 优先现拉（时间线会话缓存通常已命中），才能带上「对位在干嘛」；失败再回退分析缓存
    let (evidence, from_cache) = match fetch_match_evidence(client, puuid, game_id).await {
        Ok(evidence) => (evidence, false),
        Err(err) => {
            if let Some(cached) = cached_evidence.filter(|e| e.game_id == game_id) {
                log::warn!("[过程复盘] 现拉失败，回退分析缓存 gameId={game_id}: {err}");
                (cached, true)
            } else {
                return Err(err);
            }
        }
    };

    if !crate::domains::analysis::queue_config::QueueType::from_queue_id(evidence.queue_id as i32).is_ranked() {
        return Err("过程复盘仅支持排位对局（单双/灵活）".into());
    }

    let quality = match evidence.quality {
        crate::domains::analysis::EvidenceQuality::Full => "full",
        crate::domains::analysis::EvidenceQuality::TimelineMissing => "timelineMissing",
        crate::domains::analysis::EvidenceQuality::TimelinePartial => "timelinePartial",
    };

    Ok(GameProcessReview {
        insight: build_single_match_process_insight(&evidence),
        key_moments: build_key_moments(&evidence),
        opponent_compare: build_opponent_compare(&evidence),
        quality: quality.into(),
        from_cache,
    })
}

async fn fetch_match_evidence(client: &Client, puuid: &str, game_id: u64) -> Result<MatchEvidence, String> {
    let fetcher = super::fetcher::lcu_fetcher(client);
    let game = fetcher
        .fetch_game_detail(game_id)
        .await
        .map_err(|e| format!("获取对局详情失败: {e}"))?;
    let timeline = match fetcher.fetch_game_timeline(game_id).await {
        Ok(timeline) => Some(timeline),
        Err(error) => {
            log::warn!("[过程复盘] 时间线不可用 gameId={game_id}: {error}");
            None
        }
    };

    extract_match_evidence(&game, timeline.as_ref(), puuid).map_err(|e| e.to_string())
}

pub async fn get_game_detail_logic(client: &Client, game_id: u64) -> Result<GameDetail, String> {
    let raw_detail = super::fetcher::lcu_fetcher(client)
        .fetch_game_detail(game_id)
        .await
        .map_err(|error| format!("获取游戏详细信息失败: {error}"))?;
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
        .map(|p| (p.participant_id, p.player))
        .collect();

    let api_participants = api_game_data.participants;

    let mut participants: Vec<ParticipantInfo> = Vec::with_capacity(api_participants.len());
    for p in api_participants {
        // 详情列表不展示分路；对位由过程复盘 / evidence 层单独解析
        let position = None;
        let stats = p.stats; // No need to clone anymore
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

        if team_id == 100 {
            blue_team_stats.kills += kills;
            blue_team_stats.gold_earned += gold;
            blue_team_stats.total_damage_dealt_to_champions += damage;
            blue_team_stats.vision_score += vision;
        } else {
            red_team_stats.kills += kills;
            red_team_stats.gold_earned += gold;
            red_team_stats.total_damage_dealt_to_champions += damage;
            red_team_stats.vision_score += vision;
        }

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

        let player_identity = player_map.get(&p.participant_id);
        let summoner_name = player_identity.map_or_else(String::new, |pi| {
            if let (Some(name), Some(tag)) = (&pi.game_name, &pi.tag_line) {
                if !name.is_empty() && !tag.is_empty() {
                    return format!("{}#{}", name, tag);
                }
            }
            if let Some(s_name) = &pi.summoner_name {
                return s_name.clone();
            }
            String::from("未知玩家")
        });
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

/// 获取指定召唤师最近几场简单战绩
///
/// # 参数
/// - `client`: HTTP 客户端
/// - `puuid`: 玩家 PUUID
/// - `count`: 获取对局数量
/// - `queue_id`: 可选的队列ID，用于过滤（如 420=单排, 440=灵活排）
/// - `perspective`: 建议视角（默认SelfImprovement）
/// - `target_name`: 目标玩家名称（用于建议措辞）
pub async fn get_recent_matches_by_puuid(
    client: &Client,
    puuid: &str,
    count: usize,
    queue_id: Option<i32>,
) -> Result<PlayerMatchStats, String> {
    get_recent_matches_by_puuid_with_perspective(
        client,
        puuid,
        count,
        queue_id,
        AdvicePerspective::SelfImprovement,
        None,
    )
    .await
}

/// 获取指定召唤师最近几场战绩（支持指定建议视角）
///
/// 兼容适配器：团队/选人概览路径，强制 Simple（零时间线），
/// 避免 10 玩家 × N 时间线把客户端拖垮。个人深度分析走 `analyze_matches`。
pub async fn get_recent_matches_by_puuid_with_perspective(
    client: &Client,
    puuid: &str,
    count: usize,
    queue_id: Option<i32>,
    perspective: AdvicePerspective,
    target_name: Option<String>,
) -> Result<PlayerMatchStats, String> {
    let mut request = super::analysis_service::legacy_overview_request(count as u32, queue_id, None);
    request.perspective = Some(perspective);
    request.target_player = target_name;
    let result = super::analysis_service::analyze_matches_for_puuid(client, puuid, &request).await?;
    Ok(super::analysis_service::to_player_match_stats(&result))
}
