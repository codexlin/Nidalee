import { describe, expect, it } from 'vitest'
import { buildRankedPositionSnapshot, formatRankLabel } from './playerPresentation'

describe('formatRankLabel', () => {
  it('shows localized tier, division and league points', () => {
    expect(formatRankLabel('EMERALD', 'III', 42)).toBe('流光翡翠 III · 42 LP')
  })

  it('omits unavailable rank details', () => {
    expect(formatRankLabel('MASTER', 'NA', null)).toBe('超凡大师')
    expect(formatRankLabel(null, null, null)).toBe('')
  })
})

describe('buildRankedPositionSnapshot', () => {
  const jungle = {
    position: 'JUNGLE',
    sample: { games: 12, wins: 7, winRate: 58.3, avgKda: 3.4 }
  }
  const support = {
    position: 'SUPPORT',
    sample: { games: 5, wins: 2, winRate: 40, avgKda: 2.8 }
  }

  it('uses the backend primary position and marks a same-role game', () => {
    const result = buildRankedPositionSnapshot({
      primaryPosition: 'JUNGLE',
      positions: [support, jungle],
      currentPosition: { ...jungle, isPrimary: true }
    })

    expect(result.primary?.position).toBe('JUNGLE')
    expect(result.currentKind).toBe('same')
  })

  it('keeps an off-role position explicit instead of changing the primary position', () => {
    const result = buildRankedPositionSnapshot({
      primaryPosition: 'JUNGLE',
      positions: [jungle, support],
      currentPosition: { ...support, isPrimary: false }
    })

    expect(result.primary?.position).toBe('JUNGLE')
    expect(result.current?.position).toBe('SUPPORT')
    expect(result.currentKind).toBe('different')
  })

  it('falls back to the largest position sample when primary position is absent', () => {
    const result = buildRankedPositionSnapshot({
      positions: [support, jungle]
    })

    expect(result.primary?.position).toBe('JUNGLE')
    expect(result.currentKind).toBe('unknown')
  })
})
