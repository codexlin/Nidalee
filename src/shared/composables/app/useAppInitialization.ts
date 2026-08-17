import { useBootstrapStaticData } from '@/shared/composables/data/useVersionedData'

/**
 * 应用初始化组合式函数
 * 职责：处理应用启动时的初始化逻辑
 */
export function useAppInitialization() {
  useDeviceWebSocket()
  const dataStore = useDataStore()
  const settingsStore = useSettingsStore()
  const connectionStore = useConnectionStore()

  const isInitialized = ref(false)
  const initializationError = ref<string | null>(null)

  // 启动 hydrate：Rust 静态包元信息 + 英雄/技能 IPC + 队列（按版本缓存）
  const { metaQuery, isReady: staticReady } = useBootstrapStaticData()

  watchEffect(() => {
    const version = metaQuery.data.value?.version
    if (version && version !== dataStore.gameVersion) {
      dataStore.setGameVersion(version)
    }
  })

  const initializeConnection = async () => {
    try {
      console.log('[AppInit] 初始化连接状态...')
      await connectionStore.checkConnection()
    } catch (error) {
      console.error('[AppInit] 初始化连接状态失败:', error)
    }
  }

  const initializeApp = async () => {
    try {
      console.log('[AppInit] 开始应用初始化...')

      settingsStore.initTheme()

      // 静态目录由 useBootstrapStaticData 后台拉取；此处只等连接
      await initializeConnection()

      isInitialized.value = true
      console.log('[AppInit] 应用初始化完成', {
        staticReady: staticReady.value,
        version: metaQuery.data.value?.version
      })
    } catch (error) {
      console.error('[AppInit] 应用初始化失败:', error)
      initializationError.value = error instanceof Error ? error.message : '未知错误'
    }
  }

  const cleanup = () => {
    console.log('[AppInit] 清理应用资源...')
    isInitialized.value = false
    initializationError.value = null
  }

  const reinitialize = async () => {
    console.log('[AppInit] 重新初始化应用...')
    cleanup()
    await initializeApp()
  }

  return {
    isInitialized,
    initializationError,
    initializeApp,
    cleanup,
    reinitialize
  }
}
