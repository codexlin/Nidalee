import { describe, expect, it } from 'vitest'
import {
  normalizePerformanceCategory,
  normalizeRankedScope,
  performanceScopeKey,
  performanceScopeToAnalysis
} from './performanceScope'

describe('performanceScope', () => {
  it('将三个排位范围映射为唯一后端模式', () => {
    expect(performanceScopeToAnalysis({ category: 'ranked', rankedScope: 'mixed' })).toEqual({ mode: 'mixedRanked' })
    expect(performanceScopeToAnalysis({ category: 'ranked', rankedScope: 'solo' })).toEqual({
      mode: 'soloRanked',
      queueId: 420
    })
    expect(performanceScopeToAnalysis({ category: 'ranked', rankedScope: 'flex' })).toEqual({
      mode: 'flexRanked',
      queueId: 440
    })
  })

  it('其他模式只有一个稳定键且由后端排除排位', () => {
    const scope = { category: 'other', rankedScope: 'flex' } as const
    expect(performanceScopeKey(scope)).toBe('other')
    expect(performanceScopeToAnalysis(scope)).toEqual({ mode: 'normals' })
  })

  it('持久化值损坏时回到安全默认值', () => {
    expect(normalizePerformanceCategory('invalid')).toBe('ranked')
    expect(normalizeRankedScope('invalid')).toBe('mixed')
  })
})
