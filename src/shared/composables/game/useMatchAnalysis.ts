import { invoke } from '@tauri-apps/api/core'
import { useQueryClient } from '@tanstack/vue-query'
import type { PerformanceScope } from '@/common/performanceScope'
import {
  fetchSummonerAnalysis,
  summonerAnalysisQueryKey
} from '@/features/summoner-performance/composables/useSummonerAnalysisQuery'
import { usePersonalMatchAnalysisStore } from '@/shared/stores/features/personalMatchAnalysisStore'

let analyzeSequence = 0

export function cancelPendingMatchAnalysis() {
  analyzeSequence += 1
}

interface AnalyzeMatchesOptions {
  scope?: PerformanceScope
  background?: boolean
}

/**
 * 当前账号分析协调器。查询结果同时进入 Vue Query 缓存和当前账号 Store；
 * 战绩查询页只消费同一个 Query 缓存，不会再维护第二条后端请求链路。
 */
export function useMatchAnalysis() {
  const queryClient = useQueryClient()
  const analysisStore = usePersonalMatchAnalysisStore()
  const dataStore = useDataStore()
  const activityStore = useActivityStore()
  const settingsStore = useSettingsStore()

  const analyzeMatches = async (options: AnalyzeMatchesOptions = {}): Promise<MatchAnalysisResult | null> => {
    const scope = options.scope ?? settingsStore.performanceScope
    const background = options.background ?? false
    const sequence = ++analyzeSequence

    if (!background) {
      analysisStore.setLoading(true)
      analysisStore.setError(null)
    }

    try {
      let summoner = dataStore.summonerInfo
      if (!summoner?.puuid) {
        summoner = await invoke<SummonerInfo>('get_current_summoner')
        if (summoner) dataStore.setSummonerInfo(summoner)
      }

      const puuid = summoner?.puuid?.trim()
      if (!puuid) throw new Error('当前召唤师 PUUID 不可用')

      const result = await queryClient.fetchQuery({
        queryKey: summonerAnalysisQueryKey(puuid, scope),
        queryFn: () => fetchSummonerAnalysis(puuid, scope),
        staleTime: 0
      })

      if (sequence !== analyzeSequence) return null

      analysisStore.setResult(result, puuid, scope)
      if (!background) {
        activityStore.addActivity('success', `战绩分析完成（${result.displayGames} 场）`, 'data')
      }
      return result
    } catch (error: unknown) {
      if (sequence !== analyzeSequence) return null
      const message = error instanceof Error ? error.message : String(error)
      if (!background) {
        analysisStore.clear()
        analysisStore.setError(message)
        activityStore.addActivity('error', '战绩分析失败', 'error')
      }
      return null
    } finally {
      if (sequence === analyzeSequence) {
        analysisStore.setLoading(false)
      }
    }
  }

  return { analyzeMatches, cancelPendingMatchAnalysis }
}
