export const POST_GAME_REFRESH_DELAYS_MS = [0, 1_000, 2_000, 3_000, 4_000] as const

export type PostGameRefreshOutcome = 'updated' | 'exhausted' | 'cancelled'

interface RunPostGameRefreshOptions {
  baselineGameId: number | null
  refresh: () => Promise<MatchAnalysisResult | null>
  wait: (delayMs: number) => Promise<void>
  isCancelled: () => boolean
  delaysMs?: readonly number[]
}

export function getLatestMatchId(stats: PlayerMatchStats | null | undefined): number | null {
  const latest = stats?.recentPerformance.reduce<MatchPerformance | null>((current, match) => {
    if (match.gameId === undefined) return current
    if (!current) return match
    return (match.gameCreation ?? 0) > (current.gameCreation ?? 0) ? match : current
  }, null)

  return latest?.gameId ?? null
}

export async function runPostGameRefresh({
  baselineGameId,
  refresh,
  wait,
  isCancelled,
  delaysMs = POST_GAME_REFRESH_DELAYS_MS
}: RunPostGameRefreshOptions): Promise<PostGameRefreshOutcome> {
  for (const delayMs of delaysMs) {
    if (isCancelled()) return 'cancelled'
    if (delayMs > 0) await wait(delayMs)
    if (isCancelled()) return 'cancelled'

    const result = await refresh()
    if (isCancelled()) return 'cancelled'

    const latestGameId = getLatestMatchId(result?.overallStats)
    if (latestGameId !== null && (baselineGameId === null || latestGameId !== baselineGameId)) {
      return 'updated'
    }
  }

  return 'exhausted'
}
