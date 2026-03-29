---
name: tauri-v2-best-practices
description: Best practices for Tauri v2 - Build desktop applications with web technologies
metadata:
  tags: tauri, v2, desktop, rust, javascript, typescript, cross-platform
---

## When to use

Use this skills whenever you are dealing with Tauri v2 code to obtain the domain-specific knowledge.

## How to use

Read individual rule files for detailed explanations and code examples:

### Core Architecture
- [rules/project-structure.md](rules/project-structure.md) - Tauri v2 project structure and file organization
- [rules/commands.md](rules/commands.md) - Frontend-Backend communication (Commands and Events)
- [rules/configuration.md](rules/configuration.md) - tauri.conf.json configuration guide

### Window Management
- [rules/window-management.md](rules/window-management.md) - Window creation, management, and control
- [rules/window-customization.md](rules/window-customization.md) - Custom window styles (transparent, frameless, rounded corners)

### System Interaction
- [rules/filesystem.md](rules/filesystem.md) - File system operations (read, write, watch)
- [rules/shell.md](rules/shell.md) - Execute shell commands and open external programs
- [rules/notifications.md](rules/notifications.md) - System notifications
- [rules/clipboard.md](rules/clipboard.md) - Clipboard operations
- [rules/dialog.md](rules/dialog.md) - System dialogs (file picker, message boxes)

### Security
- [rules/permissions.md](rules/permissions.md) - Permission system (Tauri v2 new feature)
- [rules/capabilities.md](rules/capabilities.md) - Capability configuration and scopes

### Advanced Features
- [rules/tray.md](rules/tray.md) - System tray
- [rules/menu.md](rules/menu.md) - Application menus (native and context menus)

### Frontend Integration
- [rules/frontend-integration.md](rules/frontend-integration.md) - Integration with frontend frameworks

### Deployment
- [rules/building.md](rules/building.md) - Build and bundle configuration
- [rules/distribution.md](rules/distribution.md) - Distribution, signing, and release

## Quick Reference

### Package Manager

This project uses **pnpm** for package management:

```bash
# Install dependencies
pnpm install

# Add frontend package
pnpm add @tauri-apps/plugin-<name>

# Add Rust plugin
cargo add tauri-plugin-<name>

# Development server
pnpm tauri dev

# Build production
pnpm tauri build
```

### Key Concepts

- **Commands**: Frontend calls to Rust functions via `invoke()`
- **Events**: Bidirectional communication between frontend and backend
- **Permissions**: Capability-based security model
- **Plugins**: Official plugins for common functionality