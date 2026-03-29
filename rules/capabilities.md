---
name: capabilities
description: Capabilities configuration and scope
metadata:
  tags: capabilities, scope, security, access control
---

# Capabilities Configuration and Scope

Capabilities is the core mechanism in Tauri v2 for defining application functionality access permissions.

## Capability File Structure

### Basic Capability File

```json
{
  "$schema": "../gen/schemas/capabilities-schema.json",
  "identifier": "main-capability",
  "description": "Main application capabilities",
  "windows": ["main", "secondary"],
  "permissions": [
    "core:default",
    "fs:allow-appdata-read"
  ]
}
```

### Complete Capability Configuration

```json
{
  "$schema": "../gen/schemas/capabilities-schema.json",
  "identifier": "main",
  "description": "Main application capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "fs:allow-appdata-read",
    "fs:allow-appdata-write",
    "shell:allow-open"
  ],
  "platforms": ["windows", "macOS", "linux"],
  "webviews": ["main"],
  "remote": {
    "urls": ["https://example.com"]
  }
}
```

## Capability Scope

### Window Scope

```json
{
  "identifier": "main-window",
  "windows": ["main"],
  "permissions": [
    "core:window:allow-create",
    "core:window:allow-close"
  ]
}
```

### WebView Scope

```json
{
  "identifier": "webview-scope",
  "webviews": ["main", "secondary"],
  "permissions": [
    "core:webview:allow-create",
    "core:webview:allow-print"
  ]
}
```

### Platform Scope

```json
{
  "identifier": "platform-specific",
  "platforms": ["windows"],
  "permissions": [
    "shell:allow-execute"
  ]
}
```

## Permission Scope

### File System Scope

```json
{
  "permissions": [
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$APPDATA/configs/**" },
        { "path": "$DOCUMENTS/myapp/**" },
        { "path": "$DOWNLOAD/temp/**" }
      ],
      "deny": [
        { "path": "$APPDATA/configs/secrets/**" }
      ]
    }
  ]
}
```

### Shell Scope

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
          "args": true
        }
      ]
    }
  ]
}
```

### HTTP Scope

```json
{
  "permissions": [
    {
      "identifier": "http:allow-request",
      "allow": [
        {
          "url": "https://api.example.com/**",
          "methods": ["GET", "POST", "PUT", "DELETE"]
        },
        {
          "url": "https://cdn.example.com/**",
          "methods": ["GET"]
        }
      ]
    }
  ]
}
```

## Multi-Capability Configuration

### Layered Capabilities

```json
// capabilities/base.json
{
  "identifier": "base",
  "description": "Base capabilities for all windows",
  "permissions": [
    "core:default"
  ]
}

// capabilities/main.json
{
  "identifier": "main",
  "description": "Main window capabilities",
  "windows": ["main"],
  "permissions": [
    "fs:allow-appdata-read",
    "fs:allow-appdata-write"
  ]
}

