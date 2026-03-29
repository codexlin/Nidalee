---
name: tray
description: System tray
metadata:
  tags: tray, system-tray, icon, menu
---

# System Tray

Tauri v2 provides complete system tray support, allowing applications to run in the background and interact with users through tray icons.

## Install Plugin

```bash
# Add plugin (Rust)
cargo add tauri-plugin-tray

# Frontend install (optional, for frontend control)
pnpm add @tauri-apps/plugin-tray
```

### Register in Rust

```rust
fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_tray::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Basic Tray Configuration

### Create Tray Icon

```rust
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem};

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu items
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;

    // Create menu
    let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

    // Create tray icon
    let tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click { .. } => {
                    // Handle click event
                }
                TrayIconEvent::DoubleClick { .. } => {
                    // Handle double-click event
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
```

### Use in Application

```rust
fn run() {
    tauri::Builder::default()
        .setup(|app| {
            setup_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Tray Menu

### Create Complex Menu

```rust
use tauri::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem};

fn create_tray_menu(app: &tauri::App) -> Result<Menu, Box<dyn std::error::Error>> {
    // Create menu items
    let show_i = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;

    // Create submenu
    let recent_files = Submenu::with_items(
        app,
        "Recent Files",
        true,
        &[
            &MenuItem::with_id(app, "file1", "file1.txt", true, None::<&str>)?,
            &MenuItem::with_id(app, "file2", "file2.txt", true, None::<&str>)?,
        ]
    )?;

    // Separator
    let separator = PredefinedMenuItem::separator(app)?;

    // Settings submenu
    let settings = Submenu::with_items(
        app,
        "Settings",
        true,
        &[
            &MenuItem::with_id(app, "preferences", "Preferences...", true, None::<&str>)?,
            &MenuItem::with_id(app, "account", "Account", true, None::<&str>)?,
        ]
    )?;

    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    // Create main menu
    let menu = Menu::with_items(app, &[
        &show_i,
        &hide_i,
        &separator,
        &recent_files,
        &settings,
        &separator,
        &quit_i,
    ])?;

    Ok(menu)
}
```

### Dynamically Update Menu

```rust
#[tauri::command]
fn update_tray_menu(app: tauri::AppHandle, recent_files: Vec<String>) -> Result<(), String> {
    // Get tray instance
    let tray = app.tray_by_id("main").ok_or("Tray not found")?;

    // Create new menu
    let mut items: Vec<Box<dyn IsMenuItem>> = vec![
        Box::new(MenuItem::with_id(&app, "show", "Show", true, None::<&str>).unwrap()),
        Box::new(PredefinedMenuItem::separator(&app).unwrap()),
    ];

    // Add recent files
    for (i, file) in recent_files.iter().enumerate() {
        let item = MenuItem::with_id(
            &app,
            format!("recent-{}", i),
            file,
            true,
            None::<&str>
        ).unwrap();
        items.push(Box::new(item));
    }

    items.push(Box::new(PredefinedMenuItem::separator(&app).unwrap()));
    items.push(Box::new(MenuItem::with_id(&app, "quit", "Quit", true, None::<&str>).unwrap()));

    // Set new menu
    let menu = Menu::with_items(&app, &items.iter().map(|i| i.as_ref()).collect::<Vec<_>>()).unwrap();
    tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;

    Ok(())
}
```

## Tray Icon

### Set Icon

```rust
use tauri::image::Image;

fn setup_tray_with_icon(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Load icon from file
    let icon = Image::from_path("icons/tray-icon.png")?;

    let tray = TrayIconBuilder::new()
        .icon(icon)
        .icon_as_template(true)  // macOS: as template icon
        .build(app)?;

    Ok(())
}
```

### Dynamically Change Icon

```rust
#[tauri::command]
fn set_tray_icon(app: tauri::AppHandle, icon_path: String) -> Result<(), String> {
    let tray = app.tray_by_id("main").ok_or("Tray not found")?;

    let icon = Image::from_path(&icon_path).map_err(|e| e.to_string())?;
    tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn set_tray_icon_by_status(app: tauri::AppHandle, status: String) -> Result<(), String> {
    let tray = app.tray_by_id("main").ok_or("Tray not found")?;

    let icon_path = match status.as_str() {
        "online" => "icons/tray-online.png",
        "offline" => "icons/tray-offline.png",
        "busy" => "icons/tray-busy.png",
        _ => "icons/tray-default.png",
    };

    let icon = Image::from_path(icon_path).map_err(|e| e.to_string())?;
    tray.set_icon(Some(icon)).map_err(|e| e.to_string())?;

    Ok(())
}
```

## Tray Tooltip

### Set Tooltip

```rust
fn setup_tray_with_tooltip(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let tray = TrayIconBuilder::new()
        .tooltip("My Application\nStatus: Online")
        .build(app)?;

    Ok(())
}
```

### Dynamically Update Tooltip

```rust
#[tauri::command]
fn update_tray_tooltip(app: tauri::AppHandle, tooltip: String) -> Result<(), String> {
    let tray = app.tray_by_id("main").ok_or("Tray not found")?;
    tray.set_tooltip(Some(tooltip)).map_err(|e| e.to_string())?;
    Ok(())
}
```

## Tray Event Handling

### Click Events

```rust
fn setup_tray_events(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let tray = TrayIconBuilder::new()
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click { button, .. } => {
                    match button {
                        MouseButton::Left => {
                            // Left click: show/hide window
                            if let Some(window) = tray.app_handle().get_webview_window("main") {
                                if window.is_visible().unwrap() {
                                    window.hide().unwrap();
                                } else {
                                    window.show().unwrap();
                                    window.set_focus().unwrap();
                                }
                            }
                        }
                        MouseButton::Right => {
                            // Right click: show menu
                            tray.set_show_menu_on_left_click(false);
                        }
                        _ => {}
                    }
                }
                TrayIconEvent::DoubleClick { .. } => {
                    // Double click event
                }
                TrayIconEvent::Enter { .. } => {
                    // Mouse enter
                }
                TrayIconEvent::Leave { .. } => {
                    // Mouse leave
                }
                _ => {}
            }
        })
        .build(app)?;

    Ok(())
}
```

## Complete Example

```rust
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::Manager;

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    // Create menu
    let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide_i = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_i, &hide_i, &separator, &quit_i])?;

    // Create tray
    let tray = TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("My Application")
        .on_menu_event(|app, event| {
            match event.id.as_ref() {
                "quit" => {
                    app.exit(0);
                }
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.show().unwrap();
                        window.set_focus().unwrap();
                    }
                }
                "hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        window.hide().unwrap();
                    }
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap() {
                            window.hide().unwrap();
                        } else {
                            window.show().unwrap();
                            window.set_focus().unwrap();
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn run() {
    tauri::Builder::default()
        .setup(|app| {
            setup_tray(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    // On close: minimize to tray instead of exiting
                    window.hide().unwrap();
                    api.prevent_close();
                }
                _ => {}
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Permission Configuration

Configure tray permissions in `capabilities/default.json`:

```json
{
  "permissions": [
    "tray:default"
  ]
}
```

## Best Practices

1. **Icon Design**: Design clear tray icons that support different resolutions
2. **Menu Organization**: Group related functions using separators
3. **State Feedback**: Reflect application state through icon changes
4. **Tooltip**: Provide useful tooltip information
5. **Click Behavior**: Define consistent click behavior (left click show/hide, right click menu)
6. **Exit Confirmation**: Provide clear exit options
7. **Resource Management**: Properly manage tray resources to avoid memory leaks