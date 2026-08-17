use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/LiveClientPlayer.ts",
    rename_all = "camelCase"
)]
pub struct LiveClientPlayer {
    #[serde(rename = "summonerName")]
    pub summoner_name: String,
    #[serde(default, rename = "riotId")]
    #[ts(optional)]
    pub riot_id: Option<String>,
    #[serde(default, rename = "riotIdGameName")]
    #[ts(optional)]
    pub riot_id_game_name: Option<String>,
    #[serde(default, rename = "riotIdTagLine")]
    #[ts(optional)]
    pub riot_id_tag_line: Option<String>,
    #[serde(rename = "championName")]
    pub champion_name: String,
    #[serde(rename = "isBot")]
    pub is_bot: bool,
    #[serde(rename = "isDead")]
    pub is_dead: bool,
    #[ts(type = "any[]")]
    pub items: Vec<Value>,
    pub level: i32,
    pub position: String,
    #[serde(rename = "rawChampionName")]
    pub raw_champion_name: String,
    #[serde(rename = "respawnTimer")]
    pub respawn_timer: f64,
    #[ts(type = "any")]
    pub runes: Value,
    #[ts(type = "any")]
    pub scores: Value,
    #[serde(rename = "skinID")]
    pub skin_id: i32,
    #[ts(type = "any")]
    #[serde(rename = "summonerSpells")]
    pub summoner_spells: Value,
    pub team: String,
}

impl LiveClientPlayer {
    pub fn canonical_riot_id(&self) -> Option<Cow<'_, str>> {
        let game_name = self
            .riot_id_game_name
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let tag_line = self
            .riot_id_tag_line
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty());

        if let (Some(game_name), Some(tag_line)) = (game_name, tag_line) {
            return Some(Cow::Owned(format!("{game_name}#{tag_line}")));
        }

        self.riot_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(Cow::Borrowed)
    }

    pub fn normalize_human_identity(&mut self) {
        if self.is_bot {
            return;
        }

        if let Some(riot_id) = self.canonical_riot_id().map(Cow::into_owned) {
            self.summoner_name = riot_id;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LcuAuthInfo {
    pub app_port: u16,
    pub remoting_auth_token: String,
    pub riotclient_app_port: u16,
    pub riotclient_auth_token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/GameDetail.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct GameDetail {
    #[ts(type = "number")]
    pub game_id: u64,
    pub game_duration: i32,
    #[ts(type = "number")]
    pub game_creation: i64,
    pub game_mode: String,
    pub game_type: String,
    pub game_version: String,
    pub map_id: i32,
    pub queue_id: i32,
    pub teams: Vec<TeamInfo>,
    pub participants: Vec<ParticipantInfo>,
    pub blue_team_stats: TeamStats,
    pub red_team_stats: TeamStats,
    pub best_player_champion_id: i32,
    pub max_damage: i32,
    pub max_tank_champion_id: i32,
    pub max_tank: i32,
    pub max_streak_champion_id: i32,
    pub max_streak: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/TeamInfo.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct TeamInfo {
    #[serde(default)]
    pub team_id: Option<i32>,
    #[serde(default)]
    pub win: Option<String>,
    #[serde(default)]
    pub bans: Vec<BanInfo>,
    #[serde(default)]
    pub baron_kills: Option<i32>,
    #[serde(default)]
    pub dragon_kills: Option<i32>,
    #[serde(default)]
    pub tower_kills: Option<i32>,
    #[serde(default)]
    pub inhibitor_kills: Option<i32>,
    /// 峡谷先锋击杀数（LCU 常见字段）
    #[serde(default)]
    pub rift_herald_kills: Option<i32>,
    /// 虚空幼体击杀数（较新版本）
    #[serde(default)]
    pub horde_kills: Option<i32>,
    #[serde(default)]
    pub first_blood: Option<bool>,
    #[serde(default)]
    pub first_tower: Option<bool>,
    /// 部分 LCU 版本误拼为 firstDargon
    /// Some LCU versions misspell this field as `firstDargon`.
    #[serde(default, alias = "firstDargon")]
    pub first_dragon: Option<bool>,
    #[serde(default)]
    pub first_baron: Option<bool>,
    #[serde(default)]
    pub first_inhibitor: Option<bool>,
    #[serde(default)]
    pub first_rift_herald: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(export, export_to = "../../src/types/generated/BanInfo.ts", rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct BanInfo {
    #[serde(default)]
    pub champion_id: Option<i32>,
    #[serde(default)]
    pub pick_turn: Option<i32>,
}
#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ParticipantInfo.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantInfo {
    pub participant_id: i32,
    pub champion_id: i32,
    /// 英雄中文名（含 JADE/经典模式 600xx）；详情列表展示用，避免前端目录未命中
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub champion_name: Option<String>,
    pub summoner_name: String,
    #[ts(type = "number")]
    pub profile_icon_id: i64,
    pub team_id: i32,
    pub rank_tier: Option<String>,
    /// 召唤师技能 1（闪现等）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spell1_id: Option<i32>,
    /// 召唤师技能 2
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spell2_id: Option<i32>,
    /// 主系符文树（精密 8000 / 主宰 8100 …）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perk_primary_style: Option<i32>,
    /// 基石符文（perk0）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perk0: Option<i32>,
    /// 分路：TOP / JUNGLE / MID / ADC / SUPPORT（无法识别时省略）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    pub stats: ParticipantStats,
    pub score: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ParticipantStats.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantStats {
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub champ_level: i32,
    pub gold_earned: i32,
    pub total_damage_dealt_to_champions: i32,
    pub total_damage_taken: i32,
    pub vision_score: i32,
    /// 补刀（小兵）
    #[serde(default)]
    pub total_minions_killed: i32,
    /// 野怪补刀
    #[serde(default)]
    pub neutral_minions_killed: i32,
    /// 最大多杀（2=双杀 … 5=五杀）
    #[serde(default)]
    pub largest_multi_kill: i32,
    /// 最大连杀
    #[serde(default)]
    pub largest_killing_spree: i32,
    /// 推塔数
    #[serde(default)]
    pub turret_kills: i32,
    /// 推水晶数
    #[serde(default)]
    pub inhibitor_kills: i32,
    /// 插眼
    #[serde(default)]
    pub wards_placed: i32,
    /// 排眼
    #[serde(default)]
    pub wards_killed: i32,
    /// 对防御塔伤害
    #[serde(default)]
    pub damage_dealt_to_turrets: i32,
    pub item0: Option<i32>,
    pub item1: Option<i32>,
    pub item2: Option<i32>,
    pub item3: Option<i32>,
    pub item4: Option<i32>,
    pub item5: Option<i32>,
    pub item6: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/TeamStats.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct TeamStats {
    pub kills: i32,
    pub gold_earned: i32,
    pub total_damage_dealt_to_champions: i32,
    pub vision_score: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/PlayerInfo.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Rust-owned schema exported for the frontend event contract.
pub struct PlayerInfo {
    pub summoner_name: String,
    pub champion_id: i32,
    pub team_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/MatchInfo.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Rust-owned schema exported for the frontend event contract.
pub struct MatchInfo {
    pub match_id: String,
    pub players: Vec<PlayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/RecentGame.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] // Rust-owned schema exported for the frontend filter contract.
pub struct RecentGame {
    #[ts(type = "number")]
    pub game_id: u64,
    pub champion_id: i32,
    pub game_mode: String,
    pub win: bool,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub game_duration: i32,
    #[ts(type = "number")]
    pub game_creation: i64,
    #[ts(type = "number")]
    pub queue_id: i64,
    pub performance_rating: String,
}
