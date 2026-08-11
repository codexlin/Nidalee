export const useDataStore = defineStore('data', () => {
  // 召唤师数据
  const summonerInfo = ref<SummonerInfo | null>(null)
  const summonerRank = ref<unknown>(null)
  const isSummonerLoaded = ref(false)
  const isSummonerLoading = ref(false)

  // 战绩数据
  const matchHistory = ref<MatchPerformance[]>([])
  const matchStatistics = ref<PlayerMatchStats | null>(null)
  const isMatchHistoryLoaded = ref(false)
  const isMatchHistoryLoading = ref(false)

  // 游戏版本（与 Rust static_catalog 同源，由 AppInit / AppEvents 写入）
  const gameVersion = ref<string>('')

  const summonerLevel = computed(() => {
    return summonerInfo.value?.summonerLevel || 0
  })

  const summonerName = computed(() => {
    return summonerInfo.value?.displayName || ''
  })

  const summonerIcon = computed(() => {
    return summonerInfo.value?.profileIconId || 0
  })

  const totalMatches = computed(() => {
    return matchHistory.value.length
  })

  const winRate = computed(() => {
    if (totalMatches.value === 0) return 0
    const wins = matchHistory.value.filter((match) => match.win).length
    return Math.round((wins / totalMatches.value) * 100)
  })

  const setSummonerInfo = (info: SummonerInfo) => {
    summonerInfo.value = info
    isSummonerLoaded.value = true
    isSummonerLoading.value = false
  }

  const setSummonerRank = (rank: unknown) => {
    summonerRank.value = rank
  }

  const clearSummonerInfo = () => {
    summonerInfo.value = null
    summonerRank.value = null
    isSummonerLoaded.value = false
    isSummonerLoading.value = false
  }

  const startLoadingSummoner = () => {
    isSummonerLoading.value = true
  }

  const setMatchHistory = (matches: MatchPerformance[]) => {
    matchHistory.value = matches
    isMatchHistoryLoaded.value = true
    isMatchHistoryLoading.value = false
  }

  const addMatchToHistory = (match: MatchPerformance) => {
    matchHistory.value.unshift(match)
    if (matchHistory.value.length > 100) {
      matchHistory.value = matchHistory.value.slice(0, 100)
    }
  }

  const setMatchStatistics = (stats: PlayerMatchStats) => {
    matchStatistics.value = stats
    isMatchHistoryLoaded.value = true
    isMatchHistoryLoading.value = false
  }

  const clearMatchHistory = () => {
    matchHistory.value = []
    matchStatistics.value = null
    isMatchHistoryLoaded.value = false
    isMatchHistoryLoading.value = false
  }

  const startLoadingMatchHistory = () => {
    isMatchHistoryLoading.value = true
  }

  const setGameVersion = (version: string) => {
    gameVersion.value = version
  }

  const clearAccountData = () => {
    clearSummonerInfo()
    clearMatchHistory()
  }

  const clearAllData = () => {
    clearAccountData()
  }

  const isDataLoaded = computed(() => {
    return isSummonerLoaded.value && isMatchHistoryLoaded.value
  })

  const isDataLoading = computed(() => {
    return isSummonerLoading.value || isMatchHistoryLoading.value
  })

  return {
    summonerInfo,
    summonerRank,
    isSummonerLoaded,
    isSummonerLoading,
    matchHistory,
    matchStatistics,
    isMatchHistoryLoaded,
    isMatchHistoryLoading,
    gameVersion,
    summonerLevel,
    summonerName,
    summonerIcon,
    totalMatches,
    winRate,
    isDataLoaded,
    isDataLoading,
    setSummonerInfo,
    setSummonerRank,
    clearSummonerInfo,
    startLoadingSummoner,
    setMatchHistory,
    addMatchToHistory,
    setMatchStatistics,
    clearMatchHistory,
    startLoadingMatchHistory,
    setGameVersion,
    clearAccountData,
    clearAllData
  }
})
