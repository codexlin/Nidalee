import { describe, expect, it } from 'vitest'
import {
  createPresetFromRecommendation,
  normalizeBuildPosition,
  selectMatchingPreset,
  validateBuildApplicability,
  validateRuneSelection,
  type BuildPreset,
  type RecommendedRuneSnapshot
} from './buildPreset'

const selection = {
  primaryStyleId: 8000,
  subStyleId: 8200,
  selectedPerkIds: [8005, 9111, 9104, 8014, 8233, 8236, 5005, 5008, 5001]
}

function preset(overrides: Partial<BuildPreset> & Pick<BuildPreset, 'id'>): BuildPreset {
  const { id, ...changes } = overrides
  return {
    id,
    name: id,
    applicability: { scope: 'champion-all', championId: 59, championName: '德玛西亚皇子', position: null },
    components: { runes: selection },
    source: { kind: 'custom' },
    isDefault: false,
    createdAt: 1,
    updatedAt: 1,
    usageCount: 0,
    ...changes
  }
}

describe('buildPreset', () => {
  it('rejects incomplete and duplicate rune selections', () => {
    expect(validateRuneSelection({ ...selection, selectedPerkIds: selection.selectedPerkIds.slice(0, 8) })).toContain(
      '9'
    )
    expect(
      validateRuneSelection({ ...selection, selectedPerkIds: [...selection.selectedPerkIds.slice(0, 8), 8005] })
    ).toContain('重复')
    expect(validateRuneSelection(selection)).toBeNull()
  })

  it('requires applicability fields to agree with the selected scope', () => {
    expect(
      validateBuildApplicability({
        scope: 'champion-position',
        championId: 59,
        championName: '德玛西亚皇子',
        position: null
      })
    ).not.toBeNull()
    expect(
      validateBuildApplicability({
        scope: 'position-all',
        championId: null,
        championName: null,
        position: 'JUNGLE'
      })
    ).toBeNull()
  })

  it('matches exact champion and position before broader scopes', () => {
    const positionWide = preset({
      id: 'position',
      applicability: { scope: 'position-all', championId: null, championName: null, position: 'JUNGLE' }
    })
    const championWide = preset({ id: 'champion' })
    const exact = preset({
      id: 'exact',
      applicability: { scope: 'champion-position', championId: 59, championName: '德玛西亚皇子', position: 'JUNGLE' }
    })

    expect(selectMatchingPreset([positionWide, championWide, exact], 59, 'jungle')?.id).toBe('exact')
    expect(normalizeBuildPosition('middle')).toBe('MID')
  })

  it('uses default then latest update as deterministic tie breakers', () => {
    const olderDefault = preset({ id: 'default', isDefault: true, updatedAt: 1 })
    const newer = preset({ id: 'newer', updatedAt: 9 })
    expect(selectMatchingPreset([newer, olderDefault], 59, 'TOP')?.id).toBe('default')
  })

  it('creates an owned snapshot without sharing the recommendation array', () => {
    const snapshot: RecommendedRuneSnapshot = {
      championId: 59,
      championName: '德玛西亚皇子',
      position: 'JUNGLE',
      selection,
      source: {
        kind: 'opgg',
        provider: 'opgg',
        region: 'kr',
        mode: 'ranked',
        tier: 'diamond_plus',
        capturedAt: 10
      }
    }
    const created = createPresetFromRecommendation(snapshot)
    snapshot.selection.selectedPerkIds[0] = 1
    expect(created.components.runes.selectedPerkIds[0]).toBe(8005)
    expect(created.applicability.scope).toBe('champion-position')
  })
})
