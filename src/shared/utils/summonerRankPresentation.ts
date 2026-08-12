export interface SummonerRankPresentation {
  tier: string
  rank: string
  leaguePoints: number
  winRate: number
}

interface BuildSummonerRankPresentationOptions {
  tier?: string | null
  division?: string | null
  leaguePoints?: number | null
  wins?: number | null
  losses?: number | null
}

export function buildSummonerRankPresentation({
  tier,
  division,
  leaguePoints,
  wins,
  losses
}: BuildSummonerRankPresentationOptions): SummonerRankPresentation {
  const resolvedWins = wins ?? 0
  const resolvedLosses = losses ?? 0
  const totalGames = resolvedWins + resolvedLosses

  return {
    tier: tier || 'UNRANKED',
    rank: division || '',
    leaguePoints: leaguePoints ?? 0,
    winRate: totalGames > 0 ? Math.round((resolvedWins / totalGames) * 100) : 0
  }
}
