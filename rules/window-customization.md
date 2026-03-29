---
name: window-customization
description: Custom window styles (transparent, frameless, rounded, etc.)
metadata:
  tags: window, customization, transparent, frameless, rounded
---

# Window Customization

Tauri v2 provides rich window customization options, including transparent windows, frameless windows, custom shapes, etc.

## Transparent Windows

### Configure Transparent Window

```json
{
  "app": {
    "windows": [
      {
        "label": "transparent",
        "title": "Transparent Window",
        "transparent": true,
        "decorations": false
      }
    ]
  }
}
```

### Create Transparent Window Dynamically

```rust
#[tauri::command]
fn create_transparent_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "transparent",
        tauri::WebviewUrl::App("/transparent".into())
    )
    .title("Transparent Window")
    .transparent(true)
    .decorations(false)
    .inner_size(400.0, 300.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

### Frontend CSS Settings

```css
/* Make background transparent */
body {
    background: transparent;
    margin: 0;
    padding: 0;
}

/* Add blur background effect */
.app-container {
    background: rgba(255, 255, 255, 0.8);
    backdrop-filter: blur(10px);
    border-radius: 10px;
    padding: 20px;
}
```

## Frameless Windows

### Create Frameless Window

```rust
#[tauri::command]
fn create_frameless_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "frameless",
        tauri::WebviewUrl::App("/frameless".into())
    )
    .title("Frameless Window")
    .decorations(false)  // Remove system frame
    .inner_size(600.0, 400.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

### Custom Title Bar

```typescript
// Implement drag functionality
import { getCurrentWindow } from '@tauri-apps/api/window';

const window = getCurrentWindow();

// Make element draggable
async function setupDraggable(element: HTMLElement) {
    element.addEventListener('mousedown', async (e) => {
        if (e.button === 0) { // Left button
            await window.startDragging();
        }
    });
}

// Make element resizable
async function setupResizable(element: HTMLElement) {
    element.addEventListener('mousedown', async (e) => {
        if (e.button === 0) {
            await window.startResizing('bottom-right');
        }
    });
}
```

## Rounded Windows

### macOS Rounded Window

```rust
#[cfg(target_os = "macos")]
use tauri::WebviewWindowExt;

#[tauri::command]
fn create_rounded_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "rounded",
        tauri::WebviewUrl::App("/rounded".into())
    )
    .title("Rounded Window")
    .transparent(true)
    .decorations(false)
    .inner_size(400.0, 300.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    // macOS specific: set rounded corners
    #[cfg(target_os = "macos")]
    {
        use cocoa::foundation::NSArray;
        use cocoa::appkit::{NSWindow, NSView};

        let ns_window = window.ns_window().unwrap();
        unsafe {
            let _: () = msg_send![ns_window, setCornerRadius:20.0];
        }
    }

    Ok(())
}
```

### CSS Rounded Corners

```css
/* Simple CSS rounded corners */
.app-container {
    background: white;
    border-radius: 20px;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
    overflow: hidden;
}

/* Glassmorphism effect */
.glass-effect {
    background: rgba(255, 255, 255, 0.1);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.2);
    border-radius: 20px;
}
```

## Custom Shadows

### System Shadows

```rust
#[tauri::command]
fn create_shadow_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "shadow",
        tauri::WebviewUrl::App("/shadow".into())
    )
    .title("Shadow Window")
    .transparent(true)
    .decorations(false)
    .shadow(true)  // Enable system shadow
    .inner_size(400.0, 300.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    Ok(())
}
```

### CSS Shadows

```css
.custom-shadow {
    box-shadow:
        0 0 0 1px rgba(0, 0, 0, 0.1),
        0 2px 4px rgba(0, 0, 0, 0.1),
        0 8px 24px rgba(0, 0, 0, 0.15);
}

.neumorphic-shadow {
    box-shadow:
        -5px -5px 10px rgba(255, 255, 255, 0.5),
        5px 5px 10px rgba(0, 0, 0, 0.1);
}
```

## Irregular Windows

### Create Irregular Shape

```rust
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{CreateRectRgn, SetWindowRgn};

#[tauri::command]
fn create_irregular_window(app: tauri::AppHandle) -> Result<(), String> {
    let window = tauri::WebviewWindowBuilder::new(
        &app,
        "irregular",
        tauri::WebviewUrl::App("/irregular".into())
    )
    .title("Irregular Window")
    .transparent(true)
    .decorations(false)
    .inner_size(400.0, 400.0)
    .center()
    .build()
    .map_err(|e| e.to_string())?;

    // Windows specific: set circular window
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::Graphics::Gdi::{CreateEllipticRgn, SetWindowRgn};

        let hwnd = window.hwnd().unwrap();
        unsafe {
            let hrgn = CreateEllipticRgn(0, 0, 400, 400);
            SetWindowRgn(HWND(hwnd.0), hrgn, true);
        }
    }

    Ok(())
}
```

## Custom Title Bar Components

### React Title Bar Component

```tsx
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useState } from 'react';

function TitleBar() {
    const [isMaximized, setIsMaximized] = useState(false);
    const window = getCurrentWindow();

    const handleMinimize = async () => {
        await window.minimize();
    };

    const handleMaximize = async () => {
        if (isMaximized) {
            await window.unmaximize();
        } else {
            await window.maximize();
        }
        setIsMaximized(!isMaximized);
    };

    const handleClose = async () => {
        await window.close();
    };

    return (
        <div
            className="titlebar"
            data-tauri-drag-region
        >
            <div className="titlebar-title">My App</div>
            <div className="titlebar-controls">
                <button onClick={handleMinimize}>−</button>
                <button onClick={handleMaximize}>
                    {isMaximized ? '❐' : '□'}
                </button>
                <button onClick={handleClose}>×</button>
            </div>
        </div>
    );
}
```

### Vue Title Bar Component

```vue
<template>
    <div class="titlebar" data-tauri-drag-region>
        <div class="titlebar-title">My App</div>
        <div class="titlebar-controls">
            <button @click="minimize">−</button>
            <button @click="toggleMaximize">
                {{ isMaximized ? '❐' : '□' }}
            </button>
            <button @click="close">×</button>
        </div>
    </div>
</template>

<script setup>
import { ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

const window = getCurrentWindow();
const isMaximized = ref(false);

const minimize = async () => {
    await window.minimize();
};

const toggleMaximize = async () => {
    if (isMaximized.value) {
        await window.unmaximize();
    } else {
        await window.maximize();
    }
    isMaximized.value = !isMaximized.value;
};

const close = async () => {
    await window.close();
};
</script>
```

### Title Bar Styles

```css
.titlebar {
    height: 32px;
    background: #2c2c2c;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px;
    -webkit-app-region: drag;
}

.titlebar-title {
    color: white;
    font-size: 14px;
    font-weight: 500;
}

.titlebar-controls {
    display: flex;
    gap: 4px;
    -webkit-app-region: no-drag;
}

.titlebar-controls button {
    width: 32px;
    height: 20px;
    border: none;
    background: transparent;
    color: white;
    cursor: pointer;
    border-radius: 2px;
    font-size: 12px;
}

.titlebar-controls button:hover {
    background: rgba(255, 255, 255, 0.1);
}

.titlebar-controls button:active {
    background: rgba(255, 255, 255, 0.2);
}
```

## Platform-Specific Notes

### macOS
- Transparent windows require `transparent: true`
- Rounded corners can be achieved through CSS or native APIs
- System automatically adds shadows

### Windows
- Supports irregular window shapes
- Can use Win32 API for advanced customization
- Transparent windows may require additional configuration on some systems

### Linux
- Transparent window support depends on desktop environment
- Some window managers may not support all customization options
- Test compatibility across different distributions

## Best Practices

1. **Performance Consideration**: Transparent windows may affect performance, use with caution
2. **User Experience**: Maintain consistency in window controls
3. **Accessibility**: Ensure accessibility of custom controls
4. **Cross-Platform**: Test behavior across different platforms
5. **Responsive Design**: Consider different screen sizes and DPI