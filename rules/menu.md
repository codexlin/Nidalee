---
name: menu
description: Application menus (native menus and context menus)
metadata:
  tags: menu, native-menu, context-menu, menubar
---

# Application Menus

Tauri v2 supports native menu bars and context menus, providing a consistent user experience with the operating system.

## Menu Types

### Menu Bar
Menu bar located at the top of the window (macOS) or application menu (Windows/Linux).

### Context Menu
Menu displayed on right-click.

## Basic Menu Creation

### Create Menu Bar

```rust
use tauri::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem};

fn create_menu_bar(app: &tauri::App) -> Result<Menu, Box<dyn std::error::Error>> {
    // File menu
    let new_i = MenuItem::with_id(app, "new", "New", true, Some("CmdOrCtrl+N"))?;
    let open_i = MenuItem::with_id(app, "open", "Open...", true, Some("CmdOrCtrl+O"))?;
    let save_i = MenuItem::with_id(app, "save", "Save", true, Some("CmdOrCtrl+S"))?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, Some("CmdOrCtrl+Q"))?;

    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[&new_i, &open_i, &save_i, &separator, &quit_i]
    )?;

    // Edit menu
    let cut_i = MenuItem::with_id(app, "cut", "Cut", true, Some("CmdOrCtrl+X"))?;
    let copy_i = MenuItem::with_id(app, "copy", "Copy", true, Some("CmdOrCtrl+C"))?;
    let paste_i = MenuItem::with_id(app, "paste", "Paste", true, Some("CmdOrCtrl+V"))?;

    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[&cut_i, &copy_i, &paste_i]
    )?;

    // Create menu bar
    let menu = Menu::with_items(app, &[&file_menu, &edit_menu])?;

    Ok(menu)
}
```

### Apply Menu Bar

