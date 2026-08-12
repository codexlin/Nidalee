import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import type { BuildPreset } from '@/shared/models/buildPreset'

const persisted = vi.hoisted(() => ({
  value: null as unknown,
  failSave: false
}))

vi.mock('@tauri-apps/plugin-store', () => ({
  load: vi.fn(async () => ({
    get: vi.fn(async () => persisted.value),
    set: vi.fn(async (_key: string, value: unknown) => {
      persisted.value = value
    }),
    delete: vi.fn(async () => {
      const existed = persisted.value !== null && persisted.value !== undefined
      persisted.value = null
      return existed
    }),
    save: vi.fn(async () => {
      if (persisted.failSave) throw new Error('disk unavailable')
    })
  }))
}))

import { useBuildPresetStore } from './buildPresetStore'

const selection = {
  primaryStyleId: 8000,
  subStyleId: 8200,
  selectedPerkIds: [8005, 9111, 9104, 8014, 8233, 8236, 5005, 5008, 5001]
}

const alternativeSelection = {
  ...selection,
  selectedPerkIds: [...selection.selectedPerkIds.slice(0, -1), 5002]
}

function preset(id: string, overrides: Partial<BuildPreset> = {}): BuildPreset {
  return {
    id,
    name: id,
    target: { championId: 59, championName: '德玛西亚皇子', scenario: 'ranked-jungle' },
    components: { runes: selection },
    source: { kind: 'custom' },
    autoUse: false,
    createdAt: 1,
    updatedAt: 1,
    usageCount: 0,
    ...overrides
  }
}

describe('buildPresetStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    persisted.value = null
    persisted.failSave = false
  })

  it('keeps only one automatic preset per champion and scenario', async () => {
    const store = useBuildPresetStore()
    await store.addPreset(preset('first', { autoUse: true }))
    await store.addPreset(preset('second', { components: { runes: alternativeSelection }, autoUse: true }))

    expect(store.presets.find((item) => item.id === 'first')?.autoUse).toBe(false)
    expect(store.presets.find((item) => item.id === 'second')?.autoUse).toBe(true)
    expect(store.findAutoPreset(59, 'ranked-jungle')?.id).toBe('second')
  })

  it('does not let different scenarios compete', async () => {
    const store = useBuildPresetStore()
    await store.addPreset(preset('ranked', { autoUse: true }))
    await store.addPreset(
      preset('normal', {
        target: { championId: 59, championName: '德玛西亚皇子', scenario: 'normal-sr' },
        autoUse: true
      })
    )

    expect(store.findAutoPreset(59, 'ranked-jungle')?.id).toBe('ranked')
    expect(store.findAutoPreset(59, 'normal-sr')?.id).toBe('normal')
  })

  it('rejects duplicate target and rune selections', async () => {
    const store = useBuildPresetStore()
    await store.addPreset(preset('first'))

    await expect(store.addPreset(preset('duplicate'))).rejects.toThrow('相同英雄、场景和符文的方案已存在')
    expect(store.presets).toHaveLength(1)
  })

  it('reuses an equivalent saved recommendation instead of adding a duplicate', async () => {
    const store = useBuildPresetStore()
    const snapshot = {
      target: { championId: 59, championName: '德玛西亚皇子', scenario: 'ranked-jungle' as const },
      selection,
      source: {
        kind: 'opgg' as const,
        provider: 'opgg' as const,
        region: 'kr',
        mode: 'ranked',
        tier: 'diamond_plus',
        capturedAt: 1
      }
    }

    const first = await store.saveRecommendation(snapshot)
    const second = await store.saveRecommendation({ ...snapshot, source: { ...snapshot.source, capturedAt: 2 } })

    expect(second.id).toBe(first.id)
    expect(store.presets).toHaveLength(1)
  })

  it('turns a manual alternative into the sole automatic preset atomically', async () => {
    const store = useBuildPresetStore()
    await store.addPreset(preset('current', { autoUse: true }))
    await store.addPreset(preset('alternative', { components: { runes: alternativeSelection } }))
    await store.setAutoUse('alternative', true)

    expect(store.presets.find((item) => item.id === 'current')?.autoUse).toBe(false)
    expect(store.findAutoPreset(59, 'ranked-jungle')?.id).toBe('alternative')
  })

  it('exports v2 and imports copies as manual alternatives', async () => {
    const store = useBuildPresetStore()
    await store.addPreset(preset('automatic', { autoUse: true }))
    const exported = store.exportPresets()
    const importFile = JSON.parse(exported)
    importFile.presets[0].components.runes = alternativeSelection

    expect(JSON.parse(exported).version).toBe(2)
    expect(await store.importPresets(JSON.stringify(importFile))).toBe(1)
    expect(store.presets.filter((item) => item.autoUse)).toHaveLength(1)
    expect(store.presets.find((item) => item.source.kind === 'import')?.autoUse).toBe(false)
    await expect(store.importPresets(JSON.stringify({ version: 1, presets: [] }))).rejects.toThrow('不支持')
  })

  it('does not commit in-memory changes when persistence fails', async () => {
    const store = useBuildPresetStore()
    await store.loadFromStore()
    persisted.failSave = true

    await expect(store.addPreset(preset('lost'))).rejects.toThrow('disk unavailable')
    expect(store.presets).toHaveLength(0)
    expect(persisted.value).toBeNull()
  })

  it('restores the plugin cache when updating persisted state fails', async () => {
    const existingState = {
      version: 2,
      presets: [preset('existing')],
      autoBuild: { enabled: false, opggTier: 'diamond_plus', showToast: true }
    }
    persisted.value = existingState
    const store = useBuildPresetStore()
    await store.loadFromStore()
    persisted.failSave = true

    await expect(store.setAutoUse('existing', true)).rejects.toThrow('disk unavailable')

    expect(store.presets[0]?.autoUse).toBe(false)
    expect(persisted.value).toEqual(existingState)
  })

  it('rejects persisted states with competing automatic presets', async () => {
    persisted.value = {
      version: 2,
      presets: [preset('first', { autoUse: true }), preset('second', { autoUse: true })],
      autoBuild: { enabled: true, opggTier: 'diamond_plus', showToast: true }
    }
    const store = useBuildPresetStore()

    await expect(store.loadFromStore()).rejects.toThrow('只能有一套自动方案')
    expect(store.isLoaded).toBe(false)
  })
})
