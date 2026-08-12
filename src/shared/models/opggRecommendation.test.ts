import { describe, expect, it } from 'vitest'
import { selectMainOpggPosition } from './opggRecommendation'

function tierList(championId: number, positions: Array<{ name: string; play: number }>): OpggTierList {
  return {
    meta: { version: '1', region: 'kr', mode: 'ranked', tier: 'diamond_plus' },
    data: [
      {
        championId,
        averageStats: {
          play: 0,
          winRate: 0,
          pickRate: 0,
          banRate: 0,
          kda: 0,
          tier: 0,
          rank: 0,
          firstPlace: null,
          totalPlace: null
        },
        positions: positions.map(({ name, play }) => ({
          name,
          stats: {
            play,
            winRate: 0,
            pickRate: 0,
            banRate: 0,
            kda: 0,
            tier: 0,
            rank: 0,
            firstPlace: null,
            totalPlace: null
          },
          counters: []
        })),
        roles: []
      }
    ]
  }
}

describe('selectMainOpggPosition', () => {
  it('selects the valid position with the largest sample', () => {
    expect(
      selectMainOpggPosition(
        tierList(59, [
          { name: 'TOP', play: 20 },
          { name: 'jungle', play: 80 },
          { name: 'MIDDLE', play: 50 }
        ]),
        59
      )
    ).toBe('JUNGLE')
  })

  it('keeps provider order when samples tie', () => {
    expect(
      selectMainOpggPosition(
        tierList(59, [
          { name: 'TOP', play: 20 },
          { name: 'JUNGLE', play: 20 }
        ]),
        59
      )
    ).toBe('TOP')
  })

  it('returns null for a missing champion or no valid position', () => {
    const data = tierList(59, [{ name: 'UNKNOWN', play: 100 }])
    expect(selectMainOpggPosition(data, 1)).toBeNull()
    expect(selectMainOpggPosition(data, 59)).toBeNull()
  })

  it('does not guess a main position from zero-sample placeholders', () => {
    expect(
      selectMainOpggPosition(
        tierList(59, [
          { name: 'TOP', play: 0 },
          { name: 'JUNGLE', play: 0 }
        ]),
        59
      )
    ).toBeNull()
  })
})
