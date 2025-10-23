import { invoke } from '@tauri-apps/api/core'

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
   * @param queueId 队列ID (420=单排, 440=灵活组排, null=所有模式)
   */
  const fetchPositionAnalysis = async (count: number = 20, queueId: number | null = null) => {
    try {
      loading.value = true
      error.value = null

      const result = await invoke<MultiPositionAnalysis>('get_match_history_with_positions', {
        count,
        queueId
      })

      positionAnalysis.value = result
      console.log('📊 多位置分析结果:', result)

      return result
    } catch (e: any) {
      error.value = e?.message || '获取位置分析失败'
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
