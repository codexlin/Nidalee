use std::future::Future;

use reqwest::Client;

use crate::domains::analysis::pipeline::{
    build_key_moments, build_opponent_compare, build_single_match_process_insight,
};
use crate::domains::analysis::{extract_match_evidence, EvidenceQuality, MatchEvidence};
use crate::shared::types::GameProcessReview;

/// 单局过程复盘：优先使用前端已有的 Evidence，否则按需拉详情 + 时间线
///
/// 仅排位（420/440）。匹配/娱乐局不应请求本接口。
pub async fn get_game_process_review_logic(
    client: &Client,
    puuid: &str,
    game_id: u64,
    cached_evidence: Option<MatchEvidence>,
) -> Result<GameProcessReview, String> {
    let (evidence, from_cache) = resolve_match_evidence(puuid, game_id, cached_evidence, || {
        fetch_match_evidence(client, puuid, game_id)
    })
    .await?;

    if !crate::domains::analysis::queue_config::QueueType::from_queue_id(evidence.queue_id as i32).is_ranked() {
        return Err("过程复盘仅支持排位对局（单双/灵活）".into());
    }

    let quality = match evidence.quality {
        EvidenceQuality::Full => "full",
        EvidenceQuality::TimelineMissing => "timelineMissing",
        EvidenceQuality::TimelinePartial => "timelinePartial",
    };

    Ok(GameProcessReview {
        insight: build_single_match_process_insight(&evidence),
        key_moments: build_key_moments(&evidence),
        opponent_compare: build_opponent_compare(&evidence),
        quality: quality.into(),
        from_cache,
    })
}

async fn resolve_match_evidence<F, Fut>(
    puuid: &str,
    game_id: u64,
    cached_evidence: Option<MatchEvidence>,
    fetch: F,
) -> Result<(MatchEvidence, bool), String>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<MatchEvidence, String>>,
{
    let matching_cache =
        cached_evidence.filter(|evidence| evidence.game_id == game_id && evidence.target_puuid == puuid);

    if let Some(cached) = matching_cache
        .as_ref()
        .filter(|evidence| evidence.quality == EvidenceQuality::Full)
    {
        return Ok((cached.clone(), true));
    }

    match fetch().await {
        Ok(evidence) => Ok((evidence, false)),
        Err(error) => {
            if let Some(cached) = matching_cache {
                log::warn!("现拉失败，回退分析缓存 gameId={game_id}: {error}");
                Ok((cached, true))
            } else {
                Err(error)
            }
        }
    }
}

async fn fetch_match_evidence(client: &Client, puuid: &str, game_id: u64) -> Result<MatchEvidence, String> {
    let fetcher = super::super::fetcher::lcu_fetcher(client);
    let game = fetcher
        .fetch_game_detail(game_id)
        .await
        .map_err(|e| format!("获取对局详情失败: {e}"))?;
    let timeline = match fetcher.fetch_game_timeline(game_id).await {
        Ok(timeline) => Some(timeline),
        Err(error) => {
            log::warn!("时间线不可用 gameId={game_id}: {error}");
            None
        }
    };

    extract_match_evidence(&game, timeline.as_ref(), puuid).map_err(|e| e.to_string())
}

#[cfg(test)]
#[path = "process_review_tests.rs"]
mod tests;
