import { invoke } from '@tauri-apps/api/core'
import type { MatchModeKey } from '@/common/queueCatalog'
import { cancelPendingMatchAnalysis, useMatchAnalysis } from '@/shared/composables/game/useMatchAnalysis'

let updateGeneration = 0
let activeAccountInitialization: { generation: number; promise: Promise<void> } | null = null
let pendingReadySummoner: SummonerInfo | null = null
const ACCOUNT_SERVICES_READINESS_DELAY_MS = 350

function waitForAccountServices(): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ACCOUNT_SERVICES_READINESS_DELAY_MS))
}

function normalizeSummoner(info: SummonerInfo): SummonerInfo {
  if (info.displayName) return info
  const riotId = [info.gameName, info.tagLine].filter(Boolean).join('#')
  return riotId ? { ...info, displayName: riotId } : info
}

/**
 * 统一更新召唤师信息和战绩信息
 *
 * 战绩路径已收敛到 `analyze_matches`（单次查询）。
 */
export function useSummonerAndMatchUpdater() {
  const dataStore = useDataStore()
  const activityStore = useActivityStore()
  const { analyzeMatches } = useMatchAnalysis()

  const fetchAndCommitSummoner = async (
    generation: number,
    readyFallback: SummonerInfo | null = null
  ): Promise<SummonerInfo | null> => {
    try {
      dataStore.startLoadingSummoner()
      const summonerInfo = normalizeSummoner(await invoke<SummonerInfo>('get_current_summoner'))
      if (generation === updateGeneration && summonerInfo?.puuid) {
        dataStore.setSummonerInfo(summonerInfo)
        activityStore.addActivity('info', '召唤师信息已更新', 'data')
        return summonerInfo
      }
    } catch (error) {
      if (generation !== updateGeneration) return null
      console.error('[Updater] 获取召唤师信息失败:', error)
      const fallback = readyFallback ?? pendingReadySummoner
      if (fallback?.puuid) {
        dataStore.setSummonerInfo(fallback)
        return fallback
      }
      dataStore.clearSummonerInfo()
    }
    return null
  }

  const updateSummonerInfo = async () => {
    const generation = ++updateGeneration
    return fetchAndCommitSummoner(generation)
  }

  /**
   * 更新战绩信息（单次 analyze_matches）
   * 无参时与仪表盘共用 settingsStore.lastMatchMode / lastMatchCount
   */
  const updateMatchHistory = async (mode?: MatchModeKey, countOverride?: number) => {
    await analyzeMatches(mode, countOverride)
  }

  const updateSummonerAndMatches = (readySummoner?: SummonerInfo): Promise<void> => {
    if (readySummoner?.puuid) {
      pendingReadySummoner = normalizeSummoner(readySummoner)
    }

    // Connected and current-summoner readiness commonly arrive within the same second. They
    // describe one account initialization and must join the same request chain.
    if (activeAccountInitialization) {
      console.log('[Updater] 合并重复的账号初始化请求')
      return activeAccountInitialization.promise
    }

    const generation = ++updateGeneration
    const promise = (async () => {
      const readyFallback = pendingReadySummoner
      if (readyFallback?.puuid) {
        // Make the authoritative WS identity visible immediately. The HTTP call below enriches
        // ranks/challenges when that endpoint has finished becoming ready.
        dataStore.setSummonerInfo(readyFallback)
      }
      const summoner = await fetchAndCommitSummoner(generation, readyFallback)
      if (!summoner || generation !== updateGeneration) return

      // LCU can expose current-summoner slightly before the match-history plugin is ready.
      // The old Dashboard duplicate accidentally acted as this delay; keep one explicit,
      // generation-guarded request instead.
      await waitForAccountServices()
      if (generation !== updateGeneration) return

      // Match analysis must use the PUUID committed by this account session. Running these
      // requests in parallel allowed persisted/previous-account state to leak into a reconnect.
      await updateMatchHistory()
    })()

    activeAccountInitialization = { generation, promise }
    void promise.finally(() => {
      if (activeAccountInitialization?.generation === generation) {
        activeAccountInitialization = null
        pendingReadySummoner = null
      }
    })
    return promise
  }

  const cancelPendingUpdates = () => {
    updateGeneration += 1
    activeAccountInitialization = null
    pendingReadySummoner = null
    cancelPendingMatchAnalysis()
  }

  return {
    updateSummonerAndMatches,
    updateSummonerInfo,
    updateMatchHistory,
    cancelPendingUpdates
  }
}
