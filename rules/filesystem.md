---
name: filesystem
description: File system operations (read, write, watch)
metadata:
  tags: filesystem, fs, read, write, watch, file
---

# File System Operations

Tauri v2 provides file system operations through the `@tauri-apps/plugin-fs` plugin.

## Install Plugin

```bash
# Add plugin (Rust)
cargo add tauri-plugin-fs --target 'cfg(any(target_os = "macos", target_os = "ios"))'

# Frontend install
pnpm add @tauri-apps/plugin-fs
```

### Register in Rust

```rust
fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Basic File Operations

### Read File

```typescript
import { readTextFile, readFile } from '@tauri-apps/plugin-fs';

// Read text file
const content = await readTextFile('config.txt', {
    baseDir: BaseDirectory.AppData
});

// Read binary file
const bytes = await readFile('image.png', {
    baseDir: BaseDirectory.AppData
});
```

### Write File

```typescript
import { writeTextFile, writeFile } from '@tauri-apps/plugin-fs';

// Write text file
await writeTextFile('config.txt', 'Hello, World!', {
    baseDir: BaseDirectory.AppData
});

// Write binary file
const data = new Uint8Array([0, 1, 2, 3]);
await writeFile('data.bin', data, {
    baseDir: BaseDirectory.AppData
});
```

### Append Content

```typescript
import { appendFile, appendToFile } from '@tauri-apps/plugin-fs';

// Append text
await appendToFile('log.txt', 'New log entry\n', {
    baseDir: BaseDirectory.AppData
});
```

## Directory Operations

### Create Directory

```typescript
import { mkdir } from '@tauri-apps/plugin-fs';

// Create directory
await mkdir('my-folder', {
    baseDir: BaseDirectory.AppData
});

// Create recursively
await mkdir('parent/child/grandchild', {
    baseDir: BaseDirectory.AppData,
    recursive: true
});
```

### Read Directory

```typescript
import { readDir } from '@tauri-apps/plugin-fs';

// Read directory contents
const entries = await readDir('.', {
    baseDir: BaseDirectory.AppData
});

for (const entry of entries) {
    console.log(entry.name, entry.isFile, entry.isDirectory);
}
```

### Delete Directory

```typescript
import { remove } from '@tauri-apps/plugin-fs';

// Delete empty directory
await remove('empty-folder', {
    baseDir: BaseDirectory.AppData
});

// Delete recursively
await remove('folder-with-contents', {
    baseDir: BaseDirectory.AppData,
    recursive: true
});
```

## File Management

### Copy File

```typescript
import { copyFile } from '@tauri-apps/plugin-fs';

await copyFile('source.txt', 'destination.txt', {
    fromPathBaseDir: BaseDirectory.AppData,
    toPathBaseDir: BaseDirectory.AppData
});
```

### Move/Rename File

```typescript
import { rename } from '@tauri-apps/plugin-fs';

await rename('old-name.txt', 'new-name.txt', {
    oldPathBaseDir: BaseDirectory.AppData,
    newPathBaseDir: BaseDirectory.AppData
});
```

### Delete File

```typescript
import { remove } from '@tauri-apps/plugin-fs';

await remove('file-to-delete.txt', {
    baseDir: BaseDirectory.AppData
});
```

### Check File Existence

```typescript
import { exists } from '@tauri-apps/plugin-fs';

const fileExists = await exists('config.txt', {
    baseDir: BaseDirectory.AppData
});
```

## File Watching

```typescript
import { watch, watchImmediate } from '@tauri-apps/plugin-fs';

// Watch for file changes
const stopWatching = await watch(
    'config.txt',
    (event) => {
        console.log('File changed:', event);
    },
    {
        baseDir: BaseDirectory.AppData,
        delayMs: 500  // Debounce delay
    }
);

// Stop watching immediately
await stopWatching();

// Trigger callback immediately
const stopImmediate = await watchImmediate(
    'config.txt',
    (event) => {
        console.log('File changed:', event);
    },
    {
        baseDir: BaseDirectory.AppData
    }
);
```

## Base Directories

```typescript
import { BaseDirectory } from '@tauri-apps/plugin-fs';

