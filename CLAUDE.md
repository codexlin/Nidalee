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

> **TIPS**: Do NOT manually import composables/stores from `src/shared/composables/**` or `src/shared/stores/**`. They are auto-imported by `unplugin-auto-import`. See `types/auto-imports.d.ts` for the full list of auto-imported symbols.

### UI Components
- Uses shadcn-vue (New York style, neutral base color)
- Components in `src/components/ui/`
- Icons: lucide-vue-next
- Component resolution via `unplugin-vue-components`

## Data Classification

Understanding the data lifecycle helps optimize caching and reduce unnecessary API calls.

### Static Data (版本化缓存)
**Definition**: Data that only changes when the LoL game version updates (typically every 2 weeks).

**Cache Strategy**: Use `['static', 'category', version]` as query key. Invalidated automatically when game version changes.

**Query Key Pattern**: `['static', '<category>', version]`

| Data | Query Key | Source | Composable |
|------|-----------|--------|------------|
| 英雄列表 (Champions) | `['static', 'champions', version]` | LCU API | `useChampions()` |
| 符文样式 (Rune Styles) | `['static', 'runes', version]` | LCU API | `useRuneStyles()` |
| 符文详情 (Perks) | `['static', 'perks', version]` | LCU API | `usePerks()` |
| 符文图标 (Perk Icons) | `['static', 'perkIcons', version]` | LCU API | `usePerkIcons()` |
| 召唤师技能 (Summoner Spells) | `['static', 'spells', version]` | LCU API | `useSummonerSpells()` |
| 游戏物品 (Items) | `['static', 'items', version]` | LCU API | - |
| 游戏模式 (Game Modes) | `['static', 'gameModes', version]` | LCU API | - |
| 地图 (Maps) | `['static', 'maps', version]` | LCU API | - |
| 队列 (Queues) | `['static', 'queues', version]` | LCU API | - |

**Cache Config**: `staleTime: Infinity, gcTime: Infinity`

---

### Semi-Static Data (会话缓存)
**Definition**: Data that changes per game session or user action, but doesn't need frequent updates.

**Cache Strategy**: Medium cache time (minutes to hours), manual refresh when needed.

| Data | Query Key | Source | Cache Time |
|------|-----------|--------|------------|
| 当前召唤师信息 (Current Summoner) | `['currentSummoner']` | LCU API | 5 min |
| 战绩历史 (Match History) | `['matchHistory', summonerId]` | LCU API | 5 min |
| 英雄详情/皮肤 (Champion Details) | `['championDetails', championId]` | Community Dragon | 1 hour |
| OP.GG 推荐 (OP.GG Builds) | `['opgg', 'build', championId, position, version]` | OP.GG API | 1 hour |
| OP.GG 强度榜 (OP.GG Tier List) | `['opgg', 'tierList', version]` | OP.GG API | 1 hour |
| 当前符文页 (Current Rune Page) | `['currentRunePage']` | LCU API | 1 min |

---

### Dynamic Data (轮询数据)
**Definition**: Data that changes frequently during gameplay. Requires periodic polling.

**Cache Strategy**: Short cache time, refreshed via polling or WebSocket fallback.

| Data | Query Key | Polling Interval | Fallback Strategy |
|------|-----------|------------------|-------------------|
| 匹配状态 (Matchmaking State) | `['matchmakingState']` | 5-15s | WebSocket + HTTP fallback |
| 大厅信息 (Lobby Info) | `['lobby']` | 5-15s | WebSocket + HTTP fallback |
| 英雄选择会话 (Champ Select) | `['champSelect']` | Real-time | WebSocket only |

---

### Real-Time Data (WebSocket 实时推送)
**Definition**: Data that must be updated immediately when changes occur. Delivered via LCU WebSocket.

**WebSocket Events**: All prefixed with `/` and monitored via `OnJsonApiEvent`

| Event Name | LCU Path | Trigger | Frontend Handler |
|------------|----------|---------|------------------|
| `gameflow-phase-change` | `/lol-gameflow/v1/gameflow-phase` | Game phase changes | `handleGamePhaseChange()` |
| `lobby-change` | `/lol-lobby/v2/lobby` | Lobby updated | `handleLobbyChange()` |
| `champ-select-session-changed` | `/lol-champ-select/v1/session` | Champ select updates | `handleChampSelectChange()` |
| `matchmaking-state-changed` | `/lol-matchmaking/v1/search` | Matchmaking status | `matchmakingStore.updateState()` |
| `connection-state-changed` | - | Connection status | `connectionStore.updateConnectionState()` |
| `game-finished` | - | Game ends | `matchAnalysisStore.clearAllData()` |
| `team-analysis-data` | - | Analysis complete | `matchAnalysisStore.setTeamAnalysisData()` |

**WebSocket Subscription** (dynamic based on game phase):
```rust
// Always subscribed
"/lol-gameflow/v1/gameflow-phase"
"/lol-gameflow/v1/session"
"/lol-summoner/v1/current-summoner"

// Phase-specific subscriptions
"Lobby/Matchmaking/None" → "/lol-lobby/v2/lobby", "/lol-matchmaking/v1/search"
"ChampSelect" → "/lol-champ-select/v1/session"
```

---

### Cache Invalidation Strategy

**Version Change Detection**:
```typescript
// On connection established, check game version
if (newVersion !== currentVersion) {
  queryClient.invalidateQueries({ queryKey: ['static'] })
}
```

**Manual Refresh**:
```typescript
// Refresh all static data
await queryClient.invalidateQueries({ queryKey: ['gameVersion'] })

// Refresh specific category
await queryClient.invalidateQueries({ queryKey: ['static', 'champions'] })
```

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
