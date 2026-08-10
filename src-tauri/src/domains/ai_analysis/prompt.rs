use crate::domains::analysis::evidence::{EvidenceBundle, GamePhase};
use crate::domains::analysis::pipeline::{is_ranked_queue, DeterministicTrait, MatchAnalysisResult};

use super::types::{AiPromptBundle, AiPromptSummary, AiPromptTrait};

/// 将分析结果压缩为脱敏 prompt 输入（绝不包含 PUUID / 召唤师名）
///
/// 硬性门槛：必须具备排位深度证据（`capabilities.local_ai` 且至少一局 420/440）。
pub fn compact_evidence_for_ai(result: &MatchAnalysisResult) -> Option<AiPromptBundle> {
    if !result.capabilities.local_ai {
        return None;
    }

    let evidence: &EvidenceBundle = result.evidence.as_ref()?;
    if evidence.matches.is_empty() {
        return None;
    }

    let ranked_games = evidence.matches.iter().filter(|m| is_ranked_queue(m.queue_id)).count() as u32;
    if ranked_games == 0 {
        return None;
    }

    let summaries: Vec<AiPromptSummary> = evidence
        .summaries
        .iter()
        .filter(|summary| is_ranked_queue(summary.queue_id))
        .take(8)
        .map(|summary| {
            let early = summary.phase_average(GamePhase::Early);
            AiPromptSummary {
                queue_id: summary.queue_id,
                position: summary.position.as_str().to_string(),
                sample_size: summary.sample_size,
                win_rate: summary.win_rate,
                avg_deaths: Some(summary.event_rates.deaths_per_game),
                early_cs_per_min: early.and_then(|p| p.avg_cs_per_min),
                laning_advantage: early.and_then(|p| p.avg_overall_advantage),
            }
        })
        .collect();

    let traits: Vec<AiPromptTrait> = result
        .traits
        .iter()
        .filter(|t| t.supports_conclusion)
        .take(12)
        .map(trait_to_prompt)
        .collect();

    let diagnostics: Vec<String> = result.diagnostics.iter().take(8).map(|d| d.message.clone()).collect();

    Some(AiPromptBundle {
        analyzed_games: evidence.match_count,
        ranked_games,
        main_position: Some(result.main_position.clone()),
        win_rate: Some(result.overall_stats.win_rate),
        summaries,
        traits,
        diagnostics,
    })
}

fn trait_to_prompt(item: &DeterministicTrait) -> AiPromptTrait {
    AiPromptTrait {
        key: item.key.clone(),
        name: item.name.clone(),
        description: item.description.clone(),
        sentiment: format!("{:?}", item.sentiment).to_ascii_lowercase(),
        sample_count: item.sample_count,
        supports_conclusion: item.supports_conclusion,
        evidence_game_ids: item.evidence_game_ids.clone(),
    }
}

/// 系统 + 用户 prompt（要求严格 JSON）
pub fn build_ai_prompt(bundle: &AiPromptBundle) -> Result<(String, String), String> {
    let system = r#"你是英雄联盟排位分析助手。只根据用户提供的聚合证据作答。
硬性规则：
1. 只输出一个 JSON 对象，不要 markdown，不要额外说明。
2. JSON schema:
{
  "summary": string,
  "confidence": number,          // 0~1
  "findings": [{ "title": string, "detail": string, "evidenceGameIds": number[], "confidence": number }],
  "suggestions": [{ "title": string, "actions": string[], "evidenceGameIds": number[], "priority": number }]
}
3. findings/suggestions 必须引用 evidenceGameIds 中已有的对局 ID；没有证据就不要编造。
4. 不要索要或假设玩家真实姓名、PUUID、队友身份。
5. 用简体中文。"#
        .to_string();

    let user = serde_json::to_string_pretty(bundle).map_err(|e| format!("序列化 AI 证据失败: {e}"))?;
    Ok((system, user))
}
