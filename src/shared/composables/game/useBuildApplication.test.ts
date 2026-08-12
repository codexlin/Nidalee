import { describe, expect, it, vi } from 'vitest'
import { runeSnapshotFromOpgg } from './useBuildApplication'

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
      { championId: 59, position: 'JUNGLE', region: 'kr', mode: 'ranked', tier: 'diamond_plus' }
    )

    expect(snapshot.championName).toBe('champion-59')
    expect(snapshot.position).toBe('JUNGLE')
    expect(snapshot.selection.selectedPerkIds).toHaveLength(9)
  })
})
