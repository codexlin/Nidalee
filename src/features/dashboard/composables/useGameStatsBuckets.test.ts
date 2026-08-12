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

function game(id: number, win = true): MatchPerformance {
  return { gameId: id, win, championId: 1, kills: 1, deaths: 1, assists: 1 } as MatchPerformance
}

describe('useGameStatsBuckets', () => {
  it('全部模式下默认走排位桶，列表跟当前桶', async () => {
    const props = reactive<GameStatsBucketProps>({
      matchStatistics: stats(20),
      rankedStats: stats(8, [game(1), game(2)]),
      otherStats: stats(12, [game(9)]),
      selectedMatchMode: 'all',
      matchCount: 20
    })

    const { activeBucket, listGames, isRankedBucketActive, showBucketTabs } = useGameStatsBuckets(props)
    await nextTick()

    expect(showBucketTabs.value).toBe(true)
    expect(activeBucket.value).toBe('ranked')
    expect(isRankedBucketActive.value).toBe(true)
    expect(listGames.value.map((g) => g.gameId)).toEqual([1, 2])

    activeBucket.value = 'other'
    await nextTick()
    expect(isRankedBucketActive.value).toBe(false)
    expect(listGames.value.map((g) => g.gameId)).toEqual([9])
  })

  it('normals 模式强制其他桶', async () => {
    const props = reactive<GameStatsBucketProps>({
      matchStatistics: stats(5, [game(3)]),
      rankedStats: stats(2, [game(1)]),
      otherStats: stats(5, [game(3)]),
      selectedMatchMode: 'normals'
    })

    const { activeBucket, isRankedBucketActive, listGames } = useGameStatsBuckets(props)
    await nextTick()

    expect(activeBucket.value).toBe('other')
    expect(isRankedBucketActive.value).toBe(false)
    expect(listGames.value.map((g) => g.gameId)).toEqual([3])
  })

  it('切换列表数据时重置分页', async () => {
    const props = reactive<GameStatsBucketProps>({
      matchStatistics: stats(
        30,
        Array.from({ length: 25 }, (_, i) => game(i + 1))
      ),
      rankedStats: stats(
        30,
        Array.from({ length: 25 }, (_, i) => game(i + 1))
      ),
      selectedMatchMode: 'mixedRanked'
    })

    const { showCount, loadMore, listGames } = useGameStatsBuckets(props)
    await nextTick()
    expect(showCount.value).toBe(10)
    loadMore()
    expect(showCount.value).toBe(20)

    props.rankedStats = stats(5, [game(99)])
    await nextTick()
    expect(listGames.value).toHaveLength(1)
    expect(showCount.value).toBe(10)
  })
})
