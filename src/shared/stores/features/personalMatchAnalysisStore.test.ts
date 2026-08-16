import { beforeEach, describe, expect, it } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { usePersonalMatchAnalysisStore } from './personalMatchAnalysisStore'

const mixedRanked = { category: 'ranked', rankedScope: 'mixed' } as const
const flexRanked = { category: 'ranked', rankedScope: 'flex' } as const

describe('personalMatchAnalysisStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('only exposes a result to the account and scope that produced it', () => {
    const store = usePersonalMatchAnalysisStore()
    const result = { displayGames: 20 } as MatchAnalysisResult

    store.setResult(result, ' player-a ', mixedRanked)

    expect(store.resultRevision).toBe(1)
    expect(store.hasResultFor('player-a', mixedRanked)).toBe(true)
    expect(store.hasResultFor('player-a', flexRanked)).toBe(false)
    expect(store.hasResultFor('player-b', mixedRanked)).toBe(false)
  })

  it('clears the result and its account/scope identity together', () => {
    const store = usePersonalMatchAnalysisStore()
    store.setResult({ displayGames: 20 } as MatchAnalysisResult, 'player-a', mixedRanked)

    store.clear()

    expect(store.resultRevision).toBe(2)
    expect(store.result).toBeNull()
    expect(store.lastPuuid).toBeNull()
    expect(store.lastScopeKey).toBeNull()
    expect(store.hasResultFor('player-a', mixedRanked)).toBe(false)
  })
})
