import { invoke } from '@tauri-apps/api/core'

/**
 * 匹配管理 Composable
 * 职责：提供匹配相关的操作方法和状态访问
 * 数据来源：matchmakingStore（由 useAppEvents 统一监听更新）
 */
export function useMatchmaking() {
  const matchmakingStore = useMatchmakingStore()
  const matchInfo = ref<MatchInfo | null>(null)

  // 从 store 获取响应式状态
  const matchmakingState = computed(() => matchmakingStore.state)
  const searchState = computed(() => matchmakingStore.searchState)

  // 注意：不在这里处理自动跳转，统一由 useGamePhaseManager 处理游戏阶段变化
  // 避免重复跳转（matchmaking-state-changed 和 gameflow-phase-change 都会触发）
  const handleMatchmaking = async () => {
    try {
      if (searchState.value === 'Searching') {
        await invoke('stop_matchmaking')
      } else {
        await invoke('start_matchmaking')
      }
    } catch (error) {
      console.error('匹配操作失败:', error)
    }
  }

  const handleAcceptMatch = async () => {
    try {
      await invoke('accept_match')
    } catch (error) {
      console.error('接受对局失败:', error)
      throw error
    }
  }

  const handleDeclineMatch = async () => {
    try {
      await invoke('decline_match')
    } catch (error) {
      console.error('拒绝对局失败:', error)
    }
  }

  return {
    matchmakingState,
    matchInfo,
    handleMatchmaking,
    handleAcceptMatch,
    handleDeclineMatch
  }
}
