/**
 * 英雄查询 composable
 * 使用 TanStack Query + 版本化缓存
 */

import { invoke } from '@tauri-apps/api/core'
import { computed, ref, type Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'

interface QueryResult<T> {
  data: Ref<T | null>
  isLoading: Ref<boolean>
  error: Ref<string | null>
  refetch: () => Promise<void>
}

/**
 * 获取当前游戏版本（内部使用）
 */
function useGameVersion() {
  return useQuery({
    queryKey: ['gameVersion'],
    queryFn: () => invoke<string>('get_game_version'),
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
  })
}

/**
 * 查询所有英雄列表（版本化缓存）
 * 使用游戏版本号作为缓存 key，版本变化时自动刷新
 */
export function useChampionSummaryQuery(): QueryResult<any[]> {
  const { data: version } = useGameVersion()

  const query = useQuery({
    queryKey: computed(() => ['static', 'champions', version.value] as const),
    queryFn: () => invoke<any[]>('get_all_champion_data'),
    staleTime: Infinity, // 版本不变时，数据永远新鲜
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value),
  })

  return {
    data: computed(() => query.data.value ?? null),
    isLoading: computed(() => query.isLoading.value),
    error: computed(() => query.error.value?.message ?? null),
    refetch: async () => {
      await query.refetch()
    },
  }
}

/**
 * 查询单个英雄的详细信息（包括皮肤）
 * 从 Community Dragon API 获取
 * 使用静态缓存（英雄详情很少变化）
 */
export function useChampionDetailsQuery(championId: Ref<number | null>): QueryResult<any> {
  const query = useQuery({
    queryKey: computed(() => ['championDetails', championId.value] as const),
    queryFn: async () => {
      if (!championId.value || championId.value <= 0) {
        return null
      }

      const url = `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/champions/${championId.value}.json`
      const response = await fetch(url)

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      return response.json()
    },
    staleTime: 1000 * 60 * 60, // 1 小时
    gcTime: 1000 * 60 * 60 * 24, // 24 小时
    refetchOnWindowFocus: false,
    enabled: computed(() => championId.value !== null && championId.value > 0),
  })

  return {
    data: computed(() => query.data.value ?? null),
    isLoading: computed(() => query.isLoading.value),
    error: computed(() => query.error.value?.message ?? null),
    refetch: async () => {
      await query.refetch()
    },
  }
}