// capabilities/tool.json
{
  "identifier": "tool",
  "description": "Tool window capabilities",
  "windows": ["tool"],
  "permissions": [
    "shell:allow-open"
  ]
}
```

### Feature-Specific Capabilities

```json
// capabilities/file-manager.json
{
  "identifier": "file-manager",
  "description": "File manager capabilities",
  "permissions": [
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$DOCUMENTS/**" },
        { "path": "$PICTURE/**" },
        { "path": "$VIDEO/**" }
      ]
    },
    "fs:allow-read-file",
    "fs:allow-write-file",
    "fs:allow-copy-file",
    "fs:allow-mkdir",
    "fs:allow-remove"
  ]
}

// capabilities/network.json
{
  "identifier": "network",
  "description": "Network capabilities",
  "permissions": [
    {
      "identifier": "http:allow-request",
      "allow": [
        {
          "url": "https://api.example.com/**"
        }
      ]
    }
  ]
}
```

## Dynamic Capability Management

### Runtime Capability Check

```rust
use tauri::Manager;

#[tauri::command]
fn check_capability(app: tauri::AppHandle, capability: String) -> bool {
    let scopes = app.state::<tauri::Scopes>();
    scopes.check_capability(&capability)
}

#[tauri::command]
fn list_capabilities(app: tauri::AppHandle) -> Vec<String> {
    let scopes = app.state::<tauri::Scopes>();
    scopes.available_capabilities()
}
```

### Frontend Capability Check

```typescript
import { invoke } from '@tauri-apps/api/core';

// Check if specific capability exists
const hasCapability = await invoke('check_capability', {
    capability: 'file-manager'
});

// List all available capabilities
const capabilities = await invoke('list_capabilities');
```

## Capability Inheritance

### Parent Window Inheritance

```json
{
  "identifier": "parent-window",
  "windows": ["parent"],
  "permissions": [
    "core:default",
    "fs:allow-appdata-read"
  ]
}

// Child window automatically inherits parent window capabilities
{
  "identifier": "child-window",
  "windows": ["child"],
  "permissions": [
    "shell:allow-open"  // Additional permissions
  ]
}
```

### Capability Composition

```json
{
  "identifier": "combined",
  "description": "Combined capabilities",
  "permissions": [
    "file-manager",      // Reference other capabilities
    "network",
    "shell:allow-open"   // Additional permissions
  ]
}
```

## Environment-Specific Capabilities

### Development Environment

```json
// capabilities/development.json
{
  "identifier": "development",
  "description": "Development environment capabilities",
  "context": {
    "development": true
  },
  "permissions": [
    "shell:allow-execute",
    "fs:allow-appdata-write"
  ]
}
```

### Production Environment

```json
// capabilities/production.json
{
  "identifier": "production",
  "description": "Production environment capabilities",
  "context": {
    "production": true
  },
  "permissions": [
    "fs:allow-appdata-read"
  ]
}
```

## Capability Validation

### Validate Capability Files

```bash
# Validate capability file format
cargo tauri capability validate

# Check for capability conflicts
cargo tauri capability check

# Generate capability documentation
cargo tauri capability docs
```

### Runtime Validation

```rust
#[cfg(debug_assertions)]
#[tauri::command]
fn debug_capabilities(app: tauri::AppHandle) {
    let scopes = app.state::<tauri::Scopes>();

    // Print all capabilities
    println!("Available capabilities: {:?}", scopes.available_capabilities());

    // Check specific capability
    for capability in scopes.available_capabilities() {
        let permissions = scopes.get_capability_permissions(&capability);
        println!("Capability {}: {:?}", capability, permissions);
    }
}
```

## Capability Scope Best Practices

### Principle of Least Privilege

```json
{
  "identifier": "minimal-file-access",
  "permissions": [
    {
      "identifier": "fs:scope",
      "allow": [
        { "path": "$APPDATA/config.json" }  // Only allow access to specific file
      ]
    }
  ]
}
```

### Layered Permission Design

```json
// Base layer
{
  "identifier": "base",
  "permissions": ["core:default"]
}

// Feature layer
{
  "identifier": "file-operations",
  "permissions": [
    "fs:allow-read-file",
    "fs:allow-write-file"
  ]
}

// Application layer
{
  "identifier": "my-app",
  "permissions": [
    "base",
    "file-operations",
    "shell:allow-open"
  ]
}
```

### Window Isolation

```json
// Main window - Full permissions
{
  "identifier": "main-window",
  "windows": ["main"],
  "permissions": [
    "fs:allow-appdata-read",
    "fs:allow-appdata-write",
    "shell:allow-open"
  ]
}

// Tool window - Restricted permissions
{
  "identifier": "tool-window",
  "windows": ["tool"],
  "permissions": [
    "shell:allow-open"
  ]
}
```

## Capability Migration

### Migrate from v1 to v2

v1 configuration:
```json
{
  "tauri": {
    "allowlist": {
      "fs": {
        "all": true,
        "scope": ["$APPDATA/**"]
      },
      "shell": {
        "all": true,
        "execute": true,
        "open": true
      }
    }
  }
}
```

v2 capability configuration:
```json
{
  "identifier": "migrated",
  "permissions": [
    "fs:default",
    {
      "identifier": "fs:scope",
      "allow": [{ "path": "$APPDATA/**" }]
    },
    "shell:default",
    "shell:allow-open",
    {
      "identifier": "shell:allow-execute",
      "allow": [{ "name": "all", "cmd": "*", "args": true }]
    }
  ]
}
```

## Capability Debugging Tools

### Development Mode Debugging

```rust
#[cfg(debug_assertions)]
fn setup_debug_tools(app: &mut tauri::App) {
    app.on_page_load(|window, _| {
        let scopes = window.state::<tauri::Scopes>();

        // Print window capabilities
        println!("Window {} capabilities:", window.label());
        for capability in scopes.window_capabilities(&window.label()) {
            println!("  - {}", capability);
        }
    });
}
```

## Best Practices

1. **Modular Design**: Organize capabilities by functional modules
2. **Least Privilege**: Grant only necessary permissions
3. **Layered Management**: Use capability inheritance and composition
4. **Documentation**: Provide detailed descriptions for each capability
5. **Testing**: Test application behavior when permissions are denied
6. **Audit**: Regularly review and update capability configuration
7. **Environment Separation**: Use different capability configurations for different environments