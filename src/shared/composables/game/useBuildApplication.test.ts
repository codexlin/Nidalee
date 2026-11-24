import { describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { runeSelectionFromOpgg, runeSnapshotFromOpgg, useBuildApplication } from './useBuildApplication'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@/shared/stores/features/buildPresetStore', () => ({ useBuildPresetStore: () => ({}) }))
vi.mock('@/lib', () => ({ getChampionName: (id: number) => `champion-${id}` }))

describe('runeSnapshotFromOpgg', () => {
  it('normalizes a recommendation into the shared rune shape', () => {
    const snapshot = runeSnapshotFromOpgg(
      {
        primaryId: 8000,
        secondaryId: 8200,
        perks: [8005, 9111, 9104, 8014, 8233, 8236, 5005, 5008, 5001],
        win: 10,
        play: 20,
        pickRate: 0.4
      },
      {
        target: { championId: 59, championName: '', scenario: 'ranked-jungle' },
        region: 'kr',
        mode: 'ranked',
        tier: 'diamond_plus'
      }
    )

    expect(snapshot.target.championName).toBe('champion-59')
    expect(snapshot.target.scenario).toBe('ranked-jungle')
    expect(snapshot.selection.selectedPerkIds).toHaveLength(9)
  })

  it('applies by catalog ID even when the provider omits the champion name, and surfaces IPC errors', async () => {
    const perks = [8230, 8226, 8210, 8237, 8242, 8444, 5008, 5008, 5011]
    const perk = { primaryId: 8200, secondaryId: 8400, perks, win: 10, play: 20, pickRate: 0.4 }
    const application = useBuildApplication()
    vi.mocked(invoke).mockResolvedValueOnce('ok').mockRejectedValueOnce('更新专属符文页失败')
    await expect(application.applyRecommendation(3, perk)).resolves.toBe('ok')
    expect(invoke).toHaveBeenCalledWith('apply_rune_selection', {
      pageLabel: 'champion-3',
      selection: { primaryStyleId: 8200, subStyleId: 8400, selectedPerkIds: perks }
    })
    await expect(application.applyRecommendation(3, perk)).rejects.toThrow('更新专属符文页失败')
  })

  it('preserves both adaptive force shards in the Galio mid recommendation', () => {
    const perks = [8230, 8226, 8210, 8237, 8242, 8444, 5008, 5008, 5011]
    const selection = runeSelectionFromOpgg({
      primaryId: 8200,
      secondaryId: 8400,
      perks,
      win: 10,
      play: 20,
      pickRate: 0.4
    })
    expect(selection.selectedPerkIds).toEqual(perks)
  })

  it('converts provider runes without requiring an auto-build target', () => {
    const selection = runeSelectionFromOpgg({
      primaryId: 8000,
      secondaryId: 8200,
      perks: [8005, 9111, 9104, 8014, 8233, 8236, 5005, 5008, 5001],
      win: 10,
      play: 20,
      pickRate: 0.4
    })

    expect(selection.selectedPerkIds).toHaveLength(9)
  })
})
