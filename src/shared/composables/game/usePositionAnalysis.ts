import { invoke } from '@tauri-apps/api/core'
import { useAnalysisSettingsStore, AnalysisMode } from '@/shared/stores/features/analysisSettingsStore'

/**
 * 分析模式枚举（与后端保持一致）
 */
export { AnalysisMode }

/**
 * 多位置分组分析 Composable
 */
export function usePositionAnalysis() {
  const loading = ref(false)
  const error = ref<string | null>(null)
  const positionAnalysis = ref<MultiPositionAnalysis | null>(null)
  const selectedPosition = ref<PositionStats | null>(null)

  /**
   * 获取多位置分组分析数据
   * @param count 对局数量
   * @param analysisMode 分析模式（前端用户选择，如果为null则使用设置中的默认模式）
   */
  const fetchPositionAnalysis = async (count: number = 20, analysisMode: AnalysisMode | null = null) => {
    const analysisSettings = useAnalysisSettingsStore()

    // 如果未指定分析模式，使用设置中的默认模式
    const mode = analysisMode || analysisSettings.config.defaultMode
    const depth = analysisSettings.config.depth

    try {
      loading.value = true
      error.value = null

      const result = await invoke<MultiPositionAnalysis>('get_match_history_with_positions', {
        count,
        analysisMode: mode,
        analysisDepth: depth
      })

      positionAnalysis.value = result
      console.log('📊 多位置分析结果:', result)

      return result
    } catch (e: unknown) {
      error.value = e instanceof Error ? e.message : '获取位置分析失败'
      console.error('❌ 获取位置分析失败:', e)
      positionAnalysis.value = null
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * 选择要查看详情的位置
   */
  const selectPosition = (position: PositionStats) => {
    selectedPosition.value = position
  }

  /**
   * 清除选中的位置
   */
  const clearSelectedPosition = () => {
    selectedPosition.value = null
  }

  /**
   * 重置所有状态
   */
  const reset = () => {
    loading.value = false
    error.value = null
    positionAnalysis.value = null
    selectedPosition.value = null
  }

  return {
    loading,
    error,
    positionAnalysis,
    selectedPosition,
    fetchPositionAnalysis,
    selectPosition,
    clearSelectedPosition,
    reset
  }
}
