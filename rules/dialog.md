---
name: dialog
description: System dialogs (file picker, message boxes)
metadata:
  tags: dialog, file-picker, message-box, confirm, alert
---

# System Dialogs

Tauri v2 provides native system dialog functionality through the `@tauri-apps/plugin-dialog` plugin.

## Install Plugin

```bash
# Add plugin (Rust)
cargo add tauri-plugin-dialog

# Frontend install
pnpm add @tauri-apps/plugin-dialog
```

### Register in Rust

```rust
fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## File Selection Dialog

### Open File

```typescript
import { open } from '@tauri-apps/plugin-dialog';

// Single file selection
const file = await open({
    multiple: false,
    directory: false
});

if (file) {
    console.log('Selected file:', file);
}
```

### Multiple File Selection

```typescript
import { open } from '@tauri-apps/plugin-dialog';

// Multiple file selection
const files = await open({
    multiple: true,
    directory: false
});

if (files && files.length > 0) {
    console.log('Selected files:', files);
}
```

### Select Directory

```typescript
import { open } from '@tauri-apps/plugin-dialog';

// Select directory
const directory = await open({
    directory: true,
    multiple: false
});

if (directory) {
    console.log('Selected directory:', directory);
}
```

### File Selection with Filters

```typescript
import { open } from '@tauri-apps/plugin-dialog';

// File type filtering
const file = await open({
    filters: [
        {
            name: 'Images',
            extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp']
        },
        {
            name: 'Documents',
            extensions: ['pdf', 'doc', 'docx', 'txt']
        },
        {
            name: 'All Files',
            extensions: ['*']
        }
    ]
});
```

### Set Default Path

```typescript
import { open } from '@tauri-apps/plugin-dialog';
import { homeDir } from '@tauri-apps/api/path';

// Set default path
const defaultPath = await homeDir();
const file = await open({
    defaultPath: defaultPath,
    filters: [
        {
            name: 'Text Files',
            extensions: ['txt']
        }
    ]
});
```

## Save File Dialog

### Basic Save

```typescript
import { save } from '@tauri-apps/plugin-dialog';

// Save file
const filePath = await save({
    filters: [
        {
            name: 'Text Files',
            extensions: ['txt']
        },
        {
            name: 'JSON Files',
            extensions: ['json']
        }
    ]
});

if (filePath) {
    console.log('Save to:', filePath);
}
```

### Set Default Filename

```typescript
import { save } from '@tauri-apps/plugin-dialog';

// Set default filename
const filePath = await save({
    defaultPath: 'untitled.txt',
    filters: [
        {
            name: 'Text Files',
            extensions: ['txt']
        }
    ]
});
```

## Message Dialog

### Confirmation Dialog

```typescript
import { confirm } from '@tauri-apps/plugin-dialog';

// Confirmation dialog
const confirmed = await confirm(
    'Are you sure you want to delete this file?',
    { title: 'Confirm Delete', kind: 'warning' }
);

if (confirmed) {
    console.log('User confirmed');
} else {
    console.log('User cancelled');
}
```

### Warning Dialog

```typescript
import { confirm } from '@tauri-apps/plugin-dialog';

// Warning dialog
const proceed = await confirm(
    'This action cannot be undone.',
    { title: 'Warning', kind: 'warning' }
);
```

### Error Dialog

```typescript
import { confirm } from '@tauri-apps/plugin-dialog();

// Error confirmation
const retry = await confirm(
    'Failed to save file. Try again?',
    { title: 'Error', kind: 'error' }
);
```

## Message Box

### Info Message Box

```typescript
import { message } from '@tauri-apps/plugin-dialog';

// Info message box
await message('File saved successfully!', { title: 'Success', kind: 'info' });
```

### Warning Message Box

```typescript
import { message } from '@tauri-apps/plugin-dialog';

// Warning message box
await message('Please check your input.', { title: 'Warning', kind: 'warning' });
```

### Error Message Box

```typescript
import { message } from '@tauri-apps/plugin-dialog';

// Error message box
await message('An error occurred while saving.', { title: 'Error', kind: 'error' });
```

## Ask Dialog

### Ask User Input

```typescript
import { ask } from '@tauri-apps/plugin-dialog';

// Ask dialog
const answer = await ask(
    'Do you want to save changes?',
    { title: 'Unsaved Changes', kind: 'info' }
);

if (answer === true) {
    console.log('User chose Yes');
} else if (answer === false) {
    console.log('User chose No');
} else {
    console.log('User cancelled');
}
```

## Complete Example

```typescript
import {
    open,
    save,
    confirm,
    message,
    ask
} from '@tauri-apps/plugin-dialog';
import { readTextFile, writeTextFile } from '@tauri-apps/plugin-fs';

class FileManager {
    async openFile(): Promise<string | null> {
        const filePath = await open({
            filters: [
                {
                    name: 'Text Files',
                    extensions: ['txt', 'md']
                },
                {
                    name: 'All Files',
                    extensions: ['*']
                }
            ]
        });

        if (!filePath) return null;

        try {
            const content = await readTextFile(filePath);
            return content;
        } catch (error) {
            await message(`Failed to open file: ${error}`, { title: 'Error', kind: 'error' });
            return null;
        }
    }

    async saveFile(content: string, defaultPath?: string): Promise<boolean> {
        const filePath = await save({
            defaultPath: defaultPath || 'untitled.txt',
            filters: [
                {
                    name: 'Text Files',
                    extensions: ['txt']
                },
                {
                    name: 'Markdown Files',
                    extensions: ['md']
                }
            ]
        });

        if (!filePath) return false;

        try {
            await writeTextFile(filePath, content);
            await message('File saved successfully!', { title: 'Success', kind: 'info' });
            return true;
        } catch (error) {
            await message(`Failed to save file: ${error}`, { title: 'Error', kind: 'error' });
            return false;
        }
    }

    async confirmSaveUnsavedChanges(): Promise<'save' | 'discard' | 'cancel'> {
        const answer = await ask(
            'You have unsaved changes. Do you want to save them?',
            { title: 'Unsaved Changes', kind: 'info' }
        );

        if (answer === true) return 'save';
        if (answer === false) return 'discard';
        return 'cancel';
    }

    async confirmDelete(fileName: string): Promise<boolean> {
        return await confirm(
            `Are you sure you want to delete "${fileName}"?`,
            { title: 'Confirm Delete', kind: 'warning' }
        );
    }
}

// Usage example
const fileManager = new FileManager();

// Open file
const content = await fileManager.openFile();

// Save file
await fileManager.saveFile('Hello, World!', 'myfile.txt');

// Confirm delete
const shouldDelete = await fileManager.confirmDelete('important.txt');
```

## Permission Configuration

Configure dialog permissions in `capabilities/default.json`:

```json
{
  "permissions": [
    "dialog:default",
    "dialog:allow-open",
    "dialog:allow-save",
    "dialog:allow-confirm",
    "dialog:allow-message",
    "dialog:allow-ask"
  ]
}
```

## Best Practices

1. **Error Handling**: Handle cases where user cancels the dialog
2. **File Filtering**: Use appropriate file filters to limit selection types
3. **Default Paths**: Provide reasonable default paths and filenames
4. **Clear Messages**: Use clear and concise dialog messages
5. **Icon Selection**: Choose appropriate icons based on message type (info/warning/error)