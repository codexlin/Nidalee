import { expect, it } from 'vitest'
import type { BuildPreset } from '@/shared/models/buildPreset'
import { groupBuildPresets } from './groupBuildPresets'

function preset(id: string, championId: number, createdAt: number): BuildPreset {
  return {
    id,
    name: '中路推荐',
    target: { championId, championName: String(championId), scenario: 'ranked-mid' },
    components: { runes: { primaryStyleId: 8200, subStyleId: 8300, selectedPerkIds: [] } },
    source: { kind: 'custom' },
    autoUse: false,
    createdAt,
    updatedAt: createdAt,
    usageCount: 0
  }
}

it('keeps cards and heroes in place when the default switches between identically named presets', () => {
  const saved = [preset('a', 134, 1), preset('b', 134, 2), preset('c', 112, 3)]
  saved[1].autoUse = true
  const before = groupBuildPresets(saved)
  const after = groupBuildPresets(
    saved.map((p) => ({ ...p, autoUse: p.id === 'a', updatedAt: p.id === 'a' ? 10 : p.updatedAt }))
  )
  expect(after.map((g) => g.championId)).toEqual(before.map((g) => g.championId))
  expect(after[1].presets.map((p) => p.id)).toEqual(before[1].presets.map((p) => p.id))
  expect(after.map((g) => g.autoCount)).toEqual([0, 1])
  expect(after[1].presets.find((p) => p.autoUse)?.id).toBe('a')
})
