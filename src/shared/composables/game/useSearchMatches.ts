import { invoke } from '@tauri-apps/api/core'
import { computed, ref, watch } from 'vue'
import { RANKED_QUEUE_IDS, matchModeExcludesRanked, normalizeMatchModeKey } from '@/common/queueCatalog'
import { useSettingsStore } from '@/shared/stores/ui/settingsStore'
import { useSearchHistoryStore } from '@/shared/stores/features/searchHistoryStore'
import { createLatestRequestGuard } from '@/shared/utils/latestRequest'
import { useMatchFilter } from './useMatchFilter'

// 专门处理战绩数据获取的 composable
export function useSearchMatches() {
  const { filterMultipleMatchesByQueueTypes } = useMatchFilter()
  const { filterMatchesByQueueTypes } = useMatchFilter()
  const settingsStore = useSettingsStore()
  const searchHistoryStore = useSearchHistoryStore()

  const loading = ref(false)
  const error = ref('')
  const result = ref<SummonerWithMatches[] | null>(null)
  const currentResult = ref<SummonerWithMatches | null>(null)
  const summonerStats = ref<PlayerMatchStats[] | null>(null)
  const searchText = ref('')
  const currentIndex = ref(-1)
  const names = ref<string[]>([])
  const summonerRequests = createLatestRequestGuard()
  const recentMatchesRequests = createLatestRequestGuard()

  // 类型过滤相关状态
  const selectedQueueTypes = ref<number[]>([])
  /** 普通模式：排除排位队列 */
  const excludeSelectedQueues = ref(false)
  const originalMatchData = ref<PlayerMatchStats[] | null>(null) // 保存原始数据
  // 基于当前结果的过滤后统计（适配直接使用 currentResult.matches 的页面）
  const filteredCurrentMatches = computed<PlayerMatchStats | null>(() => {
    const base = currentResult.value?.matches as unknown as PlayerMatchStats | undefined
    if (!base) return null
    if (!selectedQueueTypes.value.length) return base
    return filterMatchesByQueueTypes(base, selectedQueueTypes.value, {
      exclude: excludeSelectedQueues.value
    })
  })
  const clearSummonerInfo = () => {
    summonerRequests.invalidate()
    loading.value = false
    error.value = ''
    result.value = null
    currentResult.value = null
    currentIndex.value = -1
  }

  async function fetchSummonerInfo(summonerNames: string[]): Promise<SummonerWithMatches[] | null> {
    const request = summonerRequests.begin()
    try {
      loading.value = true
      error.value = ''
      const matches = await invoke<SummonerWithMatches[]>('get_summoners_and_histories', {
        names: summonerNames
      })
      if (!request.isCurrent()) return null

      if (Array.isArray(matches) && matches.length > 0) {
        result.value = matches
        // 每次查询成功后，重置索引为0（显示第一个结果）
        currentIndex.value = 0
        // 直接设置当前结果，不依赖watch
        currentResult.value = matches[0]
        searchHistoryStore.add(
          matches.map((m) => m.displayName || m.summonerInfo?.displayName || '').filter(Boolean)
        )
        return matches
      } else {
        // 查询无结果时清空当前结果
        result.value = null
        currentResult.value = null
        currentIndex.value = -1
      }
      return null
    } catch (e: unknown) {
      if (!request.isCurrent()) return null
      error.value = e instanceof Error ? e.message : '查询失败'
      result.value = null
      currentResult.value = null
      currentIndex.value = -1
      return null
    } finally {
      if (request.isCurrent()) loading.value = false
    }
  }
  const onSearch = async () => {
    clearSummonerInfo()
    names.value = []
    if (!searchText.value.trim()) return

    // 支持多个召唤师名，用英文逗号分割
    names.value = searchText.value
      .split(',')
      .map((n) => n.trim())
      .filter(Boolean)
    if (names.value.length === 0) return null

    // 若开启“查询后应用默认过滤”，先把默认队列写入本地过滤
    if (settingsStore.applyDefaultFilterOnSearch) {
      const mode = normalizeMatchModeKey(settingsStore.lastMatchMode)
      if (matchModeExcludesRanked(mode)) {
        selectedQueueTypes.value = [...RANKED_QUEUE_IDS]
        excludeSelectedQueues.value = true
      } else if (settingsStore.defaultQueueTypes?.length) {
        selectedQueueTypes.value = [...settingsStore.defaultQueueTypes]
        excludeSelectedQueues.value = false
      }
    }

    return fetchSummonerInfo(names.value)
  }
  const getRecentMatchesByPuuid = async (puuid: string[], count: number = 20) => {
    const request = recentMatchesRequests.begin()
    try {
      const settled = await Promise.allSettled(
        puuid.map((id) => invoke<PlayerMatchStats>('get_recent_matches_by_puuid', { puuid: id, count }))
      )
      if (!request.isCurrent()) return

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
      if (!request.isCurrent()) return
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
      summonerStats.value = filterMultipleMatchesByQueueTypes(originalMatchData.value, selectedQueueTypes.value, {
        exclude: excludeSelectedQueues.value
      })
    }
  }

  // 设置过滤类型
  const setFilterTypes = (queueTypes: number[]) => {
    selectedQueueTypes.value = queueTypes
    excludeSelectedQueues.value = false
    applyFilter()
  }

  // 清空过滤
  const clearFilter = () => {
    selectedQueueTypes.value = []
    excludeSelectedQueues.value = false
    applyFilter()
  }
  watch(currentIndex, (val) => {
    currentResult.value = result.value && val >= 0 ? (result.value[val] ?? null) : null
  })
  return {
    getRecentMatchesByPuuid,
    currentResult,
    filteredCurrentMatches,
    summonerStats,
    names,
    searchText,
    currentIndex,
    onSearch,
    fetchSummonerInfo,
    loading,
    result,
    error,
    clearSummonerInfo,
    // 新增的过滤相关功能
    selectedQueueTypes,
    setFilterTypes,
    clearFilter,
    originalMatchData
  }
}
