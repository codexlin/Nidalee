//! 透明置顶海克斯推荐侧栏 + 顶部三卡

use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindowBuilder};

pub const SIDE_PANEL_LABEL: &str = "augment-side-panel";
pub const CARD_LABEL: &str = "augment-overlay";

const CARD_WIDTH: f64 = 760.0;
const CARD_HEIGHT: f64 = 112.0;

const CARD_BOOTSTRAP: &str = r#"
(() => {
  window.__NIDALEE_OVERLAY__ = true;
  window.__NIDALEE_OVERLAY_ROUTE__ = '/augment-overlay';
  try {
    document.documentElement.classList.add('overlay-shell', 'dark', 'theme-zinc');
  } catch (_) {}
  const path = (location.pathname || '/').replace(/\/$/, '') || '/';
  if (path !== '/augment-overlay') {
    location.replace('/augment-overlay');
  }
})();
"#;

const SIDE_WIDTH: f64 = 320.0;
const SIDE_HEIGHT: f64 = 640.0;

const SIDE_BOOTSTRAP: &str = r#"
(() => {
  window.__NIDALEE_OVERLAY__ = true;
  window.__NIDALEE_OVERLAY_ROUTE__ = '/augment-side-panel';
  try {
    document.documentElement.classList.add('overlay-shell', 'dark', 'theme-zinc');
  } catch (_) {}
  const path = (location.pathname || '/').replace(/\/$/, '') || '/';
  if (path !== '/augment-side-panel') {
    location.replace('/augment-side-panel');
  }
})();
"#;

pub fn ensure_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(SIDE_PANEL_LABEL).is_some() {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, SIDE_PANEL_LABEL, WebviewUrl::App("/augment-side-panel".into()))
        .title("海克斯推荐")
        .inner_size(SIDE_WIDTH, SIDE_HEIGHT)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .shadow(false)
        .resizable(false)
        .initialization_script(SIDE_BOOTSTRAP)
        .build()
        .map_err(|e| format!("创建海克斯推荐失败: {e}"))?;

    if let Ok(Some(monitor)) = app.primary_monitor() {
        let area = monitor.work_area();
        let scale = monitor.scale_factor();
        let width_px = (SIDE_WIDTH * scale) as i32;
        let height_px = (SIDE_HEIGHT * scale) as i32;
        let x = area.position.x + area.size.width as i32 - width_px - (16.0 * scale) as i32;
        let y = area.position.y + ((area.size.height as i32 - height_px) / 2).max((16.0 * scale) as i32);
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }

    log::info!("[augment-overlay] 已创建推荐侧栏 {SIDE_PANEL_LABEL}");
    Ok(())
}

pub fn ensure_card_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(CARD_LABEL).is_some() {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(app, CARD_LABEL, WebviewUrl::App("/augment-overlay".into()))
        .title("海克斯本轮")
        .inner_size(CARD_WIDTH, CARD_HEIGHT)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .shadow(false)
        .resizable(false)
        .initialization_script(CARD_BOOTSTRAP)
        .build()
        .map_err(|e| format!("创建海克斯三卡失败: {e}"))?;

    if let Ok(Some(monitor)) = app.primary_monitor() {
        let area = monitor.work_area();
        let scale = monitor.scale_factor();
        let width_px = (CARD_WIDTH * scale) as i32;
        let x = area.position.x + ((area.size.width as i32 - width_px) / 2).max(0);
        let y = area.position.y + (12.0 * scale) as i32;
        let _ = window.set_position(PhysicalPosition::new(x, y));
    }

    log::info!("[augment-overlay] 已创建顶部三卡 {CARD_LABEL}");
    Ok(())
}

pub fn show_card_window(app: &AppHandle) {
    if let Err(error) = ensure_card_window(app) {
        log::error!("[augment-overlay] {error}");
        return;
    }
    if let Some(window) = app.get_webview_window(CARD_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_always_on_top(true);
        let _ = window.set_ignore_cursor_events(false);
    }
}

pub fn hide_card_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(CARD_LABEL) {
        let _ = window.hide();
    }
}

pub fn show_side_panel(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(SIDE_PANEL_LABEL) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_always_on_top(true);
        let _ = window.set_ignore_cursor_events(false);
    } else {
        log::warn!("[augment-overlay] show 时窗口不存在: {SIDE_PANEL_LABEL}");
    }
}

/// 对局里只藏内容、不关窗口，避免再显示时被游戏挡住。
pub fn conceal_side_panel(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(SIDE_PANEL_LABEL) {
        let _ = window.set_ignore_cursor_events(true);
        let _ = window.set_always_on_top(true);
    }
}

pub fn hide_side_panel(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(SIDE_PANEL_LABEL) {
        let _ = window.set_ignore_cursor_events(false);
        let _ = window.hide();
    }
    hide_card_window(app);
}
