import { useMatchAnalysisStore } from '@/features/match-analysis/store'
import { listen, type Event } from '@tauri-apps/api/event'
import { debounce, isObject } from 'radash'
import { useQueryClient } from '@tanstack/vue-query'

// 创建一个模块级别的状态，用于跟踪监听器
let unlisteners: (() => void)[] = []
let isListeningStarted = false
let listenerGeneration = 0
let currentGameVersion: string | null = null

/**
 * 应用事件处理组合式函数
 * 职责：监听和处理游戏相关的事件
 */
export function useAppEvents() {
  const gamePhaseManager = useGamePhaseManager()
  const champSelectManager = useChampSelectManager()
  const { updateMatchHistory, updateSummonerInfo } = useSummonerAndMatchUpdater()
  const connectionStore = useConnectionStore()
  const matchmakingStore = useMatchmakingStore()
  const matchAnalysisStore = useMatchAnalysisStore()
  const queryClient = useQueryClient() // ← 移到外层，所有函数共享同一个实例

  const { handleGamePhaseChange } = gamePhaseManager
  const { handleLobbyChange, handleChampSelectChange } = champSelectManager

  // 事件处理函数
  const handleGameFlowPhaseChange = (event: Event<string>) => {
    console.log('[AppEvents] 游戏阶段变化:', event.payload)
    const phase = event.payload as string
    handleGamePhaseChange(phase)
  }

  const handleGameflowSessionChanged = (event: Event<unknown>) => {
    console.log('[AppEvents] Gameflow Session 变化:', event.payload)
    // 可根据需要处理
  }

  const handleLobbyChangeEvent = (event: Event<LobbyInfo | null>) => {
    console.log('[AppEvents] 大厅变化:', event.payload)
    handleLobbyChange(event.payload as LobbyInfo | null)
  }

  const handleChampSelectSessionChanged = (event: Event<ChampSelectSession | null>) => {
    console.log('[AppEvents] 英雄选择 Session 变化:', event.payload)
    handleChampSelectChange(event.payload)
  }

  const handleMatchmakingStateChanged = (event: Event<MatchmakingState>) => {
    console.log('[AppEvents] 匹配状态变化:', event.payload)
    matchmakingStore.updateState(event.payload)
  }

  // 战绩分析数据（异步到达）
  const handleTeamAnalysisData = (event: { payload: TeamAnalysisData | null }) => {
    console.log('[AppEvents] 收到战绩分析数据，更新 UI')
    matchAnalysisStore.setTeamAnalysisData(event.payload)
  }

  const handleConnectionStateChange = async (event: Event<unknown>) => {
    const state = event.payload as ConnectedState
    await connectionStore.updateConnectionState(isObject(state) ? state.state : state)

    // 当连接成功时，检查游戏版本
    if (isObject(state) ? state.state === 'Connected' : state === 'Connected') {
      await checkGameVersion()
    }
  }

  // 检查游戏版本，版本变化时清空静态数据缓存
  const checkGameVersion = async () => {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const newVersion = await invoke<string>('get_game_version')

      if (newVersion && newVersion !== currentGameVersion) {
        if (currentGameVersion) {
          console.log(`[AppEvents] 检测到游戏版本变化: ${currentGameVersion} → ${newVersion}`)
          // 清空所有静态数据缓存
          queryClient.invalidateQueries({ queryKey: ['static'] })
        } else {
          console.log(`[AppEvents] 当前游戏版本: ${newVersion}`)
        }
        currentGameVersion = newVersion
      }
    } catch (error) {
      console.error('[AppEvents] 检查游戏版本失败:', error)
    }
  }

  const handleConnectionStateChangeDebounced = debounce({ delay: 300 }, handleConnectionStateChange)

  const handleGameFinished = () => {
    console.log('[AppEvents] 游戏结束事件')
    matchAnalysisStore.clearAllData()
  }

  const stopListening = () => {
    if (!isListeningStarted) return
    listenerGeneration += 1
    console.log(`[AppEvents] 停止 ${unlisteners.length} 个全局事件监听器...`)
    unlisteners.forEach((unlisten) => unlisten())
    unlisteners = []
    isListeningStarted = false
    console.log('[AppEvents] 全局事件监听已停止。')
  }

  const startListening = async () => {
    if (isListeningStarted) return
    isListeningStarted = true
    const generation = ++listenerGeneration
    const registered: (() => void)[] = []

    const register = async <T>(event: string, handler: (event: Event<T>) => void) => {
      const unlisten = await listen<T>(event, handler)
      if (!isListeningStarted || listenerGeneration !== generation) {
        unlisten()
        return false
      }
      registered.push(unlisten)
      return true
    }

    try {
      const registrations = [
        () => register('gameflow-phase-change', handleGameFlowPhaseChange),
        () => register('gameflow-session-changed', handleGameflowSessionChanged),
        () => register('lobby-change', handleLobbyChangeEvent),
        () => register('champ-select-session-changed', handleChampSelectSessionChanged),
        () => register('matchmaking-state-changed', handleMatchmakingStateChanged),
        () => register('connection-state-changed', handleConnectionStateChangeDebounced),
        () => register('game-finished', handleGameFinished),
        () => register('team-analysis-data', handleTeamAnalysisData)
      ]

      for (const registration of registrations) {
        if (!(await registration())) {
          registered.forEach((unlisten) => unlisten())
          return
        }
      }

      unlisteners = registered
      console.log(`[AppEvents] ${unlisteners.length} 个全局事件监听已启动`)

      // 启动时，额外尝试从后端获取一次缓存数据
      console.log('[AppEvents] 🔄 尝试从后端缓存恢复数据...')
      const { invoke } = await import('@tauri-apps/api/core')
      const cachedData = await invoke<TeamAnalysisData | null>('get_cached_analysis_data')
      if (!isListeningStarted || listenerGeneration !== generation) return
      if (cachedData) {
        console.log('[AppEvents] ✅ 找到缓存数据，正在恢复...')
        handleTeamAnalysisData({ payload: cachedData })
      }
    } catch (error) {
      console.error('[AppEvents] 启动全局事件监听失败:', error)
      registered.forEach((unlisten) => unlisten())
      if (listenerGeneration === generation) {
        unlisteners = []
        isListeningStarted = false
        listenerGeneration += 1
      }
    }
  }

  return {
    updateMatchHistory,
    updateSummonerInfo,
    startListening,
    stopListening
  }
}
