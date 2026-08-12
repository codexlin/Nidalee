//! System tray creation and event handling.

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager, Result as TauriResult,
};

pub fn setup_system_tray(app: &mut App) -> TauriResult<()> {
    let show_item = MenuItem::with_id(app, "show", "显示", true, None::<&str>)?;
    let maximize_item = MenuItem::with_id(app, "maximize", "最大化", true, None::<&str>)?;
    let minimize_item = MenuItem::with_id(app, "minimize", "最小化", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show_item, &maximize_item, &minimize_item, &quit_item])?;

    let mut tray = TrayIconBuilder::new()
        .menu(&menu)
        .tooltip("Nidalee - 英雄联盟游戏助手")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(handle_tray_icon_event)
        .on_menu_event(handle_menu_event);

    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    } else {
        log::warn!("[tray] default window icon is unavailable; creating tray without an icon");
    }

    let _tray = tray.build(app)?;
    Ok(())
}

fn handle_tray_icon_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        if let Some(window) = tray.app_handle().get_webview_window("main") {
            if let Ok(is_visible) = window.is_visible() {
                if is_visible {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.unminimize();
                    let _ = window.set_focus();
                }
            }
        }
    }
}

fn handle_menu_event(app_handle: &AppHandle, event: tauri::menu::MenuEvent) {
    let Some(window) = app_handle.get_webview_window("main") else {
        log::warn!("[tray] main window is unavailable");
        return;
    };

    match event.id.as_ref() {
        "show" => {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
        "maximize" => {
            let _ = window.maximize();
            let _ = window.set_focus();
        }
        "minimize" => {
            let _ = window.minimize();
        }
        "quit" => app_handle.exit(0),
        _ => {}
    }
}
