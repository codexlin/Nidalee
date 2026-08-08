// 战绩过滤 composable
export function useMatchFilter() {
  // 过滤战绩数据
  const filterMatchesByQueueTypes = (
    matchStatistics: PlayerMatchStats | null,
    selectedQueueTypes: number[]
  ): PlayerMatchStats | null => {
    if (!matchStatistics || selectedQueueTypes.length === 0) {
      return matchStatistics
    }

    // 过滤最近战绩
    const filteredRecentPerformance = matchStatistics.recentPerformance.filter((game) =>
      selectedQueueTypes.includes(Number(game.queueId))
    )

    // 重新计算统计数据
    const totalGames = filteredRecentPerformance.length
    const wins = filteredRecentPerformance.filter((game) => game.win).length
    const losses = totalGames - wins
    const winRate = totalGames > 0 ? (wins / totalGames) * 100 : 0

    // 计算平均KDA
    const totalKills = filteredRecentPerformance.reduce((sum, game) => sum + game.kills, 0)
    const totalDeaths = filteredRecentPerformance.reduce((sum, game) => sum + game.deaths, 0)
    const totalAssists = filteredRecentPerformance.reduce((sum, game) => sum + game.assists, 0)

    const avgKills = totalGames > 0 ? totalKills / totalGames : 0
    const avgDeaths = totalGames > 0 ? totalDeaths / totalGames : 0
    const avgAssists = totalGames > 0 ? totalAssists / totalGames : 0
    const avgKda = avgDeaths > 0 ? (avgKills + avgAssists) / avgDeaths : avgKills + avgAssists

    // 重新计算英雄统计
    const championStatsMap = new Map<number, { games: number; wins: number }>()

    filteredRecentPerformance.forEach((game) => {
      const existing = championStatsMap.get(game.championId) || { games: 0, wins: 0 }
      championStatsMap.set(game.championId, {
        games: existing.games + 1,
        wins: existing.wins + (game.win ? 1 : 0)
      })
    })

    const favoriteChampions = Array.from(championStatsMap.entries())
      .map(([championId, stats]) => ({
        championId,
        championName: '', // 需要在使用时填充
        games: stats.games,
        wins: stats.wins,
        winRate: stats.games > 0 ? (stats.wins / stats.games) * 100 : 0
      }))
      .sort((a, b) => b.games - a.games)
      .slice(0, 5) // 取前5个常用英雄

    return {
      ...matchStatistics,
      totalGames,
      wins,
      losses,
      winRate,
      avgKills,
      avgDeaths,
      avgAssists,
      avgKda,
      recentPerformance: filteredRecentPerformance,
      favoriteChampions: favoriteChampions as AnalysisChampionStats[]
    }
  }

  // 过滤多个玩家的战绩数据
  const filterMultipleMatchesByQueueTypes = (
    matchStatisticsArray: PlayerMatchStats[] | null,
    selectedQueueTypes: number[]
  ): PlayerMatchStats[] | null => {
    if (!matchStatisticsArray || selectedQueueTypes.length === 0) {
      return matchStatisticsArray
    }

    return matchStatisticsArray
      .map((stats) => filterMatchesByQueueTypes(stats, selectedQueueTypes))
      .filter(Boolean) as PlayerMatchStats[]
  }

  // 获取战绩统计信息
  const getMatchStatsSummary = (matchStatistics: PlayerMatchStats | null) => {
    if (!matchStatistics) {
      return {
        totalGames: 0,
        wins: 0,
        losses: 0,
        winRate: 0,
        avgKda: 0,
        recentGamesCount: 0
      }
    }

    return {
      totalGames: matchStatistics.totalGames || 0,
      wins: matchStatistics.wins || 0,
      losses: matchStatistics.losses || 0,
      winRate: matchStatistics.winRate || 0,
      avgKda: matchStatistics.avgKda || 0,
      recentGamesCount: matchStatistics.recentPerformance?.length || 0
    }
  }

  // 按队列类型分组战绩
  const groupMatchesByQueueType = (recentPerformance: RecentGame[]) => {
    const queueGroups = new Map<number, RecentGame[]>()

    recentPerformance.forEach((game) => {
      const existing = queueGroups.get(game.queueId) || []
      queueGroups.set(game.queueId, [...existing, game])
    })

    return Array.from(queueGroups.entries()).map(([queueId, games]) => ({
      queueId,
      games,
      wins: games.filter((g) => g.win).length,
      losses: games.filter((g) => !g.win).length,
      winRate: games.length > 0 ? (games.filter((g) => g.win).length / games.length) * 100 : 0
    }))
  }

  return {
    filterMatchesByQueueTypes,
    filterMultipleMatchesByQueueTypes,
    getMatchStatsSummary,
    groupMatchesByQueueType
  }
}
