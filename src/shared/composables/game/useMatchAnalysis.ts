import { invoke } from '@tauri-apps/api/core'
import { matchModeToInvokeArgs, type MatchModeKey } from '@/common/queueCatalog'
import { usePersonalMatchAnalysisStore } from '@/shared/stores/features/personalMatchAnalysisStore'

/** 忽略过期的 analyze_matches 响应，避免快速切换模式时「有时有数据有时没有」 */
let analyzeSeq = 0

export function cancelPendingMatchAnalysis() {
  analyzeSeq += 1
}

/** 仪表盘 / 列表刷新：基础统计，不拉时间线、不开深度证据 */
const DASHBOARD_OVERVIEW_FLAGS: AnalysisFeatureFlags = {
  enabled: true,
  timeline: false,
  opponent: false,
  teammate: false,
  selfImprovement: false
}

function buildRequest(modeKey: MatchModeKey, count: number): MatchAnalysisRequest {
  const args = matchModeToInvokeArgs(modeKey)

  return {
    count,
    mode: args.analysisMode,
    depth: 'simple',
    queueId: args.queueId ?? undefined,
    features: { ...DASHBOARD_OVERVIEW_FLAGS },
    maxAnalysisGames: count
  }
}

interface AnalyzeMatchesOptions {
  mode?: MatchModeKey
  count?: number
  background?: boolean
}

/**
 * 统一个人战绩分析：Dashboard / 自动刷新只调用一次 `analyze_matches`
 *
 * 列表刷新只做基础统计（胜率/KDA/英雄池等），**不**批量拉时间线做深度证据。
 * 排位过程复盘在对局详情里按需 `get_game_process_review`。
 */
export function useMatchAnalysis() {
  const analysisStore = usePersonalMatchAnalysisStore()
  const dataStore = useDataStore()
  const activityStore = useActivityStore()
  const settingsStore = useSettingsStore()

  /**
   * 单次分析：写入 personalMatchAnalysisStore + dataStore.matchStatistics
   */
  const analyzeMatches = async (options: AnalyzeMatchesOptions = {}): Promise<MatchAnalysisResult | null> => {
    const { mode = settingsStore.lastMatchMode, count = settingsStore.lastMatchCount, background = false } = options
    const seq = ++analyzeSeq

    if (!background) {
      analysisStore.setLoading(true)
      dataStore.startLoadingMatchHistory()
      analysisStore.setError(null)
    }

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
      if (!background) {
        activityStore.addActivity('success', `战绩分析完成（${mode} / ${result.displayGames} 场基础统计）`, 'data')
      }
      return result
    } catch (e: unknown) {
      if (seq !== analyzeSeq) {
        return null
      }
      const message = e instanceof Error ? e.message : String(e)
      console.error('[useMatchAnalysis] analyze_matches 失败:', e)
      if (!background) {
        analysisStore.setError(message)
        analysisStore.clear()
        dataStore.clearMatchHistory()
        activityStore.addActivity('error', '战绩分析失败', 'error')
      }
      return null
    } finally {
      if (seq === analyzeSeq) {
        analysisStore.setLoading(false)
        dataStore.finishLoadingMatchHistory()
      }
    }
  }

  return {
    analyzeMatches,
    cancelPendingMatchAnalysis
  }
}
