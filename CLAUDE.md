# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Package Manager
This project uses **pnpm** (required >=10.0.0, Node >=22.18.0).

```bash
# Install dependencies
pnpm install

# Development
pnpm dev          # Vite only (frontend dev server)
pnpm dev:app      # Tauri dev (full desktop app with Rust)

# Build for production
pnpm build              # Lint + format:check + type-check + test + vite build
pnpm build:prod         # Production build (NODE_ENV=production)
pnpm build:analyze      # Build with bundle analysis (dist/stats.html)

# Type checking
pnpm type-check         # Vue TypeScript check (vue-tsc --build)
pnpm types              # Full Rust → TypeScript contract regen
pnpm types:sync         # Sync only (assumes types already generated)
pnpm types:check        # Regen + verify git diff on global.d.ts (CI gate)

# Linting, formatting, testing
pnpm lint               # oxlint src/ --fix
pnpm lint:fix           # same (alias)
pnpm format             # oxfmt src/
pnpm format:check       # oxfmt --check (CI gate)
pnpm test               # vitest run (frontend unit tests)
pnpm test:watch         # vitest watch mode
```

### Rust Backend
```bash
# From src-tauri directory
cargo check             # Check compilation (all-targets, locked)
cargo build             # Build
cargo test              # Run all tests (lib + integration under tests/)
cargo clippy            # Lint
cargo fmt --all -- --check   # Format check (CI gate)
```

### Tauri Commands
```bash
# Run full development environment
pnpm tauri dev

# Build application bundle
pnpm tauri build        # Creates Windows .msi + macOS .dmg installers
```

## Architecture

### Technology Stack
- **Frontend**: Vue 3.5 + TypeScript + Vite 8
- **Backend**: Rust + Tauri 2.11 (tray-icon, image-png)
- **UI**: shadcn-vue (New York style, neutral) + Tailwind CSS 4.x + Reka UI
- **State**: Pinia 3 with `pinia-plugin-persistedstate`
- **Data fetching**: TanStack Vue Query 5 (with versioned cache keys)
- **Charts**: Chart.js + custom ThemedChart wrappers
- **Communication**: LCU (League Client Protocol) via HTTPS + WebSocket

### Directory Structure

```
src/                          # Frontend source
├── features/                 # Feature-based modules (dashboard, opgg, match-analysis, ...)
├── shared/                   # Shared frontend code
│   ├── components/           # Auto-imported UI (charts, etc.)
│   ├── composables/          # Vue composables (auto-imported)
│   ├── stores/               # Pinia stores (auto-imported)
│   └── utils/                # Utility functions
├── components/               # Global components (common/, layout/, ui/)
└── types/                    # TypeScript definitions (incl. auto-generated contracts)

src-tauri/                    # Rust backend
├── src/
│   ├── domains/              # DDD domain layer (pure business logic)
│   │   ├── analysis/         # Pipeline, evidence, traits, analyzers
│   │   ├── tactical_advice/  # Laning/farming/vision/teamfight strategies
│   │   └── ai_analysis/      # AI prompt construction, insight types
│   ├── infrastructure/       # Adapters: LCU API, WebSocket, caching
│   │   ├── champion_selection/  # champ_select, perks, summoner_spells
│   │   ├── data_services/    # summoner, champion_data, external/{opgg, ai}
│   │   ├── game_session/     # auth, connection, gameflow, lobby
│   │   ├── match_management/ # matches, matchmaking, analysis_data, ranked
│   │   └── real_time/        # websocket, liveclient
│   ├── shared/               # Cross-cutting types/errors/utils
│   ├── common/               # Cross-cutting Tauri commands (machine, builds, game, ...)
│   └── [top-level facades]   # analysis_contract, match_fetching, match_analysis, ai_contract
└── tests/                    # Integration tests (analysis_evidence, analysis_orchestrator)
```

### DDD Layering (Rust Backend)
- **Domain Layer** (`domains/`): Pure business logic, no external dependencies
- **Infrastructure Layer** (`infrastructure/`): LCU API calls, WebSocket, caching
- **Top-level facades** (`analysis_contract`, `match_fetching`, `match_analysis`, `ai_contract`): Minimal re-export surfaces for integration tests and `ts-rs` generation. `infrastructure` and `domains` stay crate-private.

Dependency direction: `Infrastructure → Domain`, with `ts-rs` and integration tests consuming only the top-level facades.

## Key Patterns

### Type Generation (Rust → TypeScript)

The contract is enforced by CI. Three-step pipeline:

```bash
node scripts/prepare-types.mjs                          # Clear stale generated files
cd src-tauri && cargo test --all-targets --locked       # ts-rs generates .ts into src-tauri/bindings/
cd .. && node scripts/sync-types.mjs                     # Aggregate into src/types/global.d.ts
git diff --exit-code -- src/types/global.d.ts           # CI gate: must be empty
```

