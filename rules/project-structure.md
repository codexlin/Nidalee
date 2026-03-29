---
name: project-structure
description: Tauri v2 project structure and file organization
metadata:
  tags: project, structure, architecture, organization
---

# Tauri v2 Project Structure

## Standard Project Structure

```
my-tauri-app/
├── src/                    # Frontend source code
│   ├── main.ts            # Frontend entry file
│   ├── App.vue/App.tsx    # Frontend main component
│   └── assets/            # Frontend resource files
├── src-tauri/             # Tauri backend code
│   ├── src/
│   │   └── main.rs        # Rust entry file
│   ├── Cargo.toml         # Rust dependency configuration
│   ├── tauri.conf.json    # Tauri configuration file
│   ├── capabilities/      # Capabilities configuration (v2 new feature)
│   │   └── default.json
│   └── icons/             # Application icons
├── package.json           # Node.js dependencies
└── index.html             # Frontend HTML entry
```

## Key File Descriptions

### Cargo.toml

Define Rust dependencies and project metadata:

```toml
[package]
name = "my-tauri-app"
version = "0.1.0"
edition = "2024"

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[features]
default = ["custom-protocol"]
custom-protocol = ["tauri/custom-protocol"]
```

### tauri.conf.json

Main configuration file, defining application metadata, window configuration, etc.:

```json
{
  "productName": "My App",
  "version": "0.1.0",
  "identifier": "com.example.myapp",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:5173"
  },
  "app": {
    "windows": [
      {
        "title": "My App",
        "width": 800,
        "height": 600
      }
    ]
  }
}
```

## Frontend Framework Support

Tauri v2 supports multiple frontend frameworks:

**Vite + React:**
```bash
pnpm create tauri-app@latest -- --template vite-react-ts
```

**Vite + Vue:**
```bash
pnpm create tauri-app@latest -- --template vite-vue-ts
```

**Vite + Svelte:**
```bash
pnpm create tauri-app@latest -- --template vite-svelte-ts
```

**Next.js:**
```bash
pnpm create tauri-app@latest -- --template nextjs-ts
```

## Development Commands

```bash
# Development mode (start both frontend and backend)
pnpm tauri dev

# Build production version
pnpm tauri build

# Build frontend only
pnpm run build
```

## Best Practices

1. **Separation of Concerns**: Keep business logic in frontend, system operations in Rust backend
2. **Type Safety**: Use TypeScript and Rust type systems, share interface definitions
3. **Error Handling**: Use `Result` type in Rust, try-catch in frontend
4. **Resource Management**: Put static resources in `src-tauri/` or frontend `public/` directory