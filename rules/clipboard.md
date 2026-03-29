---
name: clipboard
description: Clipboard operations
metadata:
  tags: clipboard, copy, paste, read, write
---

# Clipboard Operations

Tauri v2 provides clipboard read/write functionality through the `@tauri-apps/plugin-clipboard-manager` plugin.

## Install Plugin

```bash
# Add plugin (Rust)
cargo add tauri-plugin-clipboard-manager

# Frontend install
pnpm add @tauri-apps/plugin-clipboard-manager
```

### Register in Rust

```rust
fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Text Operations

### Write Text

```typescript
import { writeText } from '@tauri-apps/plugin-clipboard-manager';

// Write plain text
await writeText('Hello, World!');

// Write multi-line text
await writeText(`Line 1
Line 2
Line 3`);
```

### Read Text

```typescript
import { readText } from '@tauri-apps/plugin-clipboard-manager';

// Read clipboard text
const text = await readText();
console.log('Clipboard content:', text);
```

### Check Clipboard Content

```typescript
import { readText } from '@tauri-apps/plugin-clipboard-manager';

async function hasTextContent(): Promise<boolean> {
    try {
        const content = await readText();
        return content !== null && content !== '';
    } catch {
        return false;
    }
}
```

## HTML Content

### Write HTML

```typescript
import { writeHtml, writeText } from '@tauri-apps/plugin-clipboard-manager';

// Write HTML content
const html = '<b>Bold text</b> and <i>italic text</i>';
await writeHtml(html, 'Plain text fallback');
```

### Read HTML

```typescript
import { readHtml } from '@tauri-apps/plugin-clipboard-manager';

// Read HTML content
const html = await readHtml();
console.log('HTML content:', html);
```

## Image Operations

### Write Image

```typescript
import { writeImage, writeImageBase64 } from '@tauri-apps/plugin-clipboard-manager';

// Write image from Uint8Array
const imageData = new Uint8Array([/* image bytes */]);
await writeImage(imageData);

// Write image from Base64
const base64Image = 'data:image/png;base64,iVBORw0KGgo...';
await writeImageBase64(base64Image);
```

### Read Image

```typescript
import { readImage, readImageBase64 } from '@tauri-apps/plugin-clipboard-manager';

// Read image as Uint8Array
const imageData = await readImage();

// Read image as Base64
const base64Image = await readImageBase64();
```

## Clipboard Monitoring

### Listen for Clipboard Changes

```typescript
import { onClipboardUpdate } from '@tauri-apps/plugin-clipboard-manager';

// Listen for clipboard changes
const unlisten = await onClipboardUpdate((event) => {
    console.log('Clipboard updated:', event);
});

// Stop listening
unlisten();
```

## Complete Example

```typescript
import {
    writeText,
    readText,
    writeHtml,
    readHtml,
    writeImageBase64,
    readImageBase64,
    onClipboardUpdate
} from '@tauri-apps/plugin-clipboard-manager';

class ClipboardManager {
    private unlisten?: () => void;

    async copyText(text: string): Promise<void> {
        await writeText(text);
    }

    async pasteText(): Promise<string> {
        return await readText() || '';
    }

    async copyHtml(html: string, fallbackText?: string): Promise<void> {
        await writeHtml(html, fallbackText || '');
    }

    async pasteHtml(): Promise<string> {
        return await readHtml() || '';
    }

    async copyImage(imageBase64: string): Promise<void> {
        await writeImageBase64(imageBase64);
    }

    async pasteImage(): Promise<string | null> {
        return await readImageBase64();
    }

    async startListening(callback: (type: string) => void): Promise<void> {
        this.unlisten = await onClipboardUpdate((event) => {
            callback(event.type);
        });
    }

    stopListening(): void {
        this.unlisten?.();
    }
}

// Usage example
const clipboard = new ClipboardManager();

// Copy text
await clipboard.copyText('Hello, World!');

// Paste text
const text = await clipboard.pasteText();

// Listen for clipboard changes
await clipboard.startListening((type) => {
    console.log(`Clipboard updated with ${type}`);
});
```

## Permission Configuration

Configure clipboard permissions in `capabilities/default.json`:

```json
{
  "permissions": [
    "clipboard-manager:default",
    "clipboard-manager:allow-write-text",
    "clipboard-manager:allow-read-text",
    "clipboard-manager:allow-write-html",
    "clipboard-manager:allow-read-html",
    "clipboard-manager:allow-write-image",
    "clipboard-manager:allow-read-image"
  ]
}
```

## Best Practices

1. **Error Handling**: Handle clipboard access failures gracefully
2. **Permission Check**: Ensure proper permissions before accessing clipboard
3. **Format Compatibility**: Provide multiple format content for different scenarios
4. **Performance**: Avoid frequent clipboard read/write operations
5. **User Experience**: Provide visual feedback for successful copy operations