import { describe, expect, it } from 'vitest'
import { mapHextechTierListToOpgg } from './useHextechData'

describe('mapHextechTierListToOpgg', () => {
  it('maps tier list into OpggTierList shape for shared panel', () => {
    const mapped = mapHextechTierListToOpgg({
      dataVersion: '16.15.6',
      gamePatch: '16.15',
      region: 'zh-CN',
      data: [
        {
          championId: 157,
          name: '亚索',
          alias: 'Yasuo',
          iconUrl: 'https://example/157.png',
          roles: ['fighter'],
          winRate: 0.55,
          pickRate: 0.1,
          tier: 2,
          rank: 1
        }
      ]
    })

    expect(mapped.meta.mode).toBe('hextech')
    expect(mapped.data[0]?.championId).toBe(157)
    expect(mapped.data[0]?.roles[0]?.name).toBe('FIGHTER')
    expect(mapped.data[0]?.averageStats.winRate).toBe(0.55)
  })
})
