import { invoke } from '@tauri-apps/api/core'
import { ref, watch, type Ref } from 'vue'

/**
 * 英雄查询 composable
 * 使用后端已有的英雄数据接口
 */

interface QueryResult<T> {
  data: Ref<T | null>
  isLoading: Ref<boolean>
  error: Ref<string | null>
  refetch: () => Promise<void>
}

/**
 * 查询所有英雄列表（使用后端 get_all_champion_data）
 * @returns QueryResult<any[]>
 */
export function useChampionSummaryQuery(): QueryResult<any[]> {
  const data = ref<any[] | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const fetchData = async () => {
    isLoading.value = true
    error.value = null
    try {
      // 使用后端已有的 get_all_champion_data 接口
      const result = await invoke<any[]>('get_all_champion_data')
      data.value = result
    } catch (e: any) {
      error.value = e?.message || '获取英雄列表失败'
      console.error('[useChampionSummaryQuery] 查询失败:', e)
    } finally {
      isLoading.value = false
    }
  }

  // 自动执行一次查询
  fetchData()

  return {
    data,
    isLoading,
    error,
    refetch: fetchData
  }
}

/**
 * 查询单个英雄的详细信息（包括皮肤）
 * 从 Community Dragon API 获取
 * @param championId 英雄ID (响应式)
 * @returns QueryResult<any>
 */
export function useChampionDetailsQuery(championId: Ref<number | null>): QueryResult<any> {
  const data = ref<any | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const fetchData = async () => {
    if (!championId.value || championId.value <= 0) {
      data.value = null
      return
    }

    isLoading.value = true
    error.value = null
    try {
      // 从 Community Dragon 获取英雄完整数据（包括皮肤）
      const url = `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/champions/${championId.value}.json`
      const response = await fetch(url)

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      const result = await response.json()
      data.value = result
    } catch (e: any) {
      error.value = e?.message || '获取英雄详情失败'
      console.error('[useChampionDetailsQuery] 查询失败:', e)
      data.value = null
    } finally {
      isLoading.value = false
    }
  }

  // 监听 championId 变化，自动重新查询
  watch(
    championId,
    () => {
      fetchData()
    },
    { immediate: true }
  )

  return {
    data,
    isLoading,
    error,
    refetch: fetchData
  }
}
