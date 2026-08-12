import { describe, expect, it } from 'vitest'
import {
  buildTargetKey,
  createPresetFromRecommendation,
  normalizeBuildPosition,
  rankedPositionFromScenario,
  rankedScenarioFromPosition,
  sameBuildTarget,
  scenarioLabel,
  selectAutoPreset,
  validateBuildTarget,
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
    target: { championId: 59, championName: '德玛西亚皇子', scenario: 'ranked-jungle' },
    components: { runes: selection },
    source: { kind: 'custom' },
    autoUse: false,
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

  it('requires a complete champion and supported scenario', () => {
    expect(validateBuildTarget({ championId: 59, championName: '德玛西亚皇子', scenario: 'ranked-jungle' })).toBeNull()
    expect(
      validateBuildTarget({ championId: 0, championName: '德玛西亚皇子', scenario: 'ranked-jungle' })
    ).not.toBeNull()
    expect(validateBuildTarget({ championId: 59, championName: '', scenario: 'ranked-jungle' })).not.toBeNull()
  })

  it('maps ranked positions and scenario labels deterministically', () => {
    expect(normalizeBuildPosition('middle')).toBe('MID')
    expect(rankedScenarioFromPosition('utility')).toBe('ranked-support')
    expect(rankedPositionFromScenario('ranked-adc')).toBe('ADC')
    expect(rankedPositionFromScenario('aram')).toBeNull()
    expect(scenarioLabel('ranked-jungle')).toBe('排位打野')
    expect(scenarioLabel('normal-sr')).toBe('普通峡谷')
  })

  it('uses champion and scenario as the exact target key', () => {
    const jungle = { championId: 59, championName: '德玛西亚皇子', scenario: 'ranked-jungle' as const }
    const jungleRenamed = { ...jungle, championName: 'Jarvan IV' }
    const normal = { ...jungle, scenario: 'normal-sr' as const }

    expect(buildTargetKey(jungle)).toBe('59:ranked-jungle')
    expect(sameBuildTarget(jungle, jungleRenamed)).toBe(true)
    expect(sameBuildTarget(jungle, normal)).toBe(false)
  })

  it('selects only the auto-enabled preset for an exact target', () => {
    const manual = preset({ id: 'manual' })
    const auto = preset({ id: 'auto', autoUse: true })
    const otherScenario = preset({
      id: 'normal',
      target: { championId: 59, championName: '德玛西亚皇子', scenario: 'normal-sr' },
      autoUse: true
    })

    expect(selectAutoPreset([manual, otherScenario, auto], 59, 'ranked-jungle')?.id).toBe('auto')
    expect(selectAutoPreset([manual], 59, 'ranked-jungle')).toBeNull()
    expect(selectAutoPreset([auto], 59, 'normal-sr')).toBeNull()
    expect(selectAutoPreset([auto], 64, 'ranked-jungle')).toBeNull()
  })

  it('creates an owned manual snapshot without sharing nested data', () => {
    const snapshot: RecommendedRuneSnapshot = {
      target: { championId: 59, championName: '德玛西亚皇子', scenario: 'ranked-jungle' },
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
    snapshot.target.championName = 'changed'

    expect(created.components.runes.selectedPerkIds[0]).toBe(8005)
    expect(created.target.championName).toBe('德玛西亚皇子')
    expect(created.target.scenario).toBe('ranked-jungle')
    expect(created.autoUse).toBe(false)
  })
})