- Use `#[derive(TS)]` from `ts-rs` on Rust types meant to cross the IPC boundary
- `#[ts(export, export_to = "../../src/types/generated/X.ts")]` controls the export path
- `src/types/global.d.ts` is **generated, never hand-edited**
- `pnpm types:check` is the CI gate that blocks contract drift

### Tauri Commands

- Backend commands registered in `src-tauri/src/lib.rs` via `invoke_handler!` (grouped by domain with comment headers)
- Each infrastructure subdomain exposes its own `commands.rs` (e.g. `infrastructure/real_time/liveclient/commands.rs`)
- Frontend calls via `invoke<ReturnType>('command_name', { args })` — command names match Rust function names, paths are only registration concerns
- Some `#[cfg(debug_assertions)]`-gated dev commands live under `common/commands/data_collection.rs`

### Feature-Based Frontend Structure
Each feature in `src/features/` is self-contained with:
- `<Name>.vue` - Main component
- `components/` - Feature-specific components
- `composables/` - Feature-local logic (optional)

### Auto-Import Configuration
- **Composables**: All files in `src/shared/composables/**` are auto-imported
- **Stores**: All files in `src/shared/stores/**` are auto-imported
- **Components**: From `src/components/**` and `src/features/**`
- Vue APIs: `vue`, `vue-router`, `pinia`

> **TIPS**: Do NOT manually import composables/stores from `src/shared/composables/**` or `src/shared/stores/**`. They are auto-imported by `unplugin-auto-import`. See `types/auto-imports.d.ts` for the full list.

### UI Components
- shadcn-vue (New York style, neutral base color) wrapped around Reka UI primitives
- Components in `src/components/ui/`
- Icons: `lucide-vue-next`
- Component resolution via `unplugin-vue-components`

### Stale-Request Guards

Async composables that may receive out-of-order responses must guard state writes with `createLatestRequestGuard()` (from `src/shared/utils/latestRequest.ts`):

```ts
const guard = createLatestRequestGuard()
async function fetch() {
  const ticket = guard.begin()
  const data = await invoke(...)
  if (!ticket.isCurrent()) return   // stale response — drop it
  state.value = data
}
```

Apply this pattern in any composable whose user can retrigger the call before the previous one resolves. See `useSearchMatches.ts` + its `.test.ts` for the canonical pattern.

## Data Classification

Understanding the data lifecycle helps optimize caching and reduce unnecessary API calls.

### Static Data Ownership（权威方）

| 数据 | 权威方 | 持久化 | 前端职责 |
|------|--------|--------|----------|
| 英雄摘要（含 Jade `600xx`） | **Rust** `static_catalog` | `%AppData%/nidalee/static/{version}/` | 仅 IPC → `setChampionCatalog` |
| 召唤师技能 | **Rust** `static_catalog` | 同上 | 仅 IPC → `setSummonerSpellCatalog` |
| 游戏版本 | **Rust**（DDragon `versions.json`，与静态包同源） | `meta.json` | `useStaticCatalogMeta` / `dataStore.gameVersion` |
| 队列中文名 | **前端** CDragon | localStorage（按版本） | `useQueues` |
| 符文 UI（styles/perks） | **前端** CDragon | localStorage（按版本，无 TTL） | `useRuneData` / `useCommunityDragonPerksQuery` |
| 物品全表 | **前端** DDragon | localStorage（按版本） | `useItems`（首访，不预加载） |
| 单英雄详情/皮肤 | **前端** CDragon | Query 长缓存 | `useChampionDetails`（按需） |

**原则**：

1. 身份解析（分析 / 对局详情补名 / WS 名→id）只认 Rust 目录。
2. 前端禁止再直连 CDragon 拉英雄/技能全量。
3. 「不预加载」≠「不缓存」：物品等首访后仍按版本持久化；版本不变不重拉。
4. 失效只认游戏版本（Connected 时 `refresh_static_catalogs` + invalidate `['static']`）。

启动入口：`useBootstrapStaticData()`（meta + champions + spells + queues）。

### Static Data (版本化缓存)
**Definition**: Data that only changes when the LoL game version updates (typically every 2 weeks).

**Cache Strategy**: Use `['static', 'category', version]` as query key. Invalidated when game version changes.

**Query Key Pattern**: `['static', '<category>', version]`

| Data | Query Key | Source | Composable |
|------|-----------|--------|------------|
| 静态包元信息 | `['staticCatalogMeta']` | Rust IPC | `useStaticCatalogMeta()` |
| 英雄列表 | `['static', 'champions', version]` | Rust IPC（CDragon 落盘） | `useChampions()` |
| 召唤师技能 | `['static', 'summonerSpells', version]` | Rust IPC | `useSummonerSpells()` |
| 队列 | `['static', 'queues', version]` | CDragon + localStorage | `useQueues()` |
| 符文元数据 | `['static', 'communityDragonPerks', version]` | CDragon + localStorage | `useCommunityDragonPerksQuery()` |
| 物品 | `['static', 'items', version]` | DDragon + localStorage | `useItems()`（首访） |
| 英雄详情 | `['static', 'championDetails', version, id]` | CDragon | `useChampionDetails()` |

