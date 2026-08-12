import { describe, expect, it } from 'vitest'
import { formatRankLabel } from './playerPresentation'

describe('formatRankLabel', () => {
  it('shows localized tier, division and league points', () => {
    expect(formatRankLabel('EMERALD', 'III', 42)).toBe('流光翡翠 III · 42 LP')
  })

  it('omits unavailable rank details', () => {
    expect(formatRankLabel('MASTER', 'NA', null)).toBe('超凡大师')
    expect(formatRankLabel(null, null, null)).toBe('')
  })
})
