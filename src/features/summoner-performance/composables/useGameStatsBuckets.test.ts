import { describe, expect, it } from 'vitest'
import { nextTick, reactive } from 'vue'
import { useGameStatsBuckets, type GameStatsBucketProps } from './useGameStatsBuckets'

function stats(totalGames: number, recent: MatchPerformance[] = []): PlayerMatchStats {
  return {
    totalGames,
    wins: Math.ceil(totalGames / 2),
    losses: Math.floor(totalGames / 2),
    winRate: totalGames ? 50 : 0,
    avgKills: 0,
    avgDeaths: 0,
    avgAssists: 0,
    avgKda: 0,
    todayGames: 0,
    todayWins: 0,
    dpm: 0,
    cspm: 0,
    vspm: 0,
    traits: [],
    favoriteChampions: [],
    recentPerformance: recent
  } as PlayerMatchStats
}

function game(id: number): MatchPerformance {
  return { gameId: id, win: true, championId: 1, kills: 1, deaths: 1, assists: 1 } as MatchPerformance
}

describe('summoner performance projection', () => {
  it('直接展示后端已按范围筛选的一份统计', () => {
    const props = reactive<GameStatsBucketProps>({
      matchStatistics: stats(2, [game(1), game(2)]),
      scope: { category: 'ranked', rankedScope: 'mixed' }
    })

    const result = useGameStatsBuckets(props)

    expect(result.bucketStatistics.value?.totalGames).toBe(2)
    expect(result.listGames.value.map((item) => item.gameId)).toEqual([1, 2])
    expect(result.isRanked.value).toBe(true)
  })

  it('其他模式不复用排位位置画像', () => {
    const props = reactive<GameStatsBucketProps>({
      matchStatistics: stats(1, [game(3)]),
      positionStats: [{ position: 'TOP', games: 1 } as PositionStats],
      mainPosition: 'TOP',
      scope: { category: 'other', rankedScope: 'mixed' }
    })

    const result = useGameStatsBuckets(props)

    expect(result.bucketPositionStats.value).toBeNull()
    expect(result.bucketMainPosition.value).toBeNull()
    expect(result.isRanked.value).toBe(false)
  })

  it('数据或范围变化时重置列表分页', async () => {
    const props = reactive<GameStatsBucketProps>({
      matchStatistics: stats(
        20,
        Array.from({ length: 20 }, (_, index) => game(index + 1))
      ),
      scope: { category: 'ranked', rankedScope: 'solo' }
    })
    const result = useGameStatsBuckets(props)

    result.loadMore()
    expect(result.showCount.value).toBe(20)
    props.matchStatistics = stats(1, [game(99)])
    await nextTick()

    expect(result.showCount.value).toBe(10)
    expect(result.listGames.value[0]?.gameId).toBe(99)
  })
})
