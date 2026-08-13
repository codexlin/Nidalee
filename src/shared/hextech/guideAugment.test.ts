import { describe, expect, it } from 'vitest'
import { groupAugmentsByRarity, groupAugmentsByTier, normalizeRarityKey, filterRecommendedAugments, type HextechGuideAugment } from './guideAugment'

function augment(partial: Partial<HextechGuideAugment> & Pick<HextechGuideAugment, 'id' | 'name'>): HextechGuideAugment {
  return {
    iconUrl: '',
    rarityName: '',
    rarityDisplayName: '',
    winRate: 0,
    pickRate: 0,
    games: null,
    tier: null,
    ...partial
  }
}

describe('normalizeRarityKey', () => {
  it('maps english and chinese names', () => {
    expect(normalizeRarityKey('prismatic', '')).toBe('prismatic')
    expect(normalizeRarityKey('', '棱彩')).toBe('prismatic')
    expect(normalizeRarityKey('gold', '黄金')).toBe('gold')
    expect(normalizeRarityKey('silver', '白银')).toBe('silver')
  })
})

describe('groupAugmentsByRarity', () => {
  it('orders prismatic then gold then silver and sorts by win rate', () => {
    const groups = groupAugmentsByRarity([
      augment({ id: 1, name: '银低', rarityName: 'silver', winRate: 0.4 }),
      augment({ id: 2, name: '彩高', rarityName: 'prismatic', winRate: 0.62 }),
      augment({ id: 3, name: '金高', rarityName: 'gold', winRate: 0.58 }),
      augment({ id: 4, name: '银高', rarityName: 'silver', winRate: 0.51 }),
      augment({ id: 5, name: '彩低', rarityDisplayName: '棱彩', winRate: 0.5 })
    ])

    expect(groups.map((group) => group.key)).toEqual(['prismatic', 'gold', 'silver'])
    expect(groups[0]?.items.map((item) => item.name)).toEqual(['彩高', '彩低'])
    expect(groups[2]?.items.map((item) => item.name)).toEqual(['银高', '银低'])
    expect(groups[0]?.label).toBe('棱彩')
  })
})

describe('filterRecommendedAugments', () => {
  it('drops losing augments including T1 below 50%', () => {
    const kept = filterRecommendedAugments([
      augment({ id: 1, name: 'T1低', winRate: 0.47, games: 20, tier: 1 }),
      augment({ id: 2, name: '亏损', winRate: 0.46, games: 20, tier: 2 }),
      augment({ id: 3, name: '高', winRate: 0.6, games: 20, tier: 2 })
    ])
    expect(kept.map((item) => item.name)).toEqual(['高'])
  })

  it('keeps every sampled augment at or above 50%', () => {
    const kept = filterRecommendedAugments([
      augment({ id: 1, name: '顶', winRate: 0.62, games: 20 }),
      augment({ id: 2, name: '接近', winRate: 0.57, games: 20 }),
      augment({ id: 3, name: '偏弱', winRate: 0.54, games: 20 })
    ])
    expect(kept.map((item) => item.name)).toEqual(['顶', '接近', '偏弱'])
  })

  it('drops augments with no sampled win rate even if they are T1', () => {
    const kept = filterRecommendedAugments([
      augment({ id: 1, name: '无样本', winRate: 0, games: null, tier: 1 }),
      augment({ id: 2, name: '有样本', winRate: 0.51, games: 12, tier: 1 })
    ])
    expect(kept.map((item) => item.name)).toEqual(['有样本'])
  })

  it('hides a band when every sampled win rate is below 50%', () => {
    const kept = filterRecommendedAugments([
      augment({ id: 1, name: 'T2顶', winRate: 0.478, games: 30, tier: 2 }),
      augment({ id: 2, name: 'T2近', winRate: 0.47, games: 30, tier: 2 }),
      augment({ id: 3, name: 'T2弱', winRate: 0.4, games: 30, tier: 2 }),
      augment({ id: 4, name: '空', winRate: 0, games: null, tier: 2 })
    ])
    expect(kept).toEqual([])
  })
})

describe('groupAugmentsByTier', () => {
  it('splits T1 T2 T3 in order', () => {
    const bands = groupAugmentsByTier([
      augment({ id: 1, name: '二', winRate: 0.55, tier: 2 }),
      augment({ id: 2, name: '一高', winRate: 0.61, tier: 1 }),
      augment({ id: 3, name: '三', winRate: 0.52, tier: 3 }),
      augment({ id: 4, name: '一低', winRate: 0.58, tier: 1 })
    ])
    expect(bands.map((band) => band.label)).toEqual(['T1', 'T2', 'T3'])
    expect(bands[0]?.items.map((item) => item.name)).toEqual(['一高', '一低'])
  })

  it('keeps T4 as its own band', () => {
    const bands = groupAugmentsByTier([
      augment({ id: 1, name: '四', winRate: 0.44, tier: 4 }),
      augment({ id: 2, name: '一', winRate: 0.6, tier: 1 })
    ])
    expect(bands.map((band) => band.label)).toEqual(['T1', 'T4'])
  })
})
