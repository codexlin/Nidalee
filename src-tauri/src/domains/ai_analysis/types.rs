use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// 发给模型的脱敏证据摘要（禁止含 PUUID / 召唤师名 / 完整时间线）
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/AiPromptBundle.ts", rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct AiPromptBundle {
    pub analyzed_games: u32,
    pub ranked_games: u32,
    pub main_position: Option<String>,
    pub win_rate: Option<f64>,
    pub summaries: Vec<AiPromptSummary>,
    pub traits: Vec<AiPromptTrait>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/AiPromptSummary.ts", rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct AiPromptSummary {
    pub queue_id: i64,
    pub position: String,
    pub sample_size: u32,
    pub win_rate: f64,
    pub avg_deaths: Option<f64>,
    pub early_cs_per_min: Option<f64>,
    pub laning_advantage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/AiPromptTrait.ts", rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct AiPromptTrait {
    pub key: String,
    pub name: String,
    pub description: String,
    pub sentiment: String,
    pub sample_count: u32,
    pub supports_conclusion: bool,
    #[ts(type = "number[]")]
    pub evidence_game_ids: Vec<u64>,
}

/// 模型必须返回的结构化解读
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/generated/AiInsight.ts", rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct AiInsight {
    pub summary: String,
    pub confidence: f64,
    pub findings: Vec<AiInsightFinding>,
    pub suggestions: Vec<AiInsightSuggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/generated/AiInsightFinding.ts", rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct AiInsightFinding {
    pub title: String,
    pub detail: String,
    #[ts(type = "number[]")]
    pub evidence_game_ids: Vec<u64>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
#[ts(export, export_to = "../../src/types/generated/AiInsightSuggestion.ts", rename_all = "camelCase")]
#[serde(rename_all = "camelCase")]
pub struct AiInsightSuggestion {
    pub title: String,
    pub actions: Vec<String>,
    #[ts(type = "number[]")]
    pub evidence_game_ids: Vec<u64>,
    pub priority: i32,
}
