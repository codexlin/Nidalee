import { invoke } from '@tauri-apps/api/core'

// 专门处理战绩数据获取的 composable
export function useSearchMatches() {
  const { filterMultipleMatchesByQueueTypes } = useMatchFilter()
  const { filterMatchesByQueueTypes } = useMatchFilter()
  const settingsStore = useSettingsStore()

  const loading = ref(true)
  const error = ref('')
  const result = ref<SummonerWithMatches[] | null>(null)
  /** @deprecated 拼写错误保留；请优先用 currentResult */
  const currentRestult = ref<SummonerWithMatches | null>(null)
  const currentResult = currentRestult
  const summonerStats = ref<PlayerMatchStats[] | null>(null)
  const searchText = ref('')
  const cunrrentIndex = ref(-1)
  const names = ref<string[]>([])

  // 类型过滤相关状态
  const selectedQueueTypes = ref<number[]>([])
  const originalMatchData = ref<PlayerMatchStats[] | null>(null) // 保存原始数据
  // 基于当前结果的过滤后统计（适配直接使用 currentRestult.matches 的页面）
  const filteredCurrentMatches = computed<PlayerMatchStats | null>(() => {
    const base = currentRestult.value?.matches as unknown as PlayerMatchStats | undefined
    if (!base) return null
    if (!selectedQueueTypes.value.length) return base
    return filterMatchesByQueueTypes(base, selectedQueueTypes.value)
  })
  async function fetchSummonerInfo(names: string[]): Promise<SummonerWithMatches[] | null> {
    try {
      loading.value = true
      const matches = await invoke<SummonerWithMatches[]>('get_summoners_and_histories', { names })
      if (Array.isArray(matches) && matches.length > 0) {
        result.value = matches
        // 每次查询成功后，重置索引为0（显示第一个结果）
        cunrrentIndex.value = 0
        // 直接设置当前结果，不依赖watch
        currentRestult.value = matches[0]
        // 查询成功
        console.log('matches', matches)
        return matches
      } else {
        // 查询无结果时清空当前结果
        currentRestult.value = null
      }
      return null
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '查询失败'
      currentRestult.value = null
      return null
    } finally {
      loading.value = false
    }
  }
  const onSearch = async () => {
    loading.value = false
    error.value = ''
    result.value = null
    currentRestult.value = null
    cunrrentIndex.value = -1
    if (!searchText.value.trim()) return
    loading.value = true
    try {
      // 支持多个召唤师名，用英文逗号分割
      names.value = searchText.value
        .split(',')
        .map((n) => n.trim())
        .filter(Boolean)
      if (names.value.length === 0) return null
      // 若开启“查询后应用默认过滤”，先把默认队列写入本地过滤
      if (settingsStore.applyDefaultFilterOnSearch && settingsStore.defaultQueueTypes?.length) {
        selectedQueueTypes.value = [...settingsStore.defaultQueueTypes]
      }
      await fetchSummonerInfo(names.value)
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '查询失败'
    } finally {
      loading.value = false
    }
  }
  const getRencentMatchesByPuuid = async (puuid: string[], count: number = 20) => {
    try {
      const settled = await Promise.allSettled(
        puuid.map((id) => invoke<PlayerMatchStats>('get_recent_matches_by_puuid', { puuid: id, count }))
      )
      const successes = settled
        .filter((r): r is PromiseFulfilledResult<PlayerMatchStats> => r.status === 'fulfilled')
        .map((r) => r.value)
      const failures = settled.filter((r) => r.status === 'rejected')

      if (failures.length) {
        console.warn('部分PUUID战绩获取失败，将忽略失败项。失败数量：', failures.length)
      }

      if (successes.length > 0) {
        originalMatchData.value = successes
        applyFilter()
        console.log('获取到的战绩数据(成功项):', successes)
      } else {
        console.warn('未获取到任何战绩数据（全部失败）')
        originalMatchData.value = null
        summonerStats.value = null
      }
    } catch (error) {
      console.error('获取战绩数据失败(整体异常):', error)
      originalMatchData.value = null
      summonerStats.value = null
    }
  }

  // 应用类型过滤
  const applyFilter = () => {
    if (!originalMatchData.value) {
      summonerStats.value = null
      return
    }

    if (selectedQueueTypes.value.length === 0) {
      // 没有选择过滤类型，显示所有数据
      summonerStats.value = originalMatchData.value
    } else {
      // 应用过滤
      summonerStats.value = filterMultipleMatchesByQueueTypes(originalMatchData.value, selectedQueueTypes.value)
    }
  }

  // 设置过滤类型
  const setFilterTypes = (queueTypes: number[]) => {
    selectedQueueTypes.value = queueTypes
    applyFilter()
  }

  // 清空过滤
  const clearFilter = () => {
    selectedQueueTypes.value = []
    applyFilter()
  }
  watch(cunrrentIndex, (val) => {
    if (result.value) {
      currentRestult.value = result.value[val]
    }
  })
  return {
    getRencentMatchesByPuuid,
    currentRestult,
    currentResult,
    filteredCurrentMatches,
    summonerStats,
    names,
    searchText,
    cunrrentIndex,
    onSearch,
    fetchSummonerInfo,
    loading,
    result,
    error,
    // 新增的过滤相关功能
    selectedQueueTypes,
    setFilterTypes,
    clearFilter,
    originalMatchData
  }
}
