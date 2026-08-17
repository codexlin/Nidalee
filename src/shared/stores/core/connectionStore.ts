import { invoke } from '@tauri-apps/api/core'
import { usePersonalMatchAnalysisStore } from '@/shared/stores/features/personalMatchAnalysisStore'

export const useConnectionStore = defineStore('connection', () => {
  const hasAuth = shallowRef<boolean | undefined>(undefined)
  // 核心状态：使用 shallowRef 进行优化，因为我们只处理原始类型
  const connectionState = shallowRef<ConnectionState>('Disconnected')
  const connectionError = shallowRef<string | null>(null)

  // 计算属性
  const isConnected = computed(() => connectionState.value === 'Connected')
  const isConnecting = computed(
    () =>
      connectionState.value === ('Connecting' as ConnectionState) ||
      connectionState.value === ('ProcessFound' as ConnectionState)
  )
  const isDisconnected = computed(() => connectionState.value === 'Disconnected')
  const { updateSummonerAndMatches, cancelPendingUpdates } = useSummonerAndMatchUpdater()
  const dataStore = useDataStore()
  const sessionStore = useSessionStore()
  const gameStore = useGameStore()
  const personalMatchAnalysisStore = usePersonalMatchAnalysisStore()

  function clearAccountState() {
    cancelPendingUpdates()
    dataStore.clearAccountData()
    personalMatchAnalysisStore.setLoading(false)
    personalMatchAnalysisStore.clear()
  }

  async function checkConnection() {
    try {
      const state = await invoke<ConnectionState>('check_connection_state_command')
      await updateConnectionState(state)
      console.log('[ConnectionStore] Initial connection check:', state)
    } catch (error) {
      console.error('[ConnectionStore] Failed to check initial connection:', error)
      await updateConnectionState('Disconnected', 'Failed to communicate with backend')
    }
  }

  async function updateConnectionState(state: ConnectionState, errorMsg: string | null = null) {
    console.log(`[ConnectionStore] Updating connection state: ${state}`, errorMsg || '')
    // 如果状态没有变化，避免重复触发副作用（如重复拉取战绩）
    if (state === connectionState.value) {
      connectionError.value = errorMsg
      // Initial hydration can already say Disconnected while persisted/HMR state still contains
      // an old account. Cleanup is idempotent and must not be skipped with the state transition.
      if (state === 'Disconnected') {
        clearAccountState()
        gameStore.resetGameState()
      }
      return
    }

    connectionState.value = state
    connectionError.value = errorMsg

    switch (state) {
      case 'Connected':
        // Every transport generation starts from an empty account view. The ordered updater then
        // commits current summoner data before requesting rank and match analysis.
        clearAccountState()
        void updateSummonerAndMatches()
        sessionStore.startSession()
        break
      case 'Disconnected':
        clearAccountState()
        sessionStore.stopSession()
        gameStore.resetGameState()
        break
      case 'ProcessFound':
      case 'AuthExpired':
      case 'Unstable':
        sessionStore.stopSession()
        break
    }
  }

  function reset() {
    connectionState.value = 'Disconnected'
    connectionError.value = null
  }

  const statusText = computed(() => {
    if (connectionError.value) {
      return `连接出错: ${connectionError.value}`
    } else {
      switch (connectionState.value) {
        case 'Disconnected':
          return '等待连接到League客户端...'
        case 'ProcessFound':
          return '检测到客户端进程，正在建立连接...'
        case 'Unstable':
          return '连接不稳定，正在重试...'
        case 'AuthExpired':
          return '认证信息已过期，正在重新获取...'
        default:
          return isConnected.value ? '已连接' : '未知状态'
      }
    }
  })

  return {
    hasAuth,
    connectionState,
    connectionError,
    isConnected,
    isConnecting,
    isDisconnected,
    statusText,
    checkConnection,
    updateConnectionState,
    reset
  }
})