```rust
fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let menu = create_menu_bar(app)?;
            app.set_menu(menu)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Menu Event Handling

### Global Menu Events

```rust
fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let menu = create_menu_bar(app)?;

            // Set menu event handling
            let menu_clone = menu.clone();
            app.on_menu_event(move |app, event| {
                match event.id.as_ref() {
                    "new" => {
                        println!("New file");
                        // Create new file
                    }
                    "open" => {
                        println!("Open file");
                        // Open file
                    }
                    "save" => {
                        println!("Save file");
                        // Save file
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    "cut" => {
                        println!("Cut");
                    }
                    "copy" => {
                        println!("Copy");
                    }
                    "paste" => {
                        println!("Paste");
                    }
                    _ => {}
                }
            });

            app.set_menu(menu_clone)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Context Menu

### Create Context Menu

```rust
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

fn create_context_menu(app: &tauri::App) -> Result<Menu, Box<dyn std::error::Error>> {
    let cut_i = MenuItem::with_id(app, "cut", "Cut", true, Some("CmdOrCtrl+X"))?;
    let copy_i = MenuItem::with_id(app, "copy", "Copy", true, Some("CmdOrCtrl+C"))?;
    let paste_i = MenuItem::with_id(app, "paste", "Paste", true, Some("CmdOrCtrl+V"))?;
    let separator = PredefinedMenuItem::separator(app)?;
    let delete_i = MenuItem::with_id(app, "delete", "Delete", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&cut_i, &copy_i, &paste_i, &separator, &delete_i])?;

    Ok(menu)
}
```

### Show Context Menu

```rust
#[tauri::command]
fn show_context_menu(
    app: tauri::AppHandle,
    window: tauri::Window,
    x: f64,
    y: f64
) -> Result<(), String> {
    let menu = create_context_menu(&app).map_err(|e| e.to_string())?;

    menu.popup_at(&window, x, y).map_err(|e| e.to_string())?;

    Ok(())
}
```

### Frontend Context Menu

```typescript
import { invoke } from '@tauri-apps/api/core';

// Show context menu
document.addEventListener('contextmenu', async (e) => {
    e.preventDefault();
    await invoke('show_context_menu', {
        x: e.clientX,
        y: e.clientY
    });
});
```

## Dynamic Menu

### Update Menu Item

```rust
#[tauri::command]
fn update_menu_item(
    app: tauri::AppHandle,
    item_id: String,
    enabled: bool,
    checked: bool
) -> Result<(), String> {
    let menu = app.menu().ok_or("Menu not found")?;

    // Get menu item
    if let Some(item) = menu.get(&item_id) {
        item.set_enabled(enabled).map_err(|e| e.to_string())?;

        // If it's a checkable menu item
        if let Ok(check_item) = item.as_check_menuitem() {
            check_item.set_checked(checked).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
```

### Dynamically Add Menu Items

```rust
#[tauri::command]
fn add_recent_file(app: tauri::AppHandle, file_path: String) -> Result<(), String> {
    let menu = app.menu().ok_or("Menu not found")?;

    // Find file menu
    if let Some(file_menu) = menu.get("file") {
        if let Ok(submenu) = file_menu.as_submenu() {
            // Create new menu item
            let item = MenuItem::with_id(
                &app,
                format!("recent-{}", file_path),
                &file_path,
                true,
                None::<&str>
            ).map_err(|e| e.to_string())?;

            // Add to menu
            submenu.insert(&item, 3).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}
```

## Checkable Menu Items

### Create Checkable Menu

```rust
use tauri::menu::CheckMenuItem;

fn create_view_menu(app: &tauri::App) -> Result<Submenu, Box<dyn std::error::Error>> {
    let toolbar_i = CheckMenuItem::with_id(
        app,
        "toolbar",
        "Show Toolbar",
        true,
        true,  // Checked by default
        None::<&str>
    )?;

    let sidebar_i = CheckMenuItem::with_id(
        app,
        "sidebar",
        "Show Sidebar",
        true,
        false,  // Unchecked by default
        None::<&str>
    )?;

    let statusbar_i = CheckMenuItem::with_id(
        app,
        "statusbar",
        "Show Status Bar",
        true,
        true,  // Checked by default
        None::<&str>
    )?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[&toolbar_i, &sidebar_i, &statusbar_i]
    )?;

    Ok(view_menu)
}
```

### Handle Check Events

```rust
fn setup_view_menu_events(app: &tauri::App) {
    app.on_menu_event(|app, event| {
        match event.id.as_ref() {
            "toolbar" => {
                if let Some(window) = app.get_webview_window("main") {
                    // Toggle toolbar visibility
                    window.emit("toggle-toolbar", ()).unwrap();
                }
            }
            "sidebar" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.emit("toggle-sidebar", ()).unwrap();
                }
            }
            "statusbar" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.emit("toggle-statusbar", ()).unwrap();
                }
            }
            _ => {}
        }
    });
}
```

## Icon Menu Items

### Menu Items with Icons

```rust
use tauri::menu::IconMenuItem;
use tauri::image::Image;

fn create_icon_menu(app: &tauri::App) -> Result<Menu, Box<dyn std::error::Error>> {
    let icon = Image::from_path("icons/menu-icon.png")?;

    let icon_item = IconMenuItem::with_id(
        app,
        "icon-action",
        "Action with Icon",
        true,
        Some(icon),
        None::<&str>
    )?;

    let menu = Menu::with_items(app, &[&icon_item])?;

    Ok(menu)
}
```

## Predefined Menu Items

### Use System Standard Menu Items

```rust
use tauri::menu::PredefinedMenuItem;

fn create_standard_menu(app: &tauri::App) -> Result<Menu, Box<dyn std::error::Error>> {
    let separator = PredefinedMenuItem::separator(app)?;
    let copy = PredefinedMenuItem::copy(app, None)?;
    let cut = PredefinedMenuItem::cut(app, None)?;
    let paste = PredefinedMenuItem::paste(app, None)?;
    let select_all = PredefinedMenuItem::select_all(app, None)?;
    let undo = PredefinedMenuItem::undo(app, None)?;
    let redo = PredefinedMenuItem::redo(app, None)?;
    let minimize = PredefinedMenuItem::minimize(app, None)?;
    let zoom = PredefinedMenuItem::zoom(app)?;
    let fullscreen = PredefinedMenuItem::fullscreen(app, None)?;
    let close = PredefinedMenuItem::close_window(app, None)?;
    let quit = PredefinedMenuItem::quit(app, None)?;
    let about = PredefinedMenuItem::about(app, None, None)?;
    let hide = PredefinedMenuItem::hide(app, None)?;
    let services = PredefinedMenuItem::services(app)?;

    let menu = Menu::with_items(app, &[
        &copy, &cut, &paste, &separator, &select_all,
        &separator, &undo, &redo,
        &separator, &minimize, &zoom, &fullscreen,
        &separator, &close, &quit,
        &separator, &about, &hide, &services
    ])?;

    Ok(menu)
}
```

## Platform-Specific Menu

### macOS Application Menu

```rust
#[cfg(target_os = "macos")]
fn create_macos_menu(app: &tauri::App) -> Result<Menu, Box<dyn std::error::Error>> {
    // Application menu (shown at leftmost of menu bar)
    let about_i = PredefinedMenuItem::about(app, None, None)?;
    let preferences_i = MenuItem::with_id(app, "preferences", "Preferences...", true, Some("CmdOrCtrl+,"))?;
    let separator = PredefinedMenuItem::separator(app)?;
    let services_i = PredefinedMenuItem::services(app)?;
    let hide_i = PredefinedMenuItem::hide(app, None)?;
    let quit_i = PredefinedMenuItem::quit(app, None)?;

    let app_menu = Submenu::with_items(
        app,
        "MyApp",
        true,
        &[&about_i, &separator, &preferences_i, &separator, &services_i, &separator, &hide_i, &quit_i]
    )?;

    // Window menu
    let minimize_i = PredefinedMenuItem::minimize(app, None)?;
    let zoom_i = PredefinedMenuItem::zoom(app)?;
    let fullscreen_i = PredefinedMenuItem::fullscreen(app, None)?;
    let bring_all_to_front_i = MenuItem::with_id(app, "bring-all-to-front", "Bring All to Front", true, None::<&str>)?;

    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[&minimize_i, &zoom_i, &separator, &fullscreen_i, &separator, &bring_all_to_front_i]
    )?;

    let menu = Menu::with_items(app, &[&app_menu, &window_menu])?;

    Ok(menu)
}
```

## Complete Example

```rust
use tauri::menu::{Menu, MenuItem, Submenu, PredefinedMenuItem, CheckMenuItem};
use tauri::Manager;

fn create_application_menu(app: &tauri::App) -> Result<Menu, Box<dyn std::error::Error>> {
    // File menu
    let new_i = MenuItem::with_id(app, "new", "New", true, Some("CmdOrCtrl+N"))?;
    let open_i = MenuItem::with_id(app, "open", "Open...", true, Some("CmdOrCtrl+O"))?;
    let save_i = MenuItem::with_id(app, "save", "Save", true, Some("CmdOrCtrl+S"))?;
    let save_as_i = MenuItem::with_id(app, "save-as", "Save As...", true, Some("CmdOrCtrl+Shift+S"))?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, Some("CmdOrCtrl+Q"))?;

    let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[&new_i, &open_i, &separator, &save_i, &save_as_i, &separator, &quit_i]
    )?;

    // Edit menu
    let undo_i = PredefinedMenuItem::undo(app, None)?;
    let redo_i = PredefinedMenuItem::redo(app, None)?;
    let cut_i = PredefinedMenuItem::cut(app, None)?;
    let copy_i = PredefinedMenuItem::copy(app, None)?;
    let paste_i = PredefinedMenuItem::paste(app, None)?;
    let select_all_i = PredefinedMenuItem::select_all(app, None)?;

    let edit_menu = Submenu::with_items(
        app,
        "Edit",
        true,
        &[&undo_i, &redo_i, &separator, &cut_i, &copy_i, &paste_i, &separator, &select_all_i]
    )?;

    // View menu
    let toolbar_i = CheckMenuItem::with_id(app, "toolbar", "Show Toolbar", true, true, None::<&str>)?;
    let sidebar_i = CheckMenuItem::with_id(app, "sidebar", "Show Sidebar", true, false, None::<&str>)?;
    let statusbar_i = CheckMenuItem::with_id(app, "statusbar", "Show Status Bar", true, true, None::<&str>)?;
    let fullscreen_i = PredefinedMenuItem::fullscreen(app, None)?;

    let view_menu = Submenu::with_items(
        app,
        "View",
        true,
        &[&toolbar_i, &sidebar_i, &statusbar_i, &separator, &fullscreen_i]
    )?;

    // Window menu
    let minimize_i = PredefinedMenuItem::minimize(app, None)?;
    let zoom_i = PredefinedMenuItem::zoom(app)?;
    let close_i = PredefinedMenuItem::close_window(app, None)?;

    let window_menu = Submenu::with_items(
        app,
        "Window",
        true,
        &[&minimize_i, &zoom_i, &separator, &close_i]
    )?;

    // Help menu
    let about_i = MenuItem::with_id(app, "about", "About", true, None::<&str>)?;

    let help_menu = Submenu::with_items(app, "Help", true, &[&about_i])?;

    // Create menu bar
    let menu = Menu::with_items(app, &[&file_menu, &edit_menu, &view_menu, &window_menu, &help_menu])?;

    Ok(menu)
}

fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let menu = create_application_menu(app)?;

            app.on_menu_event(|app, event| {
                match event.id.as_ref() {
                    "new" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.emit("menu-new", ()).unwrap();
                        }
                    }
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.emit("menu-open", ()).unwrap();
                        }
                    }
                    "save" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.emit("menu-save", ()).unwrap();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    "toolbar" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.emit("toggle-toolbar", ()).unwrap();
                        }
                    }
                    "sidebar" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.emit("toggle-sidebar", ()).unwrap();
                        }
                    }
                    "statusbar" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.emit("toggle-statusbar", ()).unwrap();
                        }
                    }
                    "about" => {
                        if let Some(window) = app.get_webview_window("main") {
                            window.emit("show-about", ()).unwrap();
                        }
                    }
                    _ => {}
                }
            });

            app.set_menu(menu)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Permission Configuration

Configure menu permissions in `capabilities/default.json`:

```json
{
  "permissions": [
    "menu:default"
  ]
}
```

## Best Practices

1. **Shortcuts**: Add shortcuts for commonly used functions
2. **Menu Organization**: Organize menus following platform conventions
3. **State Sync**: Keep menu item state synchronized with UI state
4. **Dynamic Updates**: Dynamically enable/disable menu items based on application state
5. **Platform Differences**: Consider menu habits of different platforms
6. **Accessibility**: Ensure menu items have clear labels and shortcuts