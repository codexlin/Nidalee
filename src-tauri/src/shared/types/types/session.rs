use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/LobbyInfo.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct LobbyInfo {
    #[serde(default)]
    pub can_start_activity: bool,
    #[serde(default)]
    #[ts(type = "any")]
    pub game_config: serde_json::Value, // 使用 Value 因为结构复杂
    #[serde(default)]
    #[ts(type = "any[]")]
    pub invitations: Vec<serde_json::Value>,
    #[serde(default)]
    pub local_member: Option<LobbyMember>,
    #[serde(default)]
    pub members: Vec<LobbyMember>,
    #[serde(default)]
    #[ts(type = "any")]
    pub muc_jwt_dto: Option<serde_json::Value>,
    #[serde(default)]
    pub multi_user_chat_id: String,
    #[serde(default)]
    pub multi_user_chat_password: String,
    #[serde(default)]
    pub party_id: String,
    #[serde(default)]
    pub party_type: String,
    #[serde(default)]
    #[ts(type = "any[]")]
    pub restrictions: Vec<serde_json::Value>,
    #[serde(default)]
    #[ts(type = "any[]")]
    pub warnings: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/LobbyMember.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct LobbyMember {
    // 基本信息
    #[ts(type = "string")]
    #[serde(deserialize_with = "crate::shared::types::string_or_number")]
    pub summoner_id: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub puuid: String,
    #[serde(default)]
    pub summoner_name: String,
    #[ts(type = "number")]
    #[serde(default)]
    pub summoner_level: i64,
    #[ts(type = "number")]
    #[serde(default)]
    pub summoner_icon_id: i64,

    // 权限
    #[serde(default)]
    pub is_bot: bool,
    #[serde(default)]
    pub is_leader: bool,
    #[serde(default)]
    pub is_spectator: bool,
    #[serde(default)]
    pub ready: bool,

    // 机器人相关
    #[ts(type = "number")]
    #[serde(default)]
    pub bot_champion_id: i32,
    #[serde(default)]
    pub bot_difficulty: String,
    #[serde(default)]
    pub bot_id: String,
    #[serde(default)]
    pub bot_position: String,

    // 位置偏好
    #[serde(default)]
    pub first_position_preference: String,
    #[serde(default)]
    pub second_position_preference: String,

    // 其他字段（使用 default 避免解析失败）
    #[serde(default)]
    pub allowed_change_activity: bool,
    #[serde(default)]
    pub allowed_invite_others: bool,
    #[serde(default)]
    pub allowed_kick_others: bool,
    #[serde(default)]
    pub allowed_start_activity: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/SummonerInfo.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct SummonerInfo {
    // 基本信息
    pub display_name: String,
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
    #[ts(type = "number")]
    pub summoner_level: i64,
    #[ts(type = "number")]
    pub profile_icon_id: i64,
    pub puuid: String,
    #[ts(type = "string")]
    #[serde(deserialize_with = "crate::shared::types::string_or_number")]
    pub account_id: String,
    #[ts(type = "string")]
    #[serde(deserialize_with = "crate::shared::types::string_or_number")]
    pub summoner_id: String,

    // 经验信息（来自 /lol-summoner/v1/current-summoner）
    #[serde(default)]
    #[ts(type = "number")]
    pub xp_since_last_level: i64,
    #[serde(default)]
    #[ts(type = "number")]
    pub xp_until_next_level: i64,
    #[serde(default)]
    pub percent_complete_for_next_level: Option<f64>,

    // 游戏状态
    #[serde(default)]
    pub game_status: Option<String>,
    #[serde(default)]
    pub availability: Option<String>,

    // 挑战系统（current-summoner 无此字段，由 fill_challenge_info 补全）
    #[serde(default)]
    pub challenge_points: Option<String>,
    #[serde(default)]
    pub challenge_crystal_level: Option<String>,

    // 排位信息 - 单人排位
    #[serde(default)]
    pub solo_rank_tier: Option<String>,
    #[serde(default)]
    pub solo_rank_division: Option<String>,
    #[serde(default)]
    pub solo_rank_wins: Option<i32>,
    #[serde(default)]
    pub solo_rank_losses: Option<i32>,
    #[serde(default)]
    pub solo_rank_lp: Option<i32>,

    // 排位信息 - 灵活排位
    #[serde(default)]
    pub flex_rank_tier: Option<String>,
    #[serde(default)]
    pub flex_rank_division: Option<String>,
    #[serde(default)]
    pub flex_rank_wins: Option<i32>,
    #[serde(default)]
    pub flex_rank_losses: Option<i32>,
    #[serde(default)]
    pub flex_rank_lp: Option<i32>,

    // 历史最高排位
    #[serde(default)]
    pub highest_rank_this_season: Option<String>,

    // 天赋信息
    #[serde(default)]
    pub current_perk_page: Option<String>,
    #[serde(default)]
    pub primary_style_id: Option<i32>,
    #[serde(default)]
    pub sub_style_id: Option<i32>,
    #[serde(default)]
    pub selected_perk_ids: Option<Vec<i32>>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/RankedStats.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct RankedStats {
    pub queue_map: std::collections::HashMap<String, QueueStats>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/QueueStats.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct QueueStats {
    pub tier: String,
    pub division: String,
    pub league_points: u32,
    pub wins: u32,
    pub losses: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ChampSelectAction.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectAction {
    pub actor_cell_id: Option<i32>,
    pub champion_id: Option<i32>,
    pub completed: bool,
    pub id: i32,
    pub is_ally_action: Option<bool>,
    pub is_in_progress: Option<bool>,
    pub pick_turn: Option<i32>,
    #[serde(rename = "type")]
    pub action_type: String,
    pub is_current_user: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ChampSelectBans.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectBans {
    pub my_team_bans: Vec<Option<i64>>,
    pub their_team_bans: Vec<Option<i64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ChampSelectTimer.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ChampSelectTimer {
    pub phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ChampSelectPlayer.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Rust-owned schema exported for raw LCU events consumed by Vue.
pub struct ChampSelectPlayer {
    pub cell_id: i32,
    pub puuid: Option<String>,
    #[ts(type = "string | null")]
    #[serde(deserialize_with = "crate::shared::types::option_string_or_number")]
    pub summoner_id: Option<String>,
    pub champion_id: Option<f64>,
    pub champion_pick_intent: Option<f64>,
    pub selected_skin_id: Option<f64>,
    pub spell1_id: Option<f64>,
    pub spell2_id: Option<f64>,
    pub assigned_position: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ChampSelectSession.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Rust-owned schema exported for raw LCU events consumed by Vue.
pub struct ChampSelectSession {
    pub local_player_cell_id: i32,
    #[ts(type = "number")]
    #[serde(default)]
    pub queue_id: i64,
    #[serde(default)]
    pub is_custom_game: bool,
    pub my_team: Vec<ChampSelectPlayer>,
    pub their_team: Vec<ChampSelectPlayer>,
    pub bans: ChampSelectBans,
    pub timer: ChampSelectTimer,
    pub actions: Vec<Vec<ChampSelectAction>>,
}

#[derive(Debug, Default, PartialEq, Eq, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/RankInfo.ts",
    rename_all = "camelCase"
)]
pub struct RankInfo {
    pub solo_tier: Option<String>,
    pub solo_division: Option<String>,
    pub solo_lp: Option<i32>,
    pub solo_wins: Option<i32>,
    pub solo_losses: Option<i32>,
    pub flex_tier: Option<String>,
    pub flex_division: Option<String>,
    pub flex_lp: Option<i32>,
    pub flex_wins: Option<i32>,
    pub flex_losses: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/MatchmakingState.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingState {
    pub errors: Vec<MatchmakingError>,
    pub low_priority_data: LowPriorityData,
    pub search_state: String,
    pub estimated_queue_time: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/MatchmakingError.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct MatchmakingError {
    pub error_type: String,
    pub id: i32,
    pub message: String,
    #[ts(type = "number")]
    pub penalized_summoner_id: i64,
    #[ts(type = "number")]
    pub penalty_time_remaining: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/LowPriorityData.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct LowPriorityData {
    pub busted_leaver_access_token: String,
    #[ts(type = "number[]")]
    pub penalized_summoner_ids: Vec<i64>,
    pub penalty_time: f64,
    pub penalty_time_remaining: f64,
    pub reason: String,
}

/// 符文页面数据结构
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/RunePage.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct RunePage {
    pub id: i64,
    pub name: String,
    pub current: bool,
    pub is_editable: bool,
    #[serde(rename = "isDeletable")]
    pub is_deletable: bool,
    #[serde(rename = "isValid")]
    pub is_valid: bool,
    pub primary_style_id: i32,
    pub sub_style_id: i32,
    pub selected_perk_ids: Vec<i32>,
}

/// 创建符文页面的请求结构
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRunePageRequest {
    pub name: String,
    pub primary_style_id: i32,
    pub sub_style_id: i32,
    pub selected_perk_ids: Vec<i32>,
}
