---
name: window-management
description: Window creation, management, and control
metadata:
  tags: window, management, create, control, multi-window
---

# Window Management

Tauri v2 provides powerful window management features, supporting multi-window and window state control.

## Create Window

### Define in Configuration

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "Main Window",
        "width": 800,
        "height": 600,
        "center": true
      }
    ]
  }
}
```

### Create Window Dynamically

```rust
use tauri::Manager;

#[tauri::command]
fn create_new_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "new-window",
        tauri::WebviewUrl::App("/new-window".into())
    )
    .title("New Window")
    .inner_size(600.0, 400.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

## Window Control

### Control Current Window (Frontend)

```typescript
import { getCurrentWindow } from '@tauri-apps/api/window';

const window = getCurrentWindow();

// Minimize
await window.minimize();

// Maximize
await window.maximize();

// Close
await window.close();

// Show/Hide
await window.show();
await window.hide();

// Set title
await window.setTitle('New Title');

// Set size
await window.setSize(new LogicalSize(800, 600));

// Set position
await window.setPosition(new LogicalPosition(100, 100));
```

### Control Other Windows

```typescript
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';

// Get specific window
const otherWindow = WebviewWindow.getByLabel('other-window');

if (otherWindow) {
    await otherWindow.show();
    await otherWindow.setTitle('Updated Title');
}
```

## Window State

### Get Window State

```typescript
const window = getCurrentWindow();

// Is maximized
const isMaximized = await window.isMaximized();

// Is minimized
const isMinimized = await window.isMinimized();

// Is visible
const isVisible = await window.isVisible();

// Is focused
const isFocused = await window.isFocused();

// Get position
const position = await window.position();

// Get size
const size = await window.innerSize();
```

### Listen to Window Events

```typescript
const window = getCurrentWindow();

// Listen to window move
await window.onMoved(({ payload: position }) => {
    console.log('Window moved to:', position);
});

// Listen to window resize
await window.onResized(({ payload: size }) => {
    console.log('Window resized to:', size);
});

// Listen to window close
await window.onCloseRequested((event) => {
    console.log('Window is closing');
    // Prevent close
    event.preventDefault();
});

// Listen to window focus/blur
await window.onFocusChanged(({ payload: focused }) => {
    console.log('Window focus:', focused);
});
```

## Multi-Window Management

### Create Multiple Windows

```rust
#[tauri::command]
fn create_multiple_windows(app: tauri::AppHandle) -> Result<(), String> {
    // Main window
    let main_window = tauri::WebviewWindowBuilder::new(
        &app,
        "main",
        tauri::WebviewUrl::App("/".into())
    )
    .title("Main Window")
    .inner_size(800.0, 600.0)
    .build()
    .map_err(|e| e.to_string())?;

    // Tool window
    let tool_window = tauri::WebviewWindowBuilder::new(
        &app,
        "tools",
        tauri::WebviewUrl::App("/tools".into())
    )
    .title("Tools")
    .inner_size(400.0, 300.0)
    .always_on_top(true)
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

### Window Communication

```typescript
// Send event to specific window
const window = WebviewWindow.getByLabel('other-window');
if (window) {
    await window.emit('message', { data: 'Hello from main window' });
}

// Listen to events from other windows
await window.listen('message', (event) => {
    console.log('Received message:', event.payload);
});
```

## Window Options Detailed

### Common Options

```rust
tauri::WebviewWindowBuilder::new(&app, "label", tauri::WebviewUrl::App("/".into()))
    .title("Window Title")
    .inner_size(800.0, 600.0)
    .min_inner_size(400.0, 300.0)
    .max_inner_size(1920.0, 1080.0)
    .position(100.0, 100.0)
    .center()
    .resizable(true)
    .maximized(false)
    .fullscreen(false)
    .decorations(true)
    .always_on_top(false)
    .visible(true)
    .transparent(false)
    .shadow(true)
    .skip_taskbar(false)
    .closable(true)
    .minimizable(true)
    .maximizable(true)
    .build()
```

### Special Window Types

```rust
// Tool window (no taskbar icon)
.tool_window(true)

// Frameless window
.decorations(false)

// Transparent window
.transparent(true)

// Always on top
.always_on_top(true)

// Non-resizable
.resizable(false)

// Non-closable
.closable(false)
```

## Window Lifecycle

### Window Creation Hooks

```rust
fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // On window creation
            app.on_webview_ready(|window| {
                println!("Window created: {}", window.label());
            });

            // On window close
            app.on_window_event(|window, event| {
                match event {
                    tauri::WindowEvent::Destroyed => {
                        println!("Window destroyed: {}", window.label());
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Best Practices

1. **Label Naming**: Use meaningful window labels for easy management
2. **Resource Cleanup**: Close windows that are no longer needed in a timely manner
3. **Error Handling**: Handle cases where window creation fails
4. **State Management**: Save and restore window state
5. **User Experience**: Provide appropriate window feedback and animations