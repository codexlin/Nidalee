use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 浮窗中的一张海克斯卡
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/OverlayAugment.ts",
    rename_all = "camelCase"
)]
#[serde(rename_all = "camelCase")]
pub struct OverlayAugment {
    pub id: Option<i32>,
    pub name: String,
    pub rarity: String,
    pub rarity_display_name: String,
    pub icon_url: String,
    pub confidence: Option<f32>,
    pub detected_slot: i32,
    pub missing: bool,
    pub win_rate: Option<f64>,
    pub pick_rate: Option<f64>,
    pub games: Option<i32>,
    pub recommended: bool,
}

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

/// OCR 识别到海克斯选择
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
    pub augments: Vec<OverlayAugment>,
    pub recommended_augments: Vec<OverlayGuideAugment>,
    pub recommended_trios: Vec<OverlayGuideTrio>,
    pub analysis_confidence: f32,
    pub partial_update: bool,
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
    pub timestamp: i64,
}

impl OverlayAugment {
    pub fn empty(slot: i32) -> Self {
        Self {
            id: None,
            name: String::new(),
            rarity: "unknown".to_string(),
            rarity_display_name: String::new(),
            icon_url: String::new(),
            confidence: None,
            detected_slot: slot,
            missing: true,
            win_rate: None,
            pick_rate: None,
            games: None,
            recommended: false,
        }
    }
}
