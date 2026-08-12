import { describe, expect, it, vi } from 'vitest'
import { getLatestMatchId, runPostGameRefresh } from './postGameMatchRefresh'

function performance(gameId: number, gameCreation: number): MatchPerformance {
  return { gameId, gameCreation } as MatchPerformance
}

function result(...matches: MatchPerformance[]): MatchAnalysisResult {
  return {
    overallStats: { recentPerformance: matches } as PlayerMatchStats
  } as MatchAnalysisResult
}

describe('post-game match refresh', () => {
  it('uses game creation rather than array order to find the latest match', () => {
    const stats = result(performance(10, 100), performance(12, 300), performance(11, 200)).overallStats
    expect(getLatestMatchId(stats)).toBe(12)
  })

  it('retries until LCU exposes a different latest game id', async () => {
    const refresh = vi
      .fn<() => Promise<MatchAnalysisResult | null>>()
      .mockResolvedValueOnce(result(performance(10, 100)))
      .mockResolvedValueOnce(result(performance(11, 200)))
    const wait = vi.fn<(delayMs: number) => Promise<void>>().mockResolvedValue()

    await expect(
      runPostGameRefresh({
        baselineGameId: 10,
        refresh,
        wait,
        isCancelled: () => false,
        delaysMs: [0, 1_000]
      })
    ).resolves.toBe('updated')
    expect(refresh).toHaveBeenCalledTimes(2)
    expect(wait).toHaveBeenCalledWith(1_000)
  })

  it('stops before another request when the session is cancelled during a delay', async () => {
    let cancelled = false
    const refresh = vi.fn<() => Promise<MatchAnalysisResult | null>>().mockResolvedValue(result(performance(10, 100)))

    await expect(
      runPostGameRefresh({
        baselineGameId: 10,
        refresh,
        wait: async () => {
          cancelled = true
        },
        isCancelled: () => cancelled,
        delaysMs: [0, 1_000]
      })
    ).resolves.toBe('cancelled')
    expect(refresh).toHaveBeenCalledTimes(1)
  })
})
