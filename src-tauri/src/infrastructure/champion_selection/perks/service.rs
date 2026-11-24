//! LCU rune-page API.

use crate::shared::types::{CreateRunePageRequest, RunePage};
use crate::shared::utils::{lcu_get, lcu_post, lcu_request_raw};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri_plugin_store::StoreExt;
use tokio::sync::Mutex;

static APPLY_LOCK: Mutex<()> = Mutex::const_new(());

#[derive(Deserialize)]
struct RuneAccount {
    puuid: String,
}

#[derive(Serialize, Deserialize)]
struct ManagedPage {
    id: i64,
    name: String,
}

pub async fn get_rune_pages(client: &Client) -> Result<Vec<RunePage>, String> {
    log::info!("开始获取符文页面列表");
    let result: Result<Vec<RunePage>, String> = lcu_get(client, "/lol-perks/v1/pages").await;
    match &result {
        Ok(pages) => log::info!("成功获取到 {} 个符文页面", pages.len()),
        Err(e) => log::error!("获取符文页面失败: {}", e),
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
    log::info!("开始创建符文页面: {}", name);
    log::info!("主系ID: {}, 副系ID: {}", primary_style_id, sub_style_id);
    log::info!("符文IDs: {:?}", selected_perk_ids);

    let request = CreateRunePageRequest {
        name: name.to_string(),
        primary_style_id,
        sub_style_id,
        selected_perk_ids,
    };

    let body = serde_json::to_value(request).map_err(|e| format!("序列化创建符文页面请求失败: {}", e))?;

    log::info!("发送创建符文页面请求到: /lol-perks/v1/pages");
    let result: Result<RunePage, String> = lcu_post(client, "/lol-perks/v1/pages", body).await;
    match &result {
        Ok(page) => log::info!("成功创建符文页面: {}", page.name),
        Err(e) => log::error!("创建符文页面失败: {}", e),
    }
    result
}

fn managed_page(pages: Vec<RunePage>, saved: Option<&ManagedPage>) -> Option<RunePage> {
    let saved = saved?;
    pages
        .into_iter()
        .find(|page| page.is_editable && page.id == saved.id && page.name == saved.name)
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
    let body = serde_json::to_value(&page).map_err(|error| format!("序列化符文页失败: {error}"))?;
    lcu_request_raw(client, Method::PUT, &path, Some(body))
        .await?
        .error_for_status()
        .map_err(|error| format!("更新专属符文页失败: {error}"))?;
    Ok(page)
}

fn application_result(page: &RunePage, activation: Result<(), String>) -> Result<String, String> {
    activation.map_err(|error| format!("符文已写入「{}」，但未能确认激活: {error}", page.name))?;
    Ok(format!("符文页已应用: {}", page.name))
}

async fn activate_rune_page(client: &Client, page_id: i64) -> Result<(), String> {
    let response = lcu_request_raw(client, Method::PUT, "/lol-perks/v1/currentpage", Some(json!(page_id))).await?;
    let status = response.status();

    if status.is_success() {
        log::info!("已将符文页设为当前页: {page_id}");
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
/// 仅复用本账号由应用创建且仍保留原名的页面。
pub async fn apply_rune_build(
    app: &tauri::AppHandle,
    client: &Client,
    page_label: &str,
    primary_style_id: i32,
    sub_style_id: i32,
    selected_perk_ids: Vec<i32>,
) -> Result<String, String> {
    let _guard = APPLY_LOCK.lock().await;
    let account: RuneAccount = lcu_get(client, "/lol-summoner/v1/current-summoner").await?;
    if account.puuid.trim().is_empty() {
        return Err("无法识别当前账号，已停止应用符文".into());
    }
    let store = app
        .store("managed-rune-pages.json")
        .map_err(|error| error.to_string())?;
    let saved = store
        .get(&account.puuid)
        .map(serde_json::from_value::<ManagedPage>)
        .transpose()
        .map_err(|error| format!("读取专属符文页记录失败: {error}"))?;
    let pages = get_rune_pages(client).await?;
    let page_name = format!("Nidalee : {page_label}");
    let applied_page = if let Some(page) = managed_page(pages, saved.as_ref()) {
        update_rune_page(
            client,
            page,
            page_name,
            primary_style_id,
            sub_style_id,
            selected_perk_ids,
        )
        .await?
    } else {
        create_rune_page(client, &page_name, primary_style_id, sub_style_id, selected_perk_ids)
            .await
            .map_err(|error| format!("创建专属符文页失败，请检查是否有空余页位: {error}"))?
    };

    store.set(
        &account.puuid,
        json!(ManagedPage {
            id: applied_page.id,
            name: applied_page.name.clone()
        }),
    );
    store
        .save()
        .map_err(|error| format!("符文已写入，但保存专属页记录失败: {error}"))?;
    application_result(&applied_page, activate_rune_page(client, applied_page.id).await)
}

#[cfg(test)]
mod tests {
    use super::{managed_page, ManagedPage};
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
    fn leaves_personal_editable_pages_untouched() {
        let pages = vec![page(1, false, true, true), page(2, true, true, true)];
        assert!(managed_page(pages, None).is_none());
    }

    #[test]
    fn reuses_only_recorded_id_and_unchanged_name() {
        let saved = ManagedPage {
            id: 2,
            name: "page-2".into(),
        };
        let pages = vec![page(1, true, true, true), page(2, false, true, true)];
        assert_eq!(managed_page(pages, Some(&saved)).map(|p| p.id), Some(2));
        let mut renamed = page(2, true, true, true);
        renamed.name = "个人配置".into();
        assert!(managed_page(vec![renamed], Some(&saved)).is_none());
    }

    #[test]
    fn reports_written_page_when_activation_fails() {
        let applied = page(9, false, true, true);
        let result = super::application_result(&applied, Err("503".into()));
        assert_eq!(result.unwrap_err(), "符文已写入「page-9」，但未能确认激活: 503");
    }

    #[test]
    fn never_selects_non_editable_page() {
        let pages = vec![page(1, true, false, true), page(2, false, false, true)];
        let saved = ManagedPage {
            id: 2,
            name: "page-2".into(),
        };
        assert!(managed_page(pages, Some(&saved)).is_none());
    }
}
