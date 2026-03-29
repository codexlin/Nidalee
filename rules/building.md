---
name: building
description: Build and bundle configuration
metadata:
  tags: build, bundle, package, compile, distribution
---

# Build and Bundle

Tauri v2 provides powerful build and bundle features for cross-platform application distribution.

## Development Build

### Development Mode

```bash
# Start development server
pnpm tauri dev

# Specify debug port
pnpm tauri dev -- --port 3000
```

### Frontend Build

```bash
# Build frontend (based on your frontend framework)
pnpm build
```

## Production Build

### Basic Build

```bash
# Build production version
pnpm tauri build

# Specify target platform
pnpm tauri build -- --target x86_64-pc-windows-msvc
```

### Build Targets

```bash
# Windows
pnpm tauri build -- --target x86_64-pc-windows-msvc
pnpm tauri build -- --target i686-pc-windows-msvc
pnpm tauri build -- --target aarch64-pc-windows-msvc

# macOS
pnpm tauri build -- --target x86_64-apple-darwin
pnpm tauri build -- --target aarch64-apple-darwin
pnpm tauri build -- --target universal-apple-darwin  # Universal binary

# Linux
pnpm tauri build -- --target x86_64-unknown-linux-gnu
pnpm tauri build -- --target aarch64-unknown-linux-gnu
```

## Bundle Configuration

### tauri.conf.json Bundle Configuration

```json
{
  "bundle": {
    "active": true,
    "targets": ["dmg", "app", "msi", "deb", "appimage", "rpm", "nsis"],
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
      "depends": ["libgtk-3-0", "libwebkit2gtk-4.0-37"]
    },
    "macOS": {
      "frameworks": [],
      "minimumSystemVersion": "10.13",
      "license": "",
      "entitlements": null,
      "exceptionDomain": ""
    },
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "",
      "webviewInstallMode": {
        "type": "downloadBootstrapper"
      }
    }
  }
}
```

### Platform-Specific Configuration

#### macOS Configuration

```json
{
  "bundle": {
    "macOS": {
      "minimumSystemVersion": "10.13",
      "entitlements": "entitlements.plist",
      "exceptionDomain": "example.com",
      "frameworks": ["WebKit"],
      "license": "LICENSE"
    }
  }
}
```

#### Windows Configuration

```json
{
  "bundle": {
    "windows": {
      "webviewInstallMode": {
        "type": "embedBootstrapper"
      },
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

#### Linux Configuration

```json
{
  "bundle": {
    "deb": {
      "depends": [
        "libgtk-3-0",
        "libwebkit2gtk-4.0-37"
      ]
    }
  }
}
```

## Build Output

### Output Directory Structure

```
src-tauri/target/
├── release/
│   ├── bundle/
│   │   ├── dmg/           # macOS DMG
│   │   ├── app/           # macOS App Bundle
│   │   ├── msi/           # Windows MSI
│   │   ├── nsis/          # Windows NSIS
│   │   ├── deb/           # Linux DEB
│   │   ├── rpm/           # Linux RPM
│   │   └── appimage/      # Linux AppImage
│   └── myapp              # Executable
```

## Icon Configuration

### Icon Requirements

```
icons/
├── 32x32.png              # Windows/Linux taskbar
├── 128x128.png            # General icon
├── 128x128@2x.png         # Retina display
├── icon.icns              # macOS icon set
└── icon.ico               # Windows icon
```

### Generate Icons

```bash
# Generate icons using Tauri CLI
pnpm tauri icon /path/to/source.png

# Generate specific size icons
pnpm tauri icon -- --input /path/to/source.png --output ./icons
```

## Environment Variables

### Build-Time Environment Variables

```bash
# Set application version
TAURI_PRIVATE_KEY=path/to/key pnpm tauri build

# Set debug mode
TAURI_DEBUG=1 pnpm tauri build

# Set target platform
TAURI_TARGET_TRIPLE=x86_64-pc-windows-msvc pnpm tauri build
```

### Runtime Environment Variables

```rust
// Read environment variables in Rust
use std::env;

fn main() {
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    println!("Version: {}", version);
}
```

## Optimize Build

### Release Optimization

```toml
# Cargo.toml
[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"  # or "z" for smaller size
strip = true
```

### Frontend Optimization

```javascript
// vite.config.ts
import { defineConfig } from 'vite';

export default defineConfig({
  build: {
    minify: 'terser',
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
        },
      },
    },
  },
});
```

## CI/CD Build

### GitHub Actions

```yaml
# .github/workflows/build.yml
name: Build

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  build:
    strategy:
      matrix:
        platform: [macos-latest, ubuntu-latest, windows-latest]

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev

      - name: Install frontend dependencies
        run: pnpm install

      - name: Build Tauri
        run: pnpm tauri build

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.platform }}-artifacts
          path: |
            src-tauri/target/release/bundle
```

### Multi-Platform Build

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    strategy:
      matrix:
        include:
          - platform: macos-latest
            args: '--target universal-apple-darwin'
          - platform: ubuntu-latest
            args: ''
          - platform: windows-latest
            args: ''

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev

      - name: Install frontend dependencies
        run: pnpm install

      - name: Build Tauri
        run: pnpm tauri build -- ${{ matrix.args }}

      - name: Upload to Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            src-tauri/target/release/bundle/dmg/*.dmg
            src-tauri/target/release/bundle/msi/*.msi
            src-tauri/target/release/bundle/deb/*.deb
```

## Best Practices

1. **Version Management**: Use semantic versioning
2. **Icon Preparation**: Provide all required icon sizes
3. **Test Builds**: Test builds for all target platforms in CI
4. **Size Optimization**: Use appropriate optimization options to reduce app size
5. **Code Signing**: Configure code signing for production builds
6. **Documentation**: Provide clear build and installation instructions