/**
 * 自动版本检查触发器
 *
 * 监听连接状态变化，在 LoL 重新连接时检查版本
 * 版本变化时自动清空所有静态数据缓存
 */

import { listen } from '@tauri-apps/api/event'
import { onMounted, onUnmounted } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'

let versionCheckUnlistener: (() => void) | null = null

/**
 * 监听连接事件，自动触发版本检查
 */
export function useAutoVersionCheck() {
  const queryClient = useQueryClient()

  const startVersionCheck = async () => {
    if (versionCheckUnlistener) {
      return // 已经在监听
    }

    versionCheckUnlistener = await listen<ConnectionState>('connection-state-changed', async (event) => {
      const state = event.payload

      // 只在连接成功时检查版本
      if (state.state === 'Connected') {
        console.log('[VersionCheck] 连接成功，检查游戏版本...')

        // 记录旧版本
        const oldVersion = queryClient.getQueryData(['gameVersion'])

        // 刷新版本查询
        await queryClient.invalidateQueries({ queryKey: ['gameVersion'] })

        // 等待版本数据更新
        setTimeout(() => {
          const newVersion = queryClient.getQueryData(['gameVersion'])

          if (oldVersion !== newVersion) {
            console.log(`[VersionCheck] 版本变化: ${oldVersion} → ${newVersion}，清空静态数据缓存`)
            // 版本变化后，其他版本化的查询会自动失效并重新获取
          } else {
            console.log('[VersionCheck] 版本未变化，保持缓存')
          }
        }, 100)
      }
    })

    console.log('[VersionCheck] 版本检查监听已启动')
  }

  const stopVersionCheck = () => {
    if (versionCheckUnlistener) {
      versionCheckUnlistener()
      versionCheckUnlistener = null
      console.log('[VersionCheck] 版本检查监听已停止')
    }
  }

  // 组件挂载时启动监听
  onMounted(() => {
    startVersionCheck()
  })

  // 组件卸载时停止监听
  onUnmounted(() => {
    stopVersionCheck()
  })

  return {
    startVersionCheck,
    stopVersionCheck
  }
}

/**
 * 手动触发版本检查
 */
export function useManualVersionCheck() {
  const queryClient = useQueryClient()

  const checkVersion = async () => {
    console.log('[VersionCheck] 手动触发版本检查')
    await queryClient.invalidateQueries({ queryKey: ['gameVersion'] })
  }

  return { checkVersion }
}

/**
 * 版本变化监听器
 * 当版本变化时执行回调
 */
export function useVersionChange(callback: (newVersion: string, oldVersion: string | null) => void) {
  const queryClient = useQueryClient()
  let currentVersion: string | null = null

  const checkAndNotify = () => {
    const newVersion = queryClient.getQueryData<string>(['gameVersion'])
    if (newVersion && newVersion !== currentVersion) {
      callback(newVersion, currentVersion)
      currentVersion = newVersion
    }
  }

  // 初始化当前版本
  currentVersion = queryClient.getQueryData<string>(['gameVersion']) ?? null

  return { checkAndNotify }
}

/**
 * 连接状态类型
 */
interface ConnectionState {
  state: 'Connected' | 'Disconnected' | 'ProcessFound' | 'Unstable' | 'AuthExpired'
}
