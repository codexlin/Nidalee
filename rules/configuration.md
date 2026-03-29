---
name: configuration
description: tauri.conf.json configuration guide
metadata:
  tags: configuration, tauri.conf.json, settings, manifest
---

# Tauri Configuration Guide

`tauri.conf.json` is the core configuration file for Tauri applications.

## Basic Configuration

```json
{
  "$schema": "../node_modules/@tauri-apps/cli/config.schema.json",
  "productName": "My App",
  "version": "0.1.0",
  "identifier": "com.example.myapp",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  }
}
```

## App Configuration

### Window Configuration

```json
{
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "My App",
        "width": 800,
        "height": 600,
        "minWidth": 400,
        "minHeight": 300,
        "maxWidth": 1920,
        "maxHeight": 1080,
        "resizable": true,
        "maximized": false,
        "fullscreen": false,
        "decorations": true,
        "alwaysOnTop": false,
        "visible": true,
        "center": true,
        "transparent": false
      }
    ]
  }
}
```

### Security Configuration

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost",
      "dangerousDisableAssetCspModification": false,
      "freezePrototype": false,
      "dangerousRemoteDomainIpcAccess": []
    }
  }
}
```

### macOS Specific Configuration

```json
{
  "app": {
    "macOSPrivateApi": false,
    "exceptionDomain": ""
  }
}
```

## Bundle Configuration

```json
{
  "bundle": {
    "active": true,
    "targets": ["dmg", "app", "msi", "deb", "appimage"],
    "identifier": "com.example.myapp",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "category": "DeveloperTool",
    "shortDescription": "A Tauri App",
    "longDescription": "A longer description of your app",
    "deb": {
      "depends": []
    },
    "macOS": {
      "frameworks": [],
      "minimumSystemVersion": "10.13",
      "license": "",
      "entitlements": null
    },
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": ""
    }
  }
}
```

## Plugin Configuration

```json
{
  "plugins": {
    "shell": {
      "open": true
    },
    "fs": {
      "scope": ["$APPDATA/**", "$DOWNLOAD/**"]
    },
    "dialog": {
      "open": true,
      "save": true
    }
  }
}
```

## Complete Example

```json
{
  "$schema": "../node_modules/@tauri-apps/cli/config.schema.json",
  "productName": "My Awesome App",
  "version": "1.0.0",
  "identifier": "com.mycompany.awesomeapp",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173",
    "beforeDevCommand": "pnpm dev",
    "beforeBuildCommand": "pnpm build"
  },
  "app": {
    "windows": [
      {
        "label": "main",
        "title": "My Awesome App",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "center": true,
        "decorations": true,
        "transparent": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' asset: https://asset.localhost; connect-src 'self' https://api.example.com"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "identifier": "com.mycompany.awesomeapp",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "category": "Productivity",
    "shortDescription": "An awesome desktop app",
    "longDescription": "This is a longer description of my awesome app"
  }
}
```

## Environment-Specific Configuration

Use `tauri.macos.conf.json`, `tauri.windows.conf.json`, or `tauri.linux.conf.json` to override platform-specific configurations.

```json
// tauri.macos.conf.json
{
  "bundle": {
    "macOS": {
      "minimumSystemVersion": "11.0"
    }
  }
}
```