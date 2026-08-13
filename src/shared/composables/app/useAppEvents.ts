import { useMatchAnalysisStore } from '@/features/match-analysis/store'
import { listen, type Event } from '@tauri-apps/api/event'
import { debounce, isObject } from 'radash'
import { useQueryClient } from '@tanstack/vue-query'

// 创建一个模块级别的状态，用于跟踪监听器
let unlisteners: (() => void)[] = []
let listenersReady = false
let listeningRequested = false
let startListeningPromise: Promise<boolean> | null = null
let listenerGeneration = 0
let cancelActiveConnectionStateDebounce: (() => void) | null = null
let currentGameVersion: string | null = null
let lastStaticCatalogCheckAt = 0
const STATIC_CATALOG_CHECK_TTL_MS = 30_000

/**
 * 应用事件处理组合式函数
 * 职责：监听和处理游戏相关的事件
 */
export function useAppEvents() {
  const gamePhaseManager = useGamePhaseManager()
  const champSelectManager = useChampSelectManager()
  const { updateMatchHistory, updateSummonerInfo, updateSummonerAndMatches, cancelPendingUpdates } =
    useSummonerAndMatchUpdater()
  const connectionStore = useConnectionStore()
  const gameStore = useGameStore()
  const dataStore = useDataStore()
  const matchmakingStore = useMatchmakingStore()
  const matchAnalysisStore = useMatchAnalysisStore()
  const queryClient = useQueryClient() // ← 移到外层，所有函数共享同一个实例

  const { handleGamePhaseChange, cancelPendingAutoAccept } = gamePhaseManager
  const { handleLobbyChange, handleChampSelectChange } = champSelectManager

  // 事件处理函数
  const handleGameFlowPhaseChange = (event: Event<string>) => {
    const phase = event.payload as string
    handleGamePhaseChange(phase)
  }

  const handleGameflowSessionChanged = (_event: Event<unknown>) => {
    // 可根据需要处理
  }

  const handleLobbyChangeEvent = (event: Event<LobbyInfo | null>) => {
    handleLobbyChange(event.payload as LobbyInfo | null)
  }

  const handleChampSelectSessionChanged = (event: Event<ChampSelectSession | null>) => {
    handleChampSelectChange(event.payload)
  }

  const handleMatchmakingStateChanged = (event: Event<MatchmakingState | null>) => {
    if (event.payload) matchmakingStore.updateState(event.payload)
    else matchmakingStore.clearState()
  }

  const handleSummonerChange = (event: Event<SummonerInfo | null>) => {
    const summoner = event.payload
    if (!summoner?.puuid) {
      // LCU can publish current-summoner Delete several seconds before the socket actually
      // closes. Cancel stale work now, but let the authoritative Disconnected transition own
      // visible account/game cleanup so the UI never shows Connected with empty data.
      cancelPendingUpdates()
      return
    }

    const current = dataStore.summonerInfo
    if (current?.puuid === summoner.puuid && current.displayName) return

    // The transport can connect before current-summoner is HTTP-ready. This event is the
    // authoritative readiness signal; retry the ordered account -> match initialization now.
    void updateSummonerAndMatches(summoner)
  }

  const updateTeamAnalysisData = (data: TeamAnalysisData | null) => {
    matchAnalysisStore.setTeamAnalysisData(data)
  }

  // 战绩分析数据（异步到达）
  const handleTeamAnalysisData = (event: Event<TeamAnalysisData | null>) => {
    updateTeamAnalysisData(event.payload)
  }

  const handleConnectionStateChange = async (event: Event<unknown>) => {
    const state = event.payload as ConnectedState
    const nextState = isObject(state) ? state.state : state
    await connectionStore.updateConnectionState(nextState)

    if (nextState === 'Disconnected') {
      matchAnalysisStore.clearAllData()
      matchmakingStore.clearState()
    }

    // 当连接成功时，检查游戏版本
    if (nextState === 'Connected') {
      await checkGameVersion()
    }
  }

  // 检查游戏版本：委托 Rust 刷新静态包（短 TTL，避免快速重连刷屏）
  const checkGameVersion = async () => {
    const now = Date.now()
    if (now - lastStaticCatalogCheckAt < STATIC_CATALOG_CHECK_TTL_MS && currentGameVersion) {
      return
    }
    lastStaticCatalogCheckAt = now

    try {
      const { refreshStaticCatalogsOnVersionChange } = await import('@/shared/composables/data/useVersionedData')
      const { refreshed, meta } = await refreshStaticCatalogsOnVersionChange(queryClient)
      dataStore.setGameVersion(meta.version)

      if (refreshed && currentGameVersion) {
      } else if (!currentGameVersion) {
      } else {
      }
      currentGameVersion = meta.version
    } catch (error) {
      console.error('[AppEvents] 检查游戏版本失败:', error)
    }
  }

  const handleConnectionStateChangeDebounced = debounce({ delay: 300 }, handleConnectionStateChange)

  const stopListening = () => {
    cancelPendingAutoAccept()
    cancelPendingUpdates()
    gameStore.clearChampSelect()
    if (!listeningRequested && !listenersReady && unlisteners.length === 0) return

    listeningRequested = false
    listenerGeneration += 1
    handleConnectionStateChangeDebounced.cancel()
    cancelActiveConnectionStateDebounce?.()
    cancelActiveConnectionStateDebounce = null
    unlisteners.forEach((unlisten) => {
      try {
        unlisten()
      } catch (error) {
        console.warn('[AppEvents] 事件监听器清理失败:', error)
      }
    })
    unlisteners = []
    listenersReady = false
    // 允许旧的异步注册收尾前开启新一代监听；旧 generation 会自行卸载。
    startListeningPromise = null
  }

  const startListening = (): Promise<boolean> => {
    if (listenersReady) return Promise.resolve(true)
    if (startListeningPromise) return startListeningPromise

    listeningRequested = true
    const generation = ++listenerGeneration
    const registered: (() => void)[] = []

    const register = async <T>(event: string, handler: (event: Event<T>) => void) => {
      const unlisten = await listen<T>(event, handler)
      if (!listeningRequested || listenerGeneration !== generation) {
        unlisten()
        return false
      }
      registered.push(unlisten)
      return true
    }

    const cleanupRegistered = () => {
      handleConnectionStateChangeDebounced.cancel()
      registered.forEach((unlisten) => {
        try {
          unlisten()
        } catch (error) {
          console.warn('[AppEvents] 未完成注册的事件监听器清理失败:', error)
        }
      })
      registered.length = 0
    }

    const pending = (async (): Promise<boolean> => {
      try {
        const registrations = [
          () => register('gameflow-phase-change', handleGameFlowPhaseChange),
          () => register('gameflow-session-changed', handleGameflowSessionChanged),
          () => register('lobby-change', handleLobbyChangeEvent),
          () => register('champ-select-session-changed', handleChampSelectSessionChanged),
          () => register('matchmaking-state-changed', handleMatchmakingStateChanged),
          () => register('summoner-change', handleSummonerChange),
          () => register('connection-state-changed', handleConnectionStateChangeDebounced),
          () => register('team-analysis-data', handleTeamAnalysisData)
        ]

        for (const registration of registrations) {
          if (!(await registration())) {
            cleanupRegistered()
            return false
          }
        }

        if (!listeningRequested || listenerGeneration !== generation) {
          cleanupRegistered()
          return false
        }

        unlisteners = registered
        listenersReady = true
        cancelActiveConnectionStateDebounce = () => handleConnectionStateChangeDebounced.cancel()

        return true
      } catch (error) {
        console.error('[AppEvents] 启动全局事件监听失败:', error)
        cleanupRegistered()
        if (listenerGeneration === generation) {
          unlisteners = []
          listenersReady = false
          listeningRequested = false
          listenerGeneration += 1
        }
        return false
      }
    })()

    startListeningPromise = pending
    const clearPending = () => {
      if (startListeningPromise === pending) startListeningPromise = null
    }
    void pending.then(clearPending, clearPending)
    return pending
  }

  return {
    updateMatchHistory,
    updateSummonerInfo,
    startListening,
    stopListening
  }
}
