---
name: permissions
description: Permission system details (Tauri v2 new feature)
metadata:
  tags: permissions, security, capabilities, access control
---

# Permission System

Tauri v2 introduces a new permission system that controls application permissions through capabilities files.

## Permission System Overview

### Permission Types

1. **Core Permissions**: Built-in Tauri function permissions
2. **Plugin Permissions**: Function permissions provided by plugins
3. **Custom Permissions**: User-defined permissions

### Permission Scope

Permissions can be applied to:
- Global scope
- Specific windows
- Specific plugins
- Specific features

## Basic Permission Configuration

### Default Permission File

Create `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/capabilities-schema.json",
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "fs:allow-appdata-read",
    "fs:allow-appdata-write"
  ]
}
```

### Permission Format

```json
{
  "identifier": "capability-name",
  "description": "Capability description",
  "windows": ["window-label"],  // Applicable windows
  "permissions": [
    "permission-identifier",
    {
      "identifier": "permission-identifier",
      "allow": [...],  // Allowed operations
      "deny": [...]    // Denied operations
    }
  ]
}
```

## Core Permissions

### Default Core Permissions

```json
{
  "permissions": [
    "core:default"
  ]
}
```

### Window Permissions

```json
{
  "permissions": [
    "core:window:allow-create",
    "core:window:allow-close",
    "core:window:allow-show",
    "core:window:allow-hide",
    "core:window:allow-maximize",
    "core:window:allow-unmaximize",
    "core:window:allow-minimize",
    "core:window:allow-unminimize",
    "core:window:allow-set-title",
    "core:window:allow-set-size",
    "core:window:allow-set-position"
  ]
}
```

### WebView Permissions

```json
{
  "permissions": [
    "core:webview:allow-create",
    "core:webview:allow-close",
    "core:webview:allow-print"
  ]
}
```

## File System Permissions

### Basic File System Permissions

```json
{
  "permissions": [
    "fs:default",
    "fs:allow-appdata-read",
    "fs:allow-appdata-write",
    "fs:allow-appconfig-read",
    "fs:allow-appconfig-write"
  ]
}
```

### File System Permissions with Scope

```json
{
  "permissions": [
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$APPDATA/configs/**" },
        { "path": "$DOWNLOAD/**" },
        { "path": "$DOCUMENTS/myapp/**" }
      ]
    }
  ]
}
```

### Specific Operation Permissions

```json
{
  "permissions": [
    "fs:allow-read-file",
    "fs:allow-write-file",
    "fs:allow-read-dir",
    "fs:allow-copy-file",
    "fs:allow-mkdir",
    "fs:allow-remove"
  ]
}
```

## Shell Permissions

### Open Permission

```json
{
  "permissions": [
    "shell:allow-open"
  ]
}
```

### Execute Command Permission

```json
{
  "permissions": [
    {
      "identifier": "shell:allow-execute",
      "allow": [
        {
          "name": "git",
          "cmd": "git",
          "args": ["status", "log", "pull", "push"]
        },
        {
          "name": "node",
          "cmd": "node",
          "args": true  // Allow all arguments
        }
      ]
    }
  ]
}
```

## HTTP Permissions

### Basic HTTP Permissions

```json
{
  "permissions": [
    "http:default"
  ]
}
```

### Specific Domain Permissions

```json
{
  "permissions": [
    {
      "identifier": "http:allow-request",
      "allow": [
        {
          "url": "https://api.example.com/**"
        },
        {
          "url": "https://cdn.example.com/**"
        }
      ]
    }
  ]
}
```

## Notification Permissions

```json
{
  "permissions": [
    "notification:default",
    "notification:allow-notify",
    "notification:allow-request-permission"
  ]
}
```

## Multi-Window Permission Configuration

### Different Windows Different Permissions

```json
// capabilities/main-window.json
{
  "identifier": "main-window",
  "description": "Main window capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "fs:allow-appdata-read",
    "fs:allow-appdata-write"
  ]
}

// capabilities/tool-window.json
{
  "identifier": "tool-window",
  "description": "Tool window capabilities",
  "windows": ["tool"],
  "permissions": [
    "core:default",
    "shell:allow-open"
  ]
}
```

## Custom Permissions

### Create Custom Permissions

