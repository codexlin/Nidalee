# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Package Manager
This project uses **pnpm** (required >=10.0.0, Node >=20.0.0).

```bash
# Install dependencies
pnpm install

# Development (frontend dev server + Tauri)
pnpm dev

# Build for production
pnpm build              # Lints then builds frontend
pnpm build:prod         # Production build
pnpm build:analyze      # Build with bundle analysis

# Type checking
pnpm type-check         # Vue TypeScript check
pnpm types              # Generate TypeScript types from Rust

# Linting and formatting
pnpm lint               # Run oxlint + eslint
pnpm lint:oxlint        # Oxlint only (fast)
pnpm lint:eslint        # ESLint only
pnpm format             # Prettier format
```

### Rust Backend
```bash
# From src-tauri directory
cargo check             # Check compilation
cargo build             # Build
cargo test              # Run tests (76 tests)
cargo clippy            # Lint
```

### Tauri Commands
```bash
# Run full development environment
pnpm tauri dev

# Build application bundle
pnpm tauri build        # Creates .msi installer
```

## Architecture

### Technology Stack
- **Frontend**: Vue 3 + TypeScript + Vite
- **Backend**: Rust + Tauri 2.5.0
- **UI**: shadcn-vue (New York style) + Tailwind CSS 4.x
- **State**: Pinia with persistence
- **Communication**: LCU (League Client Protocol) + WebSocket

### Directory Structure

```
src/                          # Frontend source
├── features/                 # Feature-based modules (dashboard, opgg, match-analysis, etc.)
├── shared/                   # Shared frontend code
│   ├── components/           # Shared UI components
│   ├── composables/          # Vue composables (auto-imported)
│   ├── stores/               # Pinia stores (auto-imported)
│   └── utils/                # Utility functions
├── components/               # Global components
│   ├── common/               # Common UI elements
│   ├── layout/               # Layout components
│   └── ui/                   # shadcn-vue components
└── types/                    # TypeScript definitions

src-tauri/                    # Rust backend
├── src/
│   ├── domains/              # Domain layer (DDD) - pure business logic
│   │   ├── analysis/         # Match analysis, player stats
│   │   └── tactical_advice/  # Tactical advice generation
│   ├── infrastructure/       # Infrastructure layer - LCU API, caching
│   │   ├── champion_selection/
│   │   ├── data_services/    # External data (OP.GG)
│   │   ├── game_session/     # Auth, connection, lobby
│   │   ├── match_management/ # Matches, matchmaking
│   │   └── real_time/        # WebSocket
│   └── shared/               # Shared Rust code (types, utils, errors)
└── capabilities/             # Tauri v2 permissions
```

### DDD Layering (Rust Backend)
- **Domain Layer** (`domains/`): Core business logic, no external dependencies
- **Infrastructure Layer** (`infrastructure/`): LCU API calls, WebSocket, caching
- **Application Layer**: Use case orchestration (in service modules)

Dependency direction: `Infrastructure → Application → Domain`

## Key Patterns

### Type Generation (Rust → TypeScript)
Types defined in Rust using `#[derive(TsExpr)]` from `ts-rs` are auto-generated to TypeScript:
- Generated in `types/` directory
- Synced to `src/types/global.d.ts` via `pnpm types`
- Run after modifying Rust types: `cd src-tauri && cargo test -- --nocapture && cd .. && node scripts/sync-types.mjs`

### Tauri Commands
- Backend commands registered in `src-tauri/src/lib.rs` via `invoke_handler!`
- Frontend calls via `invoke<ReturnType>('command_name', { args })`
- All commands defined in `infrastructure/*/commands/*.rs`

### Feature-Based Frontend Structure
Each feature in `src/features/` is self-contained with:
- `index.vue` - Main component
- `components/` - Feature-specific components
- `composables/` - Feature logic

### Auto-Import Configuration
- **Composables**: All files in `src/shared/composables/**` are auto-imported
- **Stores**: All files in `src/shared/stores/**` are auto-imported
- **Components**: From `src/components/**` and `src/features/**`
- Vue APIs: `vue`, `vue-router`, `pinia`

### UI Components
- Uses shadcn-vue (New York style, neutral base color)
- Components in `src/components/ui/`
- Icons: lucide-vue-next
- Component resolution via `unplugin-vue-components`

## Important Notes

### WebSocket (LCU)
- LCU WebSocket connection managed in `infrastructure/real_time/websocket/`
- Commands: `start_lcu_ws`, `stop_lcu_ws`
- Event handling in `event_handler.rs`

### Game Session Management
- Connection state: `infrastructure/game_session/connection/`
- Authentication: `infrastructure/game_session/auth/`
- Lobby: `infrastructure/game_session/lobby/`

### Match Analysis
- Player stats analyzer: `domains/analysis/`
- Backend-heavy approach - analysis done in Rust, frontend displays results
- Supports queue filtering (420=ranked solo, 440=ranked flex)

### Build Configuration
- Dev server: `http://localhost:1422`
- Dev build output: `dist/`
- Tauri dev URL matches frontend dev server
- Production bundle analysis: `pnpm build:analyze`

### Windows-Specific
- Target platform: Windows only
- Installer: `.msi` via WiX
- Game path detection via Windows registry

## Testing
- Rust tests: `cargo test` (76 tests in suite)
- Test files co-located with source code

## Common Tasks

### Adding a new Tauri command
1. Create command function in `src-tauri/src/infrastructure/<module>/commands/*.rs`
2. Register in `src-tauri/src/lib.rs` invoke_handler macro
3. Add permission to `src-tauri/capabilities/default.json` if needed
4. Frontend: `invoke('command_name', { args })`

### Adding a new frontend feature
1. Create directory under `src/features/<feature-name>/`
2. Add route in `src/router/`
3. Components are auto-discovered for import

### After modifying Rust types
```bash
cd src-tauri && cargo test -- --nocapture && cd .. && node scripts/sync-types.mjs
```
