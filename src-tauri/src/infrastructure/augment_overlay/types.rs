use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 构建中心「推荐增强」条目
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OverlayGuideAugment.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OverlayGuideAugment {
    pub id: i32,
    pub name: String,
    pub icon_url: String,
    pub rarity_name: String,
    pub rarity_display_name: String,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub games: Option<i32>,
    pub tier: Option<i32>,
}

/// 构建中心「推荐三连」
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OverlayGuideTrio.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OverlayGuideTrio {
    pub augments: Vec<OverlayGuideAugment>,
    pub win_rate: f64,
    pub pick_rate: f64,
    pub games: Option<i32>,
}

/// 海克斯推荐侧栏快照。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AugmentDetectedPayload.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct AugmentDetectedPayload {
    pub success: bool,
    pub game_phase: String,
    pub champion_id: Option<i32>,
    pub champion_name: Option<String>,
    pub recommended_augments: Vec<OverlayGuideAugment>,
    pub recommended_trios: Vec<OverlayGuideTrio>,
    #[ts(type = "number")]
    pub timestamp: i64,
    pub winrate_pending: bool,
}

/// 清空海克斯浮窗
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/AugmentClearedPayload.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct AugmentClearedPayload {
    pub success: bool,
    pub game_phase: String,
    pub reason: String,
    #[ts(type = "number")]
    pub timestamp: i64,
}
