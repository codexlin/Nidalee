/**
 * 匹配状态 Store
 * 职责：管理匹配相关的状态数据
 */
export const useMatchmakingStore = defineStore('matchmaking', () => {
  // 状态
  const state = ref<MatchmakingState | null>(null)

  // 计算属性
  const searchState = computed(() => state.value?.searchState || null)
  const isSearching = computed(() => searchState.value === 'Searching')
  const estimatedQueueTime = computed(() => state.value?.estimatedQueueTime || 0)
  // 后端暂未提供排队计时字段，保留占位供后续接入
  const timeInQueue = computed(() => 0)

  // 操作方法
  function updateState(newState: MatchmakingState) {
    console.log('[MatchmakingStore] 更新匹配状态:', newState)
    state.value = newState
  }

  function clearState() {
    console.log('[MatchmakingStore] 清除匹配状态')
    state.value = null
  }

  return {
    // 状态
    state,

    // 计算属性
    searchState,
    isSearching,
    estimatedQueueTime,
    timeInQueue,

    // 方法
    updateState,
    clearState
  }
})
