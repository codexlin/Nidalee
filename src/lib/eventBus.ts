/**
 * 事件总线 - 用于组件间通信，减少直接轮询
 * 实现发布-订阅模式，提高系统解耦性
 */

type EventCallback = (data?: unknown) => void

class EventBus {
  private listeners = new Map<string, EventCallback[]>()
  private onceListeners = new Map<string, EventCallback[]>()

  /**
   * 订阅事件
   * @param event 事件名称
   * @param callback 回调函数
   */
  on(event: string, callback: EventCallback): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, [])
    }
    this.listeners.get(event)!.push(callback)
  }

  /**
   * 订阅一次性事件（触发后自动取消订阅）
   * @param event 事件名称
   * @param callback 回调函数
   */
  once(event: string, callback: EventCallback): void {
    if (!this.onceListeners.has(event)) {
      this.onceListeners.set(event, [])
    }
    this.onceListeners.get(event)!.push(callback)
  }

  /**
   * 取消订阅事件
   * @param event 事件名称
   * @param callback 要移除的回调函数
   */
  off(event: string, callback: EventCallback): void {
    const listeners = this.listeners.get(event)
    if (listeners) {
      const index = listeners.indexOf(callback)
      if (index > -1) {
        listeners.splice(index, 1)
      }
    }

    const onceListeners = this.onceListeners.get(event)
    if (onceListeners) {
      const index = onceListeners.indexOf(callback)
      if (index > -1) {
        onceListeners.splice(index, 1)
      }
    }
  }

  /**
   * 发布事件
   * @param event 事件名称
   * @param data 事件数据
   */
  emit(event: string, data?: unknown): void {
    // 触发普通监听器
    const listeners = this.listeners.get(event)
    if (listeners) {
      listeners.forEach((callback) => {
        try {
          callback(data)
        } catch (error) {
          console.error(`[EventBus] 事件 ${event} 回调执行失败:`, error)
        }
      })
    }

    // 触发一次性监听器并移除
    const onceListeners = this.onceListeners.get(event)
    if (onceListeners) {
      onceListeners.forEach((callback) => {
        try {
          callback(data)
        } catch (error) {
          console.error(`[EventBus] 一次性事件 ${event} 回调执行失败:`, error)
        }
      })
      this.onceListeners.delete(event)
    }
  }

  /**
   * 清除所有监听器
   */
  clear(): void {
    this.listeners.clear()
    this.onceListeners.clear()
  }

  /**
   * 获取事件监听器数量
   * @param event 事件名称
   */
  listenerCount(event: string): number {
    const listeners = this.listeners.get(event)?.length || 0
    const onceListeners = this.onceListeners.get(event)?.length || 0
    return listeners + onceListeners
  }

  /**
   * 获取所有事件名称
   */
  eventNames(): string[] {
    const eventNames = new Set<string>()
    this.listeners.forEach((_, event) => eventNames.add(event))
    this.onceListeners.forEach((_, event) => eventNames.add(event))
    return Array.from(eventNames)
  }
}

// 创建全局事件总线实例
export const eventBus = new EventBus()

// 预定义的事件类型
export const GameEvents = {
  // 游戏阶段事件
  GAME_PHASE_CHANGED: 'game-phase-changed',
  GAME_STARTED: 'game-started',
  GAME_ENDED: 'game-ended',

  // LiveClient 事件
  LIVECLIENT_AVAILABLE: 'liveclient-available',
  LIVECLIENT_DATA_FETCHED: 'liveclient-data-fetched',
  LIVECLIENT_ERROR: 'liveclient-error',

  // 数据更新事件
  PLAYER_DATA_UPDATED: 'player-data-updated',
  MATCH_HISTORY_UPDATED: 'match-history-updated',
  CHAMPION_DATA_LOADED: 'champion-data-loaded',

  // 连接状态事件
  CONNECTION_STATE_CHANGED: 'connection-state-changed',
  LCU_CONNECTED: 'lcu-connected',
  LCU_DISCONNECTED: 'lcu-disconnected',

  // 性能事件
  PERFORMANCE_METRIC: 'performance-metric',
  SLOW_OPERATION: 'slow-operation'
} as const

// 类型定义
export type GameEventType = (typeof GameEvents)[keyof typeof GameEvents]

// 导出类型
export type { EventCallback }
export default eventBus
