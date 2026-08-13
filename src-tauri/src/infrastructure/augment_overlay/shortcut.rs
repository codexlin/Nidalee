//! 游戏内全局快捷键：显示 / 隐藏海克斯推荐侧栏。

use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use once_cell::sync::OnceCell;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use super::state;
use super::window;

pub const DEFAULT_SHORTCUT: &str = "Insert";
pub const SHORTCUT_CHANGED_EVENT: &str = "augment-overlay-shortcut";

static CURRENT: OnceCell<Mutex<Option<String>>> = OnceCell::new();
static SHORTCUT_HELD: AtomicBool = AtomicBool::new(false);

fn current_slot() -> &'static Mutex<Option<String>> {
    CURRENT.get_or_init(|| Mutex::new(None))
}

pub fn current() -> String {
    current_slot()
        .lock()
        .ok()
        .and_then(|slot| slot.clone())
        .unwrap_or_else(|| DEFAULT_SHORTCUT.to_string())
}

pub fn register_default(app: &AppHandle) -> Result<String, String> {
    register(app, DEFAULT_SHORTCUT)
}

pub fn register(app: &AppHandle, raw: &str) -> Result<String, String> {
    let display = format_display(raw);
    if display.is_empty() {
        return Err("快捷键不能为空".to_string());
    }
    let shortcut = parse_shortcut(&display)?;
    let previous = current_slot().lock().ok().and_then(|slot| slot.clone());
    if previous.as_deref() == Some(display.as_str()) {
        return Ok(display);
    }
    if let Some(previous) = previous {
        if let Ok(previous_shortcut) = parse_shortcut(&previous) {
            let _ = app.global_shortcut().unregister(previous_shortcut);
        }
    }
    SHORTCUT_HELD.store(false, Ordering::Release);

    app.global_shortcut()
        .on_shortcut(shortcut, |app, _shortcut, event| {
            if event.state == ShortcutState::Released {
                SHORTCUT_HELD.store(false, Ordering::Release);
                return;
            }
            if SHORTCUT_HELD.swap(true, Ordering::AcqRel) {
                return;
            }
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                state::toggle_side_panel(app).await;
            });
        })
        .map_err(|error| format!("注册快捷键 {display} 失败: {error}"))?;

    if let Ok(mut slot) = current_slot().lock() {
        *slot = Some(display.clone());
    }
    emit_shortcut_changed(app, &display);
    log::info!("[augment-overlay] 已注册切换快捷键 {display}");
    Ok(display)
}

pub fn parse_shortcut(raw: &str) -> Result<Shortcut, String> {
    let canonical = to_plugin_accelerator(raw);
    Shortcut::from_str(&canonical).map_err(|error| format!("无效快捷键「{raw}」: {error}"))
}

pub fn format_display(raw: &str) -> String {
    raw.split(['+', ' '])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(format_part)
        .collect::<Vec<_>>()
        .join("+")
}

fn format_part(part: &str) -> String {
    match part.to_ascii_lowercase().as_str() {
        "ctrl" | "control" => "Ctrl".to_string(),
        "alt" | "option" => "Alt".to_string(),
        "shift" => "Shift".to_string(),
        "super" | "meta" | "cmd" | "command" | "win" => "Super".to_string(),
        "space" => "Space".to_string(),
        "home" => "Home".to_string(),
        "insert" | "ins" => "Insert".to_string(),
        other if other.len() <= 3 && other.starts_with('f') && other[1..].bytes().all(|b| b.is_ascii_digit()) => {
            other.to_ascii_uppercase()
        }
        other if other.len() == 1 => other.to_ascii_uppercase(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

fn to_plugin_accelerator(raw: &str) -> String {
    format_display(raw)
        .split('+')
        .map(|part| match part {
            "Ctrl" => "ctrl",
            "Alt" => "alt",
            "Shift" => "shift",
            "Super" => "super",
            other => other,
        })
        .map(str::to_ascii_lowercase)
        .collect::<Vec<_>>()
        .join("+")
}

fn emit_shortcut_changed<R: Runtime>(app: &AppHandle<R>, shortcut: &str) {
    let _ = app.emit(SHORTCUT_CHANGED_EVENT, shortcut);
    let _ = app.emit_to(window::SIDE_PANEL_LABEL, SHORTCUT_CHANGED_EVENT, shortcut);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_display_normalizes_keys() {
        assert_eq!(format_display("insert"), "Insert");
        assert_eq!(format_display("Insert"), "Insert");
        assert_eq!(format_display("home"), "Home");
        assert_eq!(format_display("f8"), "F8");
        assert_eq!(format_display("ctrl+shift+h"), "Ctrl+Shift+H");
    }

    #[test]
    fn plugin_accelerator_is_lowercase() {
        assert_eq!(to_plugin_accelerator("Insert"), "insert");
        assert_eq!(to_plugin_accelerator("Home"), "home");
        assert_eq!(to_plugin_accelerator("F8"), "f8");
        assert_eq!(to_plugin_accelerator("Ctrl+Shift+H"), "ctrl+shift+h");
    }

    #[test]
    fn parse_shortcut_accepts_default_and_combo() {
        parse_shortcut("Insert").expect("Insert");
        parse_shortcut("Home").expect("Home");
        parse_shortcut("F8").expect("F8");
        parse_shortcut("Ctrl+Shift+H").expect("Ctrl+Shift+H");
    }
}
