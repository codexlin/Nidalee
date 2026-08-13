/**
 * 连接状态管理组合式函数
 * 职责：提供连接状态的响应式接口，不直接处理事件
 */
export function useConnection() {
  const connectionStore = useConnectionStore()

  const isConnected = computed(() => connectionStore.isConnected)
  const connectionState = computed(() => connectionStore.connectionState)
  const connectionError = computed(() => connectionStore.connectionError)
  const hasAuth = computed(() => connectionStore.hasAuth)
  const connectionMessage = computed(() => connectionStore.statusText)

  const checkConnection = () => {
    return connectionStore.checkConnection()
  }

  const clearConnection = () => {
    connectionStore.reset()
  }

  return {
    // 状态
    hasAuth,
    isConnected,
    connectionState,
    connectionError,
    connectionMessage,

    // 方法
    checkConnection,
    clearConnection
  }
}
