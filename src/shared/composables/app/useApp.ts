import { invoke } from '@tauri-apps/api/core'

let nextLifecycleGeneration = 0
let activeLifecycleGeneration: number | null = null
let lcuWsCommandQueue: Promise<void> = Promise.resolve()

const enqueueLcuWsCommand = (command: 'start_lcu_ws' | 'stop_lcu_ws') => {
  const pending = lcuWsCommandQueue.then(() => invoke<void>(command))
  // 单个命令失败不能阻断后续 stop/start 清理命令。
  lcuWsCommandQueue = pending.catch(() => {})
  return pending
}

/**
 * 主应用组合式函数
 * 职责：整合各个模块，提供应用级别的状态和方法
 */
export function useApp() {
  const settingsStore = useSettingsStore()
  const appInit = useAppInitialization()
  const appEvents = useAppEvents()
  const buildPresetStore = useBuildPresetStore()
  const autoBuild = useAutoBuild()
  const { isConnected, connectionMessage, checkConnection, hasAuth } = useConnection()
  const isDark = computed(() => settingsStore.isDark)
  let instanceGeneration: number | null = null

  const stopLcuWsBestEffort = async (context: string) => {
    try {
      await enqueueLcuWsCommand('stop_lcu_ws')
    } catch (error) {
      console.warn(`[App] 停止 LCU WebSocket 失败 (${context}):`, error)
    }
  }

  onMounted(async () => {
    const generation = ++nextLifecycleGeneration
    instanceGeneration = generation
    activeLifecycleGeneration = generation

    try {
      const listenersReady = await appEvents.startListening()
      if (generation !== activeLifecycleGeneration) return
      if (!listenersReady) {
        throw new Error('Tauri 事件监听器注册未完成')
      }

      if (!buildPresetStore.isLoaded) {
        try {
          await buildPresetStore.loadFromStore()
        } catch (error) {
          console.error('[App] 构建方案加载失败，自动构建保持关闭:', error)
        }
      }
      if (generation !== activeLifecycleGeneration) return

      autoBuild.startAutoBuildWatch()
      await enqueueLcuWsCommand('start_lcu_ws')
      if (generation !== activeLifecycleGeneration) {
        autoBuild.stopAutoBuildWatch()
        return
      }

      await appInit.initializeApp()
      if (generation !== activeLifecycleGeneration) {
        autoBuild.stopAutoBuildWatch()
        return
      }
    } catch (error) {
      autoBuild.stopAutoBuildWatch()
      if (generation !== activeLifecycleGeneration) return
      appEvents.stopListening()
      appInit.cleanup()
      await stopLcuWsBestEffort('startup-error')
      console.error('[App] 应用初始化失败:', error)
    }
  })

  onUnmounted(() => {
    autoBuild.stopAutoBuildWatch()
    if (instanceGeneration === null || activeLifecycleGeneration !== instanceGeneration) return

    activeLifecycleGeneration = null
    appEvents.stopListening()
    appInit.cleanup()
    void stopLcuWsBestEffort('app-unmounted')
  })

  return {
    // 主题相关
    isDark,
    hasAuth,
    // 连接相关
    isConnected,
    connectionMessage,
    checkConnection,

    // 应用状态
    isInitialized: appInit.isInitialized,
    initializationError: appInit.initializationError,

    // 应用方法
    fetchMatchHistory: appEvents.updateMatchHistory,
    reinitialize: appInit.reinitialize
  }
}
