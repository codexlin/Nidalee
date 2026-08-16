import { invoke } from '@tauri-apps/api/core'
import { computed, shallowRef } from 'vue'
import { useSearchHistoryStore } from '@/shared/stores/features/searchHistoryStore'
import { createLatestRequestGuard } from '@/shared/utils/latestRequest'

/** Resolves one or more Riot IDs to stable SummonerInfo identities. */
export function useSearchMatches() {
  const searchHistoryStore = useSearchHistoryStore()
  const loading = shallowRef(false)
  const error = shallowRef('')
  const result = shallowRef<SummonerInfo[]>([])
  const searchText = shallowRef('')
  const currentIndex = shallowRef(-1)
  const names = shallowRef<string[]>([])
  const requests = createLatestRequestGuard()

  const currentResult = computed(() => (currentIndex.value >= 0 ? (result.value[currentIndex.value] ?? null) : null))

  function clearSummonerInfo() {
    requests.invalidate()
    loading.value = false
    error.value = ''
    result.value = []
    currentIndex.value = -1
  }

  async function fetchSummonerInfo(summonerNames: string[]): Promise<SummonerInfo[]> {
    const request = requests.begin()
    loading.value = true
    error.value = ''
    try {
      const summoners = await invoke<SummonerInfo[]>('get_summoners_by_names', { names: summonerNames })
      if (!request.isCurrent()) return []
      if (!summoners.length) {
        error.value = '未找到匹配的召唤师，请检查名称与标签。'
        result.value = []
        currentIndex.value = -1
        return []
      }

      result.value = summoners
      currentIndex.value = summoners.length ? 0 : -1
      searchHistoryStore.add(
        summoners
          .map((summoner) => summoner.displayName || [summoner.gameName, summoner.tagLine].filter(Boolean).join('#'))
          .filter(Boolean)
      )
      return summoners
    } catch (cause: unknown) {
      if (!request.isCurrent()) return []
      error.value = cause instanceof Error ? cause.message : String(cause)
      result.value = []
      currentIndex.value = -1
      return []
    } finally {
      if (request.isCurrent()) loading.value = false
    }
  }

  async function onSearch() {
    clearSummonerInfo()
    names.value = searchText.value
      .split(',')
      .map((name) => name.trim())
      .filter(Boolean)
    if (!names.value.length) return []
    return fetchSummonerInfo(names.value)
  }

  return {
    currentResult,
    names,
    searchText,
    currentIndex,
    onSearch,
    fetchSummonerInfo,
    loading,
    result,
    error,
    clearSummonerInfo
  }
}
