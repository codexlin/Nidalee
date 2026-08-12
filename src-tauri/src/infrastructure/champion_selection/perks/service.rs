//! LCU rune-page API.

use crate::shared::types::{CreateRunePageRequest, RunePage};
use crate::shared::utils::{lcu_get, lcu_post, lcu_put, lcu_request_raw};
use reqwest::{Client, Method};
use serde_json::json;

pub async fn get_rune_pages(client: &Client) -> Result<Vec<RunePage>, String> {
    log::info!("🔧 开始获取符文页面列表");
    let result: Result<Vec<RunePage>, String> = lcu_get(client, "/lol-perks/v1/pages").await;
    match &result {
        Ok(pages) => log::info!("🔧 成功获取到 {} 个符文页面", pages.len()),
        Err(e) => log::error!("🔧 获取符文页面失败: {}", e),
    }
    result
}

/// 获取当前活跃的符文页面
pub async fn get_current_rune_page(client: &Client) -> Result<Option<RunePage>, String> {
    let pages: Vec<RunePage> = get_rune_pages(client).await?;
    Ok(pages.into_iter().find(|page| page.current))
}

/// 创建新的符文页面
pub async fn create_rune_page(
    client: &Client,
    name: &str,
    primary_style_id: i32,
    sub_style_id: i32,
    selected_perk_ids: Vec<i32>,
) -> Result<RunePage, String> {
    log::info!("🔧 开始创建符文页面: {}", name);
    log::info!("🔧 主系ID: {}, 副系ID: {}", primary_style_id, sub_style_id);
    log::info!("🔧 符文IDs: {:?}", selected_perk_ids);

    let request = CreateRunePageRequest {
        name: name.to_string(),
        primary_style_id,
        sub_style_id,
        selected_perk_ids,
    };

    let body = serde_json::to_value(request).map_err(|e| format!("序列化创建符文页面请求失败: {}", e))?;

    log::info!("🔧 发送创建符文页面请求到: /lol-perks/v1/pages");
    let result: Result<RunePage, String> = lcu_post(client, "/lol-perks/v1/pages", body).await;
    match &result {
        Ok(page) => log::info!("🔧 成功创建符文页面: {}", page.name),
        Err(e) => log::error!("🔧 创建符文页面失败: {}", e),
    }
    result
}

fn editable_page(pages: &[RunePage]) -> Option<&RunePage> {
    pages
        .iter()
        .find(|page| page.current && page.is_editable)
        .or_else(|| pages.iter().find(|page| page.is_editable && page.is_deletable))
}

async fn update_rune_page(
    client: &Client,
    mut page: RunePage,
    name: String,
    primary_style_id: i32,
    sub_style_id: i32,
    selected_perk_ids: Vec<i32>,
) -> Result<RunePage, String> {
    page.name = name;
    page.primary_style_id = primary_style_id;
    page.sub_style_id = sub_style_id;
    page.selected_perk_ids = selected_perk_ids;

    let path = format!("/lol-perks/v1/pages/{}", page.id);
    let body = serde_json::to_value(page).map_err(|error| format!("序列化符文页失败: {error}"))?;
    lcu_put(client, &path, body).await
}

async fn activate_rune_page(client: &Client, page_id: i64) -> Result<(), String> {
    let response = lcu_request_raw(client, Method::PUT, "/lol-perks/v1/currentpage", Some(json!(page_id))).await?;
    let status = response.status();

    if status.is_success() {
        log::info!("🔧 已将符文页设为当前页: {page_id}");
        return Ok(());
    }

    let detail = response.text().await.unwrap_or_default();
    let detail = detail.trim();
    if detail.is_empty() {
        Err(format!("激活符文页失败: {status}"))
    } else {
        Err(format!("激活符文页失败: {status} - {detail}"))
    }
}

/// 应用符文配置到游戏中。
///
/// 优先原地更新可编辑页；没有可编辑页时才创建新页。绝不先删除用户
/// 符文页，避免创建失败后造成不可恢复的数据丢失。
pub async fn apply_rune_build(
    client: &Client,
    page_label: &str,
    primary_style_id: i32,
    sub_style_id: i32,
    selected_perk_ids: Vec<i32>,
) -> Result<String, String> {
    let pages: Vec<RunePage> = get_rune_pages(client).await?;
    let page_name = format!("Nidalee : {page_label}");
    let applied_page = if let Some(page) = editable_page(&pages) {
        update_rune_page(
            client,
            page.clone(),
            page_name,
            primary_style_id,
            sub_style_id,
            selected_perk_ids,
        )
        .await?
    } else {
        create_rune_page(client, &page_name, primary_style_id, sub_style_id, selected_perk_ids)
            .await
            .map_err(|error| format!("没有可编辑的符文页，且创建新页失败（可能已达到页数上限）: {error}"))?
    };

    activate_rune_page(client, applied_page.id).await?;

    Ok(format!("符文页已应用: {}", applied_page.name))
}

#[cfg(test)]
mod tests {
    use super::editable_page;
    use crate::shared::types::RunePage;

    fn page(id: i64, current: bool, editable: bool, deletable: bool) -> RunePage {
        RunePage {
            id,
            name: format!("page-{id}"),
            current,
            is_editable: editable,
            is_deletable: deletable,
            is_valid: true,
            primary_style_id: 8000,
            sub_style_id: 8100,
            selected_perk_ids: vec![],
        }
    }

    #[test]
    fn prefers_current_editable_page() {
        let pages = vec![page(1, false, true, true), page(2, true, true, true)];
        assert_eq!(editable_page(&pages).map(|page| page.id), Some(2));
    }

    #[test]
    fn never_selects_non_editable_page() {
        let pages = vec![page(1, true, false, true), page(2, false, false, true)];
        assert!(editable_page(&pages).is_none());
    }
}
