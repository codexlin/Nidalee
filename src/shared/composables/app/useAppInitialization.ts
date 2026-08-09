import { getLatestVersion } from '@/lib'
import { useQueues } from '@/shared/composables/data/useVersionedData'

/**
 * 应用初始化组合式函数
 * 职责：处理应用启动时的初始化逻辑
 */
export function useAppInitialization() {
  useDeviceWebSocket()
  const dataStore = useDataStore()
  const settingsStore = useSettingsStore()
  const activityStore = useActivityStore()
  const connectionStore = useConnectionStore()

  const isInitialized = ref(false)
  const initializationError = ref<string | null>(null)

  const initializeGameVersion = async () => {
    try {
      console.log('[AppInit] 获取游戏版本...')
      const latestVersion = await getLatestVersion()

      if (latestVersion !== dataStore.gameVersion) {
        dataStore.setGameVersion(latestVersion)
        console.log('[AppInit] 游戏版本已更新:', latestVersion)
      }
    } catch (error) {
      console.error('[AppInit] 获取游戏版本失败:', error)
      activityStore.addActivity('error', '获取游戏版本失败', 'error')
    }
  }

  const initializeConnection = async () => {
    try {
      console.log('[AppInit] 初始化连接状态...')
      // 启动时强制检查一次连接，以更新持久化的陈旧状态
      await connectionStore.checkConnection()

      // 连接成功后的数据更新统一由 connectionStore.updateConnectionState('Connected') 触发，避免重复调用
    } catch (error) {
      console.error('[AppInit] 初始化连接状态失败:', error)
    }
  }

  // 预加载 CDragon 队列中文名（不依赖 LCU）
  useQueues()

  const initializeApp = async () => {
    try {
      console.log('[AppInit] 开始应用初始化...')

      // 1. 初始化主题
      settingsStore.initTheme()

      // 2. 初始化游戏版本
      await initializeGameVersion()

      // 3. 初始化连接状态
      await initializeConnection()

      isInitialized.value = true
      console.log('[AppInit] 应用初始化完成')
      activityStore.addActivity('success', '应用初始化完成', 'system')
    } catch (error) {
      console.error('[AppInit] 应用初始化失败:', error)
      initializationError.value = error instanceof Error ? error.message : '未知错误'
      activityStore.addActivity('error', '应用初始化失败', 'system')
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
    // 状态
    isInitialized,
    initializationError,

    // 方法
    initializeApp,
    cleanup,
    reinitialize
  }
}
