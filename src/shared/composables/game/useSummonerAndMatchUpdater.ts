import { invoke } from '@tauri-apps/api/core'
import type { MatchModeKey } from '@/common/queueCatalog'
import { cancelPendingMatchAnalysis, useMatchAnalysis } from '@/shared/composables/game/useMatchAnalysis'
import { getLatestMatchId, runPostGameRefresh } from '@/shared/composables/game/postGameMatchRefresh'

let updateGeneration = 0
let activeAccountInitialization: { generation: number; promise: Promise<void> } | null = null
let pendingReadySummoner: SummonerInfo | null = null
let postGameRefreshGeneration = 0
let pendingPostGameBaseline: number | null | undefined
let activePostGameRefresh: { generation: number; promise: Promise<void> } | null = null
const ACCOUNT_SERVICES_READINESS_DELAY_MS = 350
const SUMMONER_READ_RETRY_DELAYS_MS = [0, 150, 250] as const

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
    dataStore.startLoadingSummoner()
    let lastError: unknown

    for (const delayMs of SUMMONER_READ_RETRY_DELAYS_MS) {
      if (generation !== updateGeneration) return null
      if (delayMs > 0) {
        await new Promise((resolve) => window.setTimeout(resolve, delayMs))
        if (generation !== updateGeneration) return null
      }

      try {
        const summonerInfo = normalizeSummoner(await invoke<SummonerInfo>('get_current_summoner'))
        if (generation === updateGeneration && summonerInfo?.puuid) {
          dataStore.setSummonerInfo(summonerInfo)
          activityStore.addActivity('info', '召唤师信息已更新', 'data')
          return summonerInfo
        }
      } catch (error) {
        lastError = error
      }
    }

    if (generation !== updateGeneration) return null
    console.error('[Updater] 获取召唤师信息失败:', lastError)
    const fallback = readyFallback ?? pendingReadySummoner
    if (fallback?.puuid) {
      dataStore.setSummonerInfo(fallback)
      return fallback
    }
    dataStore.clearSummonerInfo()
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
  const cancelPostGameRefresh = () => {
    postGameRefreshGeneration += 1
    pendingPostGameBaseline = undefined
    activePostGameRefresh = null
  }

  const updateMatchHistory = async (mode?: MatchModeKey, countOverride?: number) => {
    cancelPostGameRefresh()
    return analyzeMatches({ mode, count: countOverride })
  }

  const preparePostGameRefresh = () => {
    postGameRefreshGeneration += 1
    pendingPostGameBaseline = getLatestMatchId(dataStore.matchStatistics)
  }

  const refreshMatchesAfterGame = (): Promise<void> => {
    if (pendingPostGameBaseline === undefined) return Promise.resolve()

    const generation = postGameRefreshGeneration
    if (activePostGameRefresh?.generation === generation) {
      return activePostGameRefresh.promise
    }

    const baselineGameId = pendingPostGameBaseline
    const promise = (async () => {
      const outcome = await runPostGameRefresh({
        baselineGameId,
        refresh: () => analyzeMatches({ background: true }),
        wait: (delayMs) => new Promise((resolve) => window.setTimeout(resolve, delayMs)),
        isCancelled: () => generation !== postGameRefreshGeneration
      })

      if (generation === postGameRefreshGeneration) {
        pendingPostGameBaseline = undefined
        if (outcome === 'exhausted') {
          console.warn('[Updater] 对局结束后 LCU 战绩仍未出现新 gameId，已停止有限重试')
        }
      }
    })()

    activePostGameRefresh = { generation, promise }
    void promise.finally(() => {
      if (activePostGameRefresh?.generation === generation) {
        activePostGameRefresh = null
      }
    })
    return promise
  }

  const updateSummonerAndMatches = (readySummoner?: SummonerInfo): Promise<void> => {
    if (readySummoner?.puuid) {
      pendingReadySummoner = normalizeSummoner(readySummoner)
    }

    // Connected and current-summoner readiness commonly arrive within the same second. They
    // describe one account initialization and must join the same request chain.
    if (activeAccountInitialization) {
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
    cancelPostGameRefresh()
    activeAccountInitialization = null
    pendingReadySummoner = null
    cancelPendingMatchAnalysis()
  }

  return {
    updateSummonerAndMatches,
    updateSummonerInfo,
    updateMatchHistory,
    preparePostGameRefresh,
    refreshMatchesAfterGame,
    cancelPendingUpdates
  }
}
