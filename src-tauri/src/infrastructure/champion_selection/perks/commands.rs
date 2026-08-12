use super::service;
use crate::http_client;
use crate::shared::types::RunePage;
use serde::Deserialize;
use std::collections::HashSet;

#[tauri::command]
pub async fn get_current_rune_page() -> Result<Option<RunePage>, String> {
    service::get_current_rune_page(http_client::get_lcu_client()).await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuneSelectionInput {
    primary_style_id: i32,
    sub_style_id: i32,
    selected_perk_ids: Vec<i32>,
}

impl RuneSelectionInput {
    fn validate(&self) -> Result<(), String> {
        if self.primary_style_id <= 0 || self.sub_style_id <= 0 {
            return Err("主系或副系无效".to_string());
        }
        if self.primary_style_id == self.sub_style_id {
            return Err("主系与副系不能相同".to_string());
        }
        if self.selected_perk_ids.len() != 9 {
            return Err(format!(
                "符文数量错误：需要 9 个，实际 {} 个",
                self.selected_perk_ids.len()
            ));
        }
        if self.selected_perk_ids.iter().any(|perk_id| *perk_id <= 0) {
            return Err("符文 ID 无效".to_string());
        }
        if self.selected_perk_ids.iter().collect::<HashSet<_>>().len() != 9 {
            return Err("符文中存在重复项".to_string());
        }
        Ok(())
    }
}

#[tauri::command]
pub async fn apply_rune_selection(page_label: String, selection: RuneSelectionInput) -> Result<String, String> {
    let page_label = page_label.trim();
    if page_label.is_empty() {
        return Err("符文页名称不能为空".to_string());
    }
    selection.validate()?;

    service::apply_rune_build(
        http_client::get_lcu_client(),
        page_label,
        selection.primary_style_id,
        selection.sub_style_id,
        selection.selected_perk_ids,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::RuneSelectionInput;

    fn valid_selection() -> RuneSelectionInput {
        RuneSelectionInput {
            primary_style_id: 8000,
            sub_style_id: 8200,
            selected_perk_ids: vec![8005, 9111, 9104, 8014, 8233, 8236, 5005, 5008, 5001],
        }
    }

    #[test]
    fn validates_complete_unique_selection() {
        assert!(valid_selection().validate().is_ok());
    }

    #[test]
    fn rejects_duplicate_perks() {
        let mut selection = valid_selection();
        selection.selected_perk_ids[8] = selection.selected_perk_ids[0];
        assert!(selection.validate().is_err());
    }
}