**Cache Config**: `staleTime: Infinity, gcTime: Infinity`（会话内）；跨启动由 Rust 磁盘 / FE `versionedCache` 负责。

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

### Dynamic Data (WS-主, HTTP-snapshot 兜底)
**Definition**: Data that changes frequently during gameplay.

**Cache Strategy**: WebSocket events drive UI updates in real time; HTTP snapshot fetches serve as a periodic health check (e.g. every 10s when the WS is quiet) and as the source of truth on reconnect.

The WebSocket supervisor (in `infrastructure/real_time/websocket/service.rs`) owns connection lifecycle and reconnection. The HTTP fallback (in the same module) feeds snapshots through the same reducer as WS events so the two paths share one state machine.

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

**WebSocket Subscription** (managed by supervisor):
```rust
// Always subscribed
"/lol-gameflow/v1/gameflow-phase"
"/lol-gameflow/v1/session"
"/lol-summoner/v1/current-summoner"

// Phase-specific subscriptions
"Lobby/Matchmaking/None" → "/lol-lobby/v2/lobby", "/lol-matchmaking/v1/search"
"ChampSelect" → "/lol-champ-select/v1/session"
```

The `WsEventHandler` (`infrastructure/real_time/websocket/event_handler.rs`) maintains per-generation cancellation handles for in-flight enrichment tasks. A late task must never republish data for an old generation — see `cancel_champ_select_analysis` / `cancel_in_game_recovery`.

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
- LCU WebSocket connection is owned by the **supervisor** (`infrastructure/real_time/websocket/service.rs`). It manages authentication recovery, subscription lifecycle, and reconnects with a small backoff.
- Commands: `start_lcu_ws`, `stop_lcu_ws` (registered in `lib.rs` as test/admin commands)
- Event handling in `event_handler.rs` — generation counters + `AbortHandle` ensure stale tasks cannot overwrite newer data.
- HTTP fallback snapshots are routed through `WsEventHandler::handle_snapshot` so they share the same reducer.

### Game Session Management
- Connection state: `infrastructure/game_session/connection/` — subscribes to WS supervisor state, does not perform its own process scanning or HTTP validation.
- Authentication: `infrastructure/game_session/auth/`
- Lobby: `infrastructure/game_session/lobby/`

### Match Analysis
- Player stats analyzer: `domains/analysis/` (evidence → pipeline → trait_strategies → services)
- The orchestration is exposed via the `analysis_contract` top-level facade
- Backend-heavy approach — analysis is done in Rust, frontend displays results via the match-analysis store
- Supports queue filtering (420=ranked solo, 440=ranked flex, etc.)

### Build Configuration
- Dev server (Vite): `http://localhost:1422`
- Dev build output: `dist/`
- Tauri dev URL matches frontend dev server
- Production bundle analysis: `pnpm build:analyze`

### Platform Support
- **Primary target**: Windows (`.msi` installer via WiX)
- **Secondary target**: macOS (`.dmg`) — workflow present, but full platform ownership is undecided (see `docs/PROJECT_STABILITY_ROADMAP.md` 阶段 8)
- Game path detection via Windows registry (Windows-only)

## Testing

### Frontend (Vitest)
- Config: `vitest.config.ts` (covers `src/**/*.test.ts`)
- Run: `pnpm test` (CI gate)
- Convention: pure-logic utilities and request-lifecycle composables ship with unit tests; Vue components do not require shallow render tests.
- The canonical pattern is `useSearchMatches.test.ts` + `latestRequest.test.ts` — read these before writing a new test.

### Rust
- Unit tests live alongside source (`#[cfg(test)] mod tests`)
- Integration tests live under `src-tauri/tests/` (e.g. `analysis_evidence.rs`, `analysis_orchestrator.rs`)
- Run: `cargo test --all-targets --locked`

## Common Tasks

### Adding a new Tauri command
1. Add a `#[tauri::command]` function in the relevant `infrastructure/<module>/commands.rs` (or `common/commands/<area>.rs` for cross-cutting commands)
2. Register it in `src-tauri/src/lib.rs` `invoke_handler!` macro under the appropriate section comment
3. Add permission to `src-tauri/capabilities/default.json` if it requires an FS/shell capability (most do not — backend-only commands bypass capabilities)
4. Frontend: `invoke('command_name', { args })`

### Adding a new frontend feature
1. Create directory under `src/features/<feature-name>/`
2. Add a main page `<Name>.vue` + optional `components/` / `composables/`
3. Add a route in `src/router/index.ts`
4. Components are auto-discovered for import; composables/stores are auto-imported if placed under `src/shared/composables/**` or `src/shared/stores/**`

### After modifying Rust types
```bash
pnpm types              # regenerate and verify
# or step by step:
node scripts/prepare-types.mjs
cd src-tauri && cargo test --all-targets --locked
cd .. && node scripts/sync-types.mjs
```

CI fails if `src/types/global.d.ts` changes are not committed alongside the Rust type changes.