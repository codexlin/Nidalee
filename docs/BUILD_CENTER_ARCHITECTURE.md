# Build Center Architecture

The Build Center owns both externally recommended builds and user-owned presets, while keeping their lifecycles separate.

## Boundaries

| Layer | Responsibility | Must not do |
| --- | --- | --- |
| Query composables | Fetch and cache provider data | Persist user presets or write LCU state |
| Build Center UI | Browse recommendations and edit owned presets | Implement matching or duplicate apply logic |
| `BuildPresetStore` | Persist presets and auto-build policy | Fetch recommendations or call LCU |
| `useBuildApplication` | Validate and apply a resolved rune selection | Decide which preset wins |
| `useAutoBuild` | Resolve one source after champion lock | Implement provider-specific LCU writes |
| Rust `apply_rune_selection` | Validate IPC input and update the current rune page | Fetch OP.GG or select a preset |

## State ownership

- Provider responses are server state and remain in Vue Query.
- Saved presets and auto-build policy are user state and remain in Pinia + Tauri Store.
- Champion-select state remains in the game/session stores and is not copied into the preset store.
- Application progress is operation-local state in `useBuildApplication`.

## Dependency direction

```text
UI → query composable
UI → BuildPresetStore
UI / useAutoBuild → useBuildApplication → Tauri command → LCU service
```

No lower layer imports UI, no Store invokes provider queries, and no provider module owns LCU mutation.

## Future build components

`BuildPreset.components` is the extension boundary. Items, summoner spells, and skill orders must be added there and applied by the same resolved-preset pipeline. A new component may have its own validator and executor, but not its own competing preset store, matching algorithm, or provider-specific IPC command.
