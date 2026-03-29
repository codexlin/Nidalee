---
name: distribution
description: Distribution, signing, and publishing
metadata:
  tags: distribution, signing, release, publish, deploy
---

# Distribution and Signing

Tauri v2 application distribution involves code signing, package format selection, and release channel configuration.

## Code Signing

### macOS Code Signing

#### Prepare Certificate

1. Join Apple Developer Program
2. Create Developer ID Application certificate
3. Download and install certificate

#### Configure Signing

```bash
# Set environment variables
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
export APPLE_ID="your@email.com"
export APPLE_PASSWORD="app-specific-password"
export APPLE_TEAM_ID="TEAM_ID"

# Build
pnpm tauri build
```

#### Verify Signature

```bash
# Verify signature
codesign -dv --verbose=4 /path/to/YourApp.app

# Verify notarization
spctl -a -v /path/to/YourApp.app
```

### Windows Code Signing

#### Prepare Certificate

1. Purchase code signing certificate (e.g., DigiCert, Sectigo)
2. Install certificate to Windows certificate store

#### Configure Signing

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

#### Use Azure Key Vault

```json
{
  "bundle": {
    "windows": {
      "signCommand": "azure-keyvault-sign"
    }
  }
}
```

### Linux Signing

#### GPG Signing

```bash
# Create GPG key
gpg --gen-key

# Sign package
dpkg-sig --sign builder yourapp.deb

# Verify signature
dpkg-sig --verify yourapp.deb
```

## Auto Update

### Configure Updater

```bash
# Install updater plugin
cargo add tauri-plugin-updater
pnpm add @tauri-apps/plugin-updater
```

### Rust Configuration

```rust
fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend Implementation

```typescript
import { check } from '@tauri-apps/plugin-updater';
import { ask, message } from '@tauri-apps/plugin-dialog';

async function checkForUpdates() {
    const update = await check();

    if (update) {
        const yes = await ask(
            `Update to ${update.version} is available!\n\nRelease notes: ${update.body}`,
            { title: 'Update Available', kind: 'info' }
        );

        if (yes) {
            await update.downloadAndInstall();

            await message(
                'Update installed successfully! Please restart the application.',
                { title: 'Update Complete', kind: 'info' }
            );
        }
    }
}
```

### Update Server Configuration

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.myapp.com/{{target}}/{{arch}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

### Generate Update Keys

```bash
# Generate key pair
pnpm tauri signer generate

# Output:
# Private key: dW50cnVzdGVkIGNvbW1lbnQ... (save to secure location)
# Public key: dW50cnVzdGVkIGNvbW1lbnQ... (add to config)
```

## Release Channels

### GitHub Releases

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: pnpm tauri build

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            src-tauri/target/release/bundle/**/*.dmg
            src-tauri/target/release/bundle/**/*.msi
            src-tauri/target/release/bundle/**/*.deb
          draft: false
          prerelease: false
```

### App Stores

#### Mac App Store

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "3rd Party Mac Developer Application: Your Name",
      "entitlements": "macos/entitlements.plist"
    }
  }
}
```

#### Microsoft Store

1. Register Microsoft developer account
2. Create app submission
3. Upload MSIX package

#### Snap Store (Linux)

```yaml
# snapcraft.yaml
name: my-tauri-app
version: '1.0.0'
summary: My Tauri Application
description: |
  A description of my application

grade: stable
confinement: strict

parts:
  my-tauri-app:
    plugin: dump
    source: ./src-tauri/target/release/bundle/deb
    source-type: deb

apps:
  my-tauri-app:
    command: my-tauri-app
    extensions: [gnome-3-38]
    plugs:
      - browser-support
      - network
      - opengl
```

## Package Formats

### Windows

#### MSI Package

```json
{
  "bundle": {
    "targets": ["msi"]
  }
}
```

#### NSIS Package

```json
{
  "bundle": {
    "targets": ["nsis"],
    "windows": {
      "nsis": {
        "installerIcon": "icons/icon.ico",
        "headerImage": "assets/header.bmp",
        "sidebarImage": "assets/sidebar.bmp"
      }
    }
  }
}
```

### macOS

#### DMG Image

```json
{
  "bundle": {
    "targets": ["dmg"],
    "macOS": {
      "dmg": {
        "windowSize": {
          "width": 600,
          "height": 400
        },
        "appPosition": {
          "x": 170,
          "y": 170
        },
        "applicationFolderPosition": {
          "x": 430,
          "y": 170
        }
      }
    }
  }
}
```

#### App Bundle

```json
{
  "bundle": {
    "targets": ["app"]
  }
}
```

### Linux

#### DEB Package

```json
{
  "bundle": {
    "targets": ["deb"],
    "deb": {
      "depends": [
        "libgtk-3-0",
        "libwebkit2gtk-4.0-37"
      ]
    }
  }
}
```

#### AppImage

```json
{
  "bundle": {
    "targets": ["appimage"]
  }
}
```

#### RPM Package

```json
{
  "bundle": {
    "targets": ["rpm"],
    "rpm": {
      "depends": [
        "gtk3",
        "webkit2gtk3"
      ]
    }
  }
}
```

## Release Checklist

### Pre-Release Checks

- [ ] Version number updated
- [ ] All tests pass
- [ ] Changelog written
- [ ] Icons and metadata complete
- [ ] Code signed
- [ ] Auto-update configured
- [ ] Documentation updated

### Post-Release Checks

- [ ] Download links work
- [ ] Installation package installs correctly
- [ ] Application starts correctly
- [ ] Auto-update works
- [ ] Crash reporting works

## Best Practices

1. **Version Management**: Use semantic versioning (SemVer)
2. **Signing Certificates**: Protect private keys and certificates
3. **Test Release**: Test before official release
4. **Update Strategy**: Provide smooth update experience
5. **Rollback Plan**: Prepare rollback solution
6. **Monitoring**: Monitor application crashes and usage