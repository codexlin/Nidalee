import { useWebSocket } from '@vueuse/core'
import { invoke } from '@tauri-apps/api/core'

export function useDeviceWebSocket() {
  const router = useRouter()
  const ws = ref<ReturnType<typeof useWebSocket> | null>(null)
  const deviceId = ref<string | null>(null)
  const status = ref<'CONNECTING' | 'OPEN' | 'CLOSED' | 'ERROR'>('CONNECTING')
  const lastServerMsg = ref<string | null>(null)
  const lastError = ref<string | null>(null)

  const connectionStore = useConnectionStore()

  const handleServerMessage = (event: MessageEvent) => {
    lastServerMsg.value = String(event.data)
    try {
      const obj = JSON.parse(String(event.data))
      if (obj && typeof obj === 'object') {
        console.log('WebSocket响应:', obj)
        if ('code' in obj && typeof obj.code === 'number') {
          if (obj.code === 403) {
            connectionStore.hasAuth = false
            void router.replace('/forbidden')
          } else if (obj.code >= 200 && obj.code < 300) {
            connectionStore.hasAuth = true
            if (router.currentRoute.value.path === '/forbidden') {
              void router.replace('/')
            }
          }
        }
      }
    } catch {
      console.log('WebSocket响应:', event.data)
    }
  }

  onMounted(async () => {
    console.log('useDeviceWebSocket', import.meta.env.VITE_WS_BASE_URL)
    try {
      const hash = await invoke<string>('get_machine_hash')
      deviceId.value = hash

      const wsUrl = `${import.meta.env.VITE_WS_BASE_URL}/${hash}`

      const wsInstance = useWebSocket(wsUrl, {
        onConnected() {
          status.value = 'OPEN'
          lastError.value = null
          console.log('WebSocket连接成功')
        },
        onDisconnected() {
          status.value = 'CLOSED'
          console.log('WebSocket连接关闭')
        },
        onError(_socket, event) {
          status.value = 'ERROR'
          lastError.value = `WebSocket 错误: ${String(event)}`
          console.log('WebSocket错误:', event)
        },
        onMessage(_socket, event) {
          handleServerMessage(event)
        },
        autoReconnect: {
          retries: 10,
          delay: 3000,
          onFailed() {
            lastError.value = 'WebSocket 多次重连失败'
          }
        },
        immediate: true
      })

      ws.value = wsInstance
    } catch (e) {
      lastError.value = '获取设备ID或连接WebSocket失败: ' + (e as unknown)?.toString()
      status.value = 'ERROR'
    }
  })

  onBeforeUnmount(() => {
    ws.value?.close()
    ws.value = null
  })

  return {
    ws,
    deviceId,
    status,
    lastServerMsg,
    lastError
  }
}
