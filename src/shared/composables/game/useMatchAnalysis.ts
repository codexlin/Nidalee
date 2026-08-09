import { invoke } from '@tauri-apps/api/core'
import { matchModeToInvokeArgs, type MatchModeKey } from '@/common/queueCatalog'
import { useAnalysisSettingsStore } from '@/shared/stores/features/analysisSettingsStore'
import { usePersonalMatchAnalysisStore } from '@/shared/stores/features/personalMatchAnalysisStore'

/** 忽略过期的 analyze_matches 响应，避免快速切换模式时「有时有数据有时没有」 */
let analyzeSeq = 0

/**
 * 统一个人战绩分析：Dashboard / 自动刷新只调用一次 `analyze_matches`
 */
export function useMatchAnalysis() {
  const analysisSettings = useAnalysisSettingsStore()
  const analysisStore = usePersonalMatchAnalysisStore()
  const dataStore = useDataStore()
  const activityStore = useActivityStore()
  const settingsStore = useSettingsStore()

  const buildRequest = (
    modeKey: MatchModeKey,
    count: number,
    overrides?: Partial<MatchAnalysisRequest>
  ): MatchAnalysisRequest => {
    const args = matchModeToInvokeArgs(modeKey)

    return {
      count,
      mode: args.analysisMode,
      depth: analysisSettings.config.depth,
      queueId: args.queueId ?? undefined,
      features: analysisSettings.toFeatureFlags(),
      maxAnalysisGames: analysisSettings.config.maxAnalysisGames,
      ...overrides
    }
  }

  /**
   * 单次分析：写入 personalMatchAnalysisStore + dataStore.matchStatistics
   */
  const analyzeMatches = async (
    modeKey?: MatchModeKey,
    countOverride?: number
  ): Promise<MatchAnalysisResult | null> => {
    const mode = modeKey ?? settingsStore.lastMatchMode
    const count = countOverride ?? settingsStore.lastMatchCount
    const seq = ++analyzeSeq

    analysisStore.setLoading(true)
    dataStore.startLoadingMatchHistory()
    analysisStore.setError(null)

    try {
      let summoner = dataStore.summonerInfo
      if (!summoner?.puuid) {
        summoner = await invoke<SummonerInfo>('get_current_summoner')
        if (summoner) dataStore.setSummonerInfo(summoner)
      }
      const puuid = summoner?.puuid
      if (!puuid) {
        throw new Error('当前召唤师 PUUID 不可用')
      }

      const request = buildRequest(mode, count)
      const result = await invoke<MatchAnalysisResult>('analyze_matches', { puuid, request })

      // 期间又发起了新请求：丢弃本次结果，避免旧模式盖住新筛选
      if (seq !== analyzeSeq) {
        return null
      }

      analysisStore.setResult(result, puuid)
      dataStore.setMatchStatistics(result.overallStats)
      activityStore.addActivity(
        'success',
        `战绩分析完成（${mode} / 展示 ${result.displayGames} 场 / 深度 ${result.analyzedGames} 场）`,
        'data'
      )
      return result
    } catch (e: unknown) {
      if (seq !== analyzeSeq) {
        return null
      }
      const message = e instanceof Error ? e.message : String(e)
      console.error('[useMatchAnalysis] analyze_matches 失败:', e)
      analysisStore.setError(message)
      analysisStore.clear()
      dataStore.clearMatchHistory()
      activityStore.addActivity('error', '战绩分析失败', 'error')
      return null
    } finally {
      if (seq === analyzeSeq) {
        analysisStore.setLoading(false)
      }
    }
  }

  /**
   * 分析任意 PUUID（不写 personalMatchAnalysisStore；供战绩搜索等场景）
   */
  const analyzeMatchesForPuuid = async (
    puuid: string,
    modeKey: MatchModeKey = 'all',
    count = 20,
    overrides?: Partial<MatchAnalysisRequest>
  ): Promise<MatchAnalysisResult | null> => {
    try {
      const request = buildRequest(modeKey, count, overrides)
      return await invoke<MatchAnalysisResult>('analyze_matches', { puuid, request })
    } catch (e: unknown) {
      console.error('[useMatchAnalysis] analyzeMatchesForPuuid 失败:', e)
      return null
    }
  }

  return {
    buildRequest,
    analyzeMatches,
    analyzeMatchesForPuuid,
    analysisStore
  }
}
