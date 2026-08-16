export const useDataStore = defineStore('data', () => {
  // 召唤师数据
  const summonerInfo = ref<SummonerInfo | null>(null)
  const summonerRank = ref<unknown>(null)
  const isSummonerLoaded = ref(false)
  const isSummonerLoading = ref(false)

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

  const setGameVersion = (version: string) => {
    gameVersion.value = version
  }

  const clearAccountData = () => {
    clearSummonerInfo()
  }

  const clearAllData = () => {
    clearAccountData()
  }

  return {
    summonerInfo,
    summonerRank,
    isSummonerLoaded,
    isSummonerLoading,
    gameVersion,
    summonerLevel,
    summonerName,
    summonerIcon,
    setSummonerInfo,
    setSummonerRank,
    clearSummonerInfo,
    startLoadingSummoner,
    setGameVersion,
    clearAccountData,
    clearAllData
  }
})