Create custom permission files in `src-tauri/permissions` directory:

```json
// permissions/my-custom-permissions.json
{
  "identifier": "my-app:custom-feature",
  "description": "Custom feature permissions",
  "commands": {
    "allow": ["my_custom_command"],
    "deny": []
  }
}
```

### Use Custom Permissions

```json
// capabilities/default.json
{
  "permissions": [
    "my-app:custom-feature"
  ]
}
```

## Permission Verification

### Runtime Permission Check

```rust
use tauri::Manager;

#[tauri::command]
fn check_permissions(app: tauri::AppHandle) -> Result<bool, String> {
    let scope = app.state::<tauri::Scopes>();

    // Check specific permission
    let has_permission = scope.check_permission("fs:allow-read-file");

    Ok(has_permission)
}
```

### Frontend Permission Check

```typescript
import { invoke } from '@tauri-apps/api/core';

// Check if specific permission exists
const hasPermission = await invoke('check_permissions');
```

## Permission Scope Variables

### Available Variables

```json
{
  "permissions": [
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$APPDATA/**" },
        { "path": "$DOCUMENTS/**" },
        { "path": "$DOWNLOAD/**" },
        { "path": "$PICTURE/**" },
        { "path": "$VIDEO/**" },
        { "path": "$AUDIO/**" },
        { "path": "$CACHE/**" },
        { "path": "$CONFIG/**" },
        { "path": "$TEMP/**" },
        { "path": "$HOME/**" },
        { "path": "$DESKTOP/**" },
        { "path": "$PUBLIC/**" },
        { "path": "$RESOURCE/**" }
      ]
    }
  ]
}
```

## Permission Inheritance

### Child Window Inheritance

```json
{
  "identifier": "parent-capability",
  "description": "Parent window capability",
  "windows": ["parent"],
  "permissions": [
    "core:default",
    "fs:allow-appdata-read"
  ],
  "platforms": ["windows", "macOS", "linux"]
}
```

### Platform-Specific Permissions

```json
// capabilities/windows-specific.json
{
  "identifier": "windows-specific",
  "description": "Windows specific capabilities",
  "platforms": ["windows"],
  "permissions": [
    "shell:allow-execute"
  ]
}
```

## Permission Debugging

### List Available Permissions

```bash
# List all available permissions
cargo tauri permission list

# View specific permission details
cargo tauri permission info fs:allow-read-file
```

### Permission Verification Tools

```rust
#[cfg(debug_assertions)]
#[tauri::command]
fn debug_permissions(app: tauri::AppHandle) {
    let scope = app.state::<tauri::Scopes>();

    // Print all permissions
    println!("Available permissions: {:?}", scope.available_permissions());

    // Check specific permission
    let check_result = scope.check_permission("fs:allow-read-file");
    println!("fs:allow-read-file: {}", check_result);
}
```

## Security Best Practices

### Principle of Least Privilege

```json
{
  "permissions": [
    "fs:allow-appdata-read"  // Read-only permission
  ]
}
```

### Fine-Grained Control

```json
{
  "permissions": [
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$APPDATA/configs/app.json" }  // Only allow access to specific file
      ]
    }
  ]
}
```

### Deny First

```json
{
  "permissions": [
    {
      "identifier": "fs:scope",
      "allow": [{ "path": "$APPDATA/**" }],
      "deny": [
        { "path": "$APPDATA/secrets/**" },
        { "path": "$APPDATA/sensitive/**" }
      ]
    }
  ]
}
```

## Permission Migration

### Migrate from v1 to v2

v1 configuration:
```json
{
  "tauri": {
    "allowlist": {
      "fs": {
        "all": true,
        "scope": ["$APPDATA/**"]
      }
    }
  }
}
```

v2 configuration:
```json
{
  "permissions": [
    "fs:default",
    {
      "identifier": "fs:scope",
      "allow": [{ "path": "$APPDATA/**" }]
    }
  ]
}
```

## Best Practices

1. **Least Privilege**: Request only necessary permissions
2. **Fine-Grained Control**: Use scopes to limit access
3. **Separate Permissions**: Create independent permissions for different modules
4. **Document**: Document the purpose of each permission
5. **Regular Review**: Regularly review and update permission configuration
6. **Testing**: Test application behavior when permissions are denied