import { computed, ref, type MaybeRefOrGetter, toValue } from 'vue'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { fetchHextechChampionDetail, fetchHextechTierList } from '@/lib/dataApi'

const STALE_MS = 1000 * 60 * 60
const GC_MS = 1000 * 60 * 60 * 24

export function hextechTierListQueryKey() {
  return ['hextech', 'tierList'] as const
}

export function hextechDetailQueryKey(championId: number) {
  return ['hextech', 'detail', championId] as const
}

/** 适配强度榜面板（复用 OP.GG 表格） */
export function mapHextechTierListToOpgg(list: HextechTierList): OpggTierList {
  return {
    meta: {
      version: list.dataVersion,
      region: list.region,
      mode: 'hextech',
      tier: 'all'
    },
    data: list.data.map((item) => ({
      championId: item.championId,
      averageStats: {
        play: 0,
        winRate: item.winRate,
        pickRate: item.pickRate,
        banRate: 0,
        kda: 0,
        tier: item.tier ?? 0,
        rank: item.rank,
        firstPlace: null,
        totalPlace: null
      },
      positions: [],
      roles: item.roles.map((name) => ({
        name: name.toUpperCase(),
        winRate: 0,
        roleRate: 0,
        play: 0
      }))
    }))
  }
}

async function loadHextechTierList(): Promise<HextechTierList> {
  const result = await fetchHextechTierList()
  if (!result.success || !result.data) {
    throw new Error(result.error || '获取海克斯强度榜失败')
  }
  return result.data
}

async function loadHextechChampionDetail(championId: number): Promise<HextechChampionDetail> {
  const result = await fetchHextechChampionDetail(championId)
  if (!result.success || !result.data) {
    throw new Error(result.error || '获取海克斯英雄详情失败')
  }
  return result.data
}

export function useHextechData(options?: {
  tierListEnabled?: MaybeRefOrGetter<boolean>
  detailEnabled?: MaybeRefOrGetter<boolean>
}) {
  const queryClient = useQueryClient()
  const championId = ref(157)

  const tierListQuery = useQuery({
    queryKey: computed(() => hextechTierListQueryKey()),
    queryFn: loadHextechTierList,
    staleTime: STALE_MS,
    gcTime: GC_MS,
    refetchOnWindowFocus: false,
    enabled: computed(() => toValue(options?.tierListEnabled) ?? true)
  })

  const detailQuery = useQuery({
    queryKey: computed(() => hextechDetailQueryKey(championId.value)),
    queryFn: () => loadHextechChampionDetail(championId.value),
    staleTime: STALE_MS,
    gcTime: GC_MS,
    refetchOnWindowFocus: false,
    enabled: computed(() => (toValue(options?.detailEnabled) ?? false) && championId.value > 0)
  })

  const tierListAsOpgg = computed((): OpggTierList | null => {
    const list = tierListQuery.data.value
    return list ? mapHextechTierListToOpgg(list) : null
  })

  const loading = computed(
    () =>
      (toValue(options?.tierListEnabled) && tierListQuery.isFetching.value) ||
      (toValue(options?.detailEnabled) && detailQuery.isFetching.value)
  )

  const error = computed(() => {
    if (toValue(options?.detailEnabled) && detailQuery.error.value) {
      return (detailQuery.error.value as Error).message
    }
    if (toValue(options?.tierListEnabled) && tierListQuery.error.value) {
      return (tierListQuery.error.value as Error).message
    }
    return null
  })

  const selectChampion = (id: number) => {
    championId.value = id
  }

  const refreshTierList = () => queryClient.invalidateQueries({ queryKey: hextechTierListQueryKey() })

  const refreshDetail = () => queryClient.invalidateQueries({ queryKey: ['hextech', 'detail'] })

  const refreshCurrent = () => {
    if (toValue(options?.detailEnabled)) return refreshDetail()
    return refreshTierList()
  }

  return {
    championId,
    tierList: computed(() => tierListQuery.data.value ?? null),
    tierListAsOpgg,
    detail: computed(() => detailQuery.data.value ?? null),
    loading,
    error,
    tierListQuery,
    detailQuery,
    selectChampion,
    refreshTierList,
    refreshDetail,
    refreshCurrent
  }
}
