import { computed, ref, type MaybeRefOrGetter, toValue } from 'vue'
import { useQuery, useQueryClient } from '@tanstack/vue-query'
import { fetchOpggChampionBuild, fetchOpggTierList } from '@/lib/dataApi'
import { OPGG_MODES, buildRequestPosition } from '../types/modes'
import { OPGG_TIER_OPTIONS } from '@/shared/utils/opggTier'

export interface OpggConfig {
  region: string
  mode: string
  tier: string
  position: string
  championId: number
}

const STALE_MS = 1000 * 60 * 60
const GC_MS = 1000 * 60 * 60 * 24

export function opggTierListQueryKey(region: string, mode: string, tier: string) {
  return ['opgg', 'tierList', region, mode, tier] as const
}

export function opggBuildQueryKey(region: string, mode: string, tier: string, championId: number, position: string) {
  return ['opgg', 'build', region, mode, tier, championId, position] as const
}

async function loadOpggTierList(region: string, mode: string, tier: string): Promise<OpggTierList> {
  const result = await fetchOpggTierList({ region, mode, tier })
  if (!result.success || !result.data) {
    throw new Error(result.error || '获取强度榜失败')
  }
  return result.data
}

async function loadOpggChampionBuild(config: OpggConfig): Promise<OpggChampionBuild> {
  const position = buildRequestPosition(config.mode, config.position) ?? undefined
  const result = await fetchOpggChampionBuild({
    region: config.region,
    mode: config.mode,
    champion_id: config.championId,
    position,
    tier: config.tier
  })
  if (!result.success || !result.data) {
    throw new Error(result.error || '获取英雄详细数据失败')
  }
  return result.data
}

export function useOpggData(options?: {
  /** 浏览强度榜时启用 */
  tierListEnabled?: MaybeRefOrGetter<boolean>
  /** 查看方案详情时启用 */
  buildEnabled?: MaybeRefOrGetter<boolean>
}) {
  const queryClient = useQueryClient()

  const config = ref<OpggConfig>({
    region: 'kr',
    mode: 'ranked',
    tier: 'emerald_plus',
    position: 'MID',
    championId: 157
  })

  const regions = [
    { value: 'global', label: '全球' },
    { value: 'kr', label: '韩服' },
    { value: 'na', label: '北美' }
  ]

  const modes = OPGG_MODES.map(({ value, label }) => ({ value, label }))

  const tiers = [...OPGG_TIER_OPTIONS]

  const positions = [
    { value: 'TOP', label: '上单' },
    { value: 'JUNGLE', label: '打野' },
    { value: 'MID', label: '中单' },
    { value: 'ADC', label: '下路' },
    { value: 'SUPPORT', label: '辅助' }
  ]

  const buildPositionKey = computed(() => buildRequestPosition(config.value.mode, config.value.position) ?? '')

  const tierListQuery = useQuery({
    queryKey: computed(() => opggTierListQueryKey(config.value.region, config.value.mode, config.value.tier)),
    queryFn: () => loadOpggTierList(config.value.region, config.value.mode, config.value.tier),
    staleTime: STALE_MS,
    gcTime: GC_MS,
    refetchOnWindowFocus: false,
    enabled: computed(() => toValue(options?.tierListEnabled) ?? true)
  })

  const buildQuery = useQuery({
    queryKey: computed(() =>
      opggBuildQueryKey(
        config.value.region,
        config.value.mode,
        config.value.tier,
        config.value.championId,
        buildPositionKey.value
      )
    ),
    queryFn: () => loadOpggChampionBuild(config.value),
    staleTime: STALE_MS,
    gcTime: GC_MS,
    refetchOnWindowFocus: false,
    enabled: computed(() => (toValue(options?.buildEnabled) ?? false) && config.value.championId > 0)
  })

  const loading = computed(
    () =>
      (toValue(options?.tierListEnabled) && tierListQuery.isFetching.value) ||
      (toValue(options?.buildEnabled) && buildQuery.isFetching.value)
  )

  const error = computed(() => {
    if (toValue(options?.buildEnabled) && buildQuery.error.value) {
      return (buildQuery.error.value as Error).message
    }
    if (toValue(options?.tierListEnabled) && tierListQuery.error.value) {
      return (tierListQuery.error.value as Error).message
    }
    return null
  })

  const refreshTierList = () =>
    queryClient.invalidateQueries({
      queryKey: ['opgg', 'tierList']
    })

  const refreshBuild = () =>
    queryClient.invalidateQueries({
      queryKey: ['opgg', 'build']
    })

  const refreshCurrent = () => {
    if (toValue(options?.buildEnabled)) return refreshBuild()
    return refreshTierList()
  }

  return {
    config,
    regions,
    modes,
    tiers,
    positions,
    tierList: computed(() => tierListQuery.data.value ?? null),
    championBuild: computed(() => buildQuery.data.value ?? null),
    loading,
    error,
    tierListQuery,
    buildQuery,
    refreshTierList,
    refreshBuild,
    refreshCurrent
  }
}
