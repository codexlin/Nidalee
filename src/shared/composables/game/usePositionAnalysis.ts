import { usePersonalMatchAnalysisStore } from '@/shared/stores/features/personalMatchAnalysisStore'

/**
 * 多位置分组视图 —— 只读 personalMatchAnalysisStore，不再单独请求
 */
export function usePositionAnalysis() {
  const analysisStore = usePersonalMatchAnalysisStore()

  const loading = computed(() => analysisStore.loading)
  const error = computed(() => analysisStore.error)
  const positionAnalysis = computed(() => analysisStore.multiPositionView)
  const selectedPosition = ref<PositionStats | null>(null)

  const selectPosition = (position: PositionStats) => {
    selectedPosition.value = position
  }

  const clearSelectedPosition = () => {
    selectedPosition.value = null
  }

  const reset = () => {
    selectedPosition.value = null
  }

  return {
    loading,
    error,
    positionAnalysis,
    selectedPosition,
    selectPosition,
    clearSelectedPosition,
    reset
  }
}
