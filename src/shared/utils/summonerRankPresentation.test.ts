import { describe, expect, it } from 'vitest'
import { buildSummonerRankPresentation } from './summonerRankPresentation'

describe('buildSummonerRankPresentation', () => {
  it('normalizes ranked data and calculates the win rate', () => {
    expect(
      buildSummonerRankPresentation({
        tier: 'EMERALD',
        division: 'IV',
        leaguePoints: 56,
        wins: 6,
        losses: 4
      })
    ).toEqual({
      tier: 'EMERALD',
      rank: 'IV',
      leaguePoints: 56,
      winRate: 60
    })
  })

  it('returns a stable unranked presentation for missing data', () => {
    expect(buildSummonerRankPresentation({})).toEqual({
      tier: 'UNRANKED',
      rank: '',
      leaguePoints: 0,
      winRate: 0
    })
  })
})