// Available base directories
BaseDirectory.Audio          // Audio directory
BaseDirectory.Cache          // Cache directory
BaseDirectory.Config         // Config directory
BaseDirectory.Data           // Data directory
BaseDirectory.Desktop        // Desktop
BaseDirectory.Document       // Document directory
BaseDirectory.Download       // Download directory
BaseDirectory.Home           // User home directory
BaseDirectory.Picture        // Picture directory
BaseDirectory.Public         // Public directory
BaseDirectory.Temp           // Temporary directory
BaseDirectory.Video          // Video directory
BaseDirectory.AppData        // App data directory
BaseDirectory.AppLocalData   // App local data
BaseDirectory.AppConfig      // App config
BaseDirectory.AppCache       // App cache
BaseDirectory.AppLog         // App log
BaseDirectory.Resource       // App resource
```

## Path Operations

### Get Special Paths

```typescript
import { appDataDir, appLocalDataDir, appConfigDir } from '@tauri-apps/api/path';

const appData = await appDataDir();
const localData = await appLocalDataDir();
const configDir = await appConfigDir();
```

### Path Joining

```typescript
import { join, resolve, dirname, basename, extname } from '@tauri-apps/api/path';

// Join paths
const fullPath = await join('folder', 'subfolder', 'file.txt');

// Resolve absolute path
const absolutePath = await resolve('relative/path');

// Get directory name
const dir = await dirname('/path/to/file.txt');  // /path/to

// Get filename
const file = await basename('/path/to/file.txt');  // file.txt

// Get extension
const ext = await extname('/path/to/file.txt');  // .txt
```

## File Metadata

```typescript
import { stat, lstat } from '@tauri-apps/plugin-fs';

// Get file info
const metadata = await stat('file.txt', {
    baseDir: BaseDirectory.AppData
});

console.log(metadata.size);        // File size
console.log(metadata.mtime);       // Modification time
console.log(metadata.atime);       // Access time
console.log(metadata.isFile);      // Is file
console.log(metadata.isDirectory); // Is directory
```

## Permission Configuration

Configure file system permissions in `capabilities/default.json`:

```json
{
  "permissions": [
    {
      "identifier": "fs:default"
    },
    {
      "identifier": "fs:allow-appdata-read"
    },
    {
      "identifier": "fs:allow-appdata-write"
    },
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$APPDATA/**" },
        { "path": "$DOWNLOAD/**" }
      ]
    }
  ]
}
```

## Complete Example

```typescript
import {
    readTextFile,
    writeTextFile,
    mkdir,
    exists,
    BaseDirectory
} from '@tauri-apps/plugin-fs';
import { join } from '@tauri-apps/api/path';

class ConfigManager {
    private configDir = 'configs';

    async saveConfig(name: string, data: object): Promise<void> {
        const configPath = await join(this.configDir, `${name}.json`);

        // Ensure directory exists
        const dirExists = await exists(this.configDir, {
            baseDir: BaseDirectory.AppData
        });

        if (!dirExists) {
            await mkdir(this.configDir, {
                baseDir: BaseDirectory.AppData,
                recursive: true
            });
        }

        // Write config
        await writeTextFile(configPath, JSON.stringify(data, null, 2), {
            baseDir: BaseDirectory.AppData
        });
    }

    async loadConfig(name: string): Promise<object | null> {
        const configPath = await join(this.configDir, `${name}.json`);

        const fileExists = await exists(configPath, {
            baseDir: BaseDirectory.AppData
        });

        if (!fileExists) {
            return null;
        }

        const content = await readTextFile(configPath, {
            baseDir: BaseDirectory.AppData
        });

        return JSON.parse(content);
    }
}
```

## Best Practices

1. **Use Base Directories**: Always use `BaseDirectory` instead of hardcoded paths
2. **Error Handling**: Handle file not found, insufficient permissions, etc.
3. **Path Security**: Validate and sanitize user-provided paths
4. **Minimal Permissions**: Request only necessary file system permissions
5. **Async Operations**: File operations are async, use `await`