/**
 * 版本化的静态数据查询
 *
 * 使用游戏版本号作为缓存 key，版本变化时自动失效
 * 避免不必要的数据请求，只在游戏更新时才重新获取
 */

import { invoke } from '@tauri-apps/api/core'
import { computed, type Ref, watchEffect } from 'vue'
import { useQuery, type UseQueryReturnType } from '@tanstack/vue-query'
import {
  fetchCommunityDragonPerks,
  fetchQueues,
  type CDragonQueue,
  type CommunityDragonPerk
} from '@/lib/dataApi'
import { setCdragonQueueNames } from '@/common/queueCatalog'

/**
 * 游戏版本查询
 */
export function useGameVersion(): UseQueryReturnType<string, Error> {
  return useQuery({
    queryKey: ['gameVersion'],
    queryFn: () => invoke<string>('get_game_version'),
    staleTime: Infinity, // 版本号不会过期
    gcTime: Infinity, // 永久保留版本信息
    refetchOnWindowFocus: false,
    retry: 1
  })
}

/**
 * 英雄列表查询（版本化）
 */
export function useChampions(): UseQueryReturnType<ChampionInfo[], Error> {
  const { data: version } = useGameVersion()

  return useQuery({
    // 版本号作为 key 的一部分，版本变化自动失效
    queryKey: computed(() => ['static', 'champions', version.value] as const),
    queryFn: () => invoke<ChampionInfo[]>('get_all_champion_data'),
    staleTime: Infinity, // 版本不变时，数据永远新鲜
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value)
  })
}

/**
 * 符文样式查询（版本化）
 */
export function useRuneStyles(): UseQueryReturnType<unknown, Error> {
  const { data: version } = useGameVersion()

  return useQuery({
    queryKey: computed(() => ['static', 'runes', version.value] as const),
    queryFn: () => invoke<unknown>('get_lcu_rune_styles'),
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value)
  })
}

/**
 * 符文详情查询（版本化）
 */
export function usePerks(): UseQueryReturnType<unknown, Error> {
  const { data: version } = useGameVersion()

  return useQuery({
    queryKey: computed(() => ['static', 'perks', version.value] as const),
    queryFn: () => invoke<unknown>('get_lcu_perks'),
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value)
  })
}

/**
 * Community Dragon 符文元数据（图标路径等）
 * OP.GG RunesCard 用其解析 perk 图标 URL
 */
export function useCommunityDragonPerksQuery(): UseQueryReturnType<CommunityDragonPerk[], Error> {
  return useQuery({
    queryKey: ['static', 'communityDragonPerks'] as const,
    queryFn: async () => {
      const res = await fetchCommunityDragonPerks()
      if (!res.success || !res.data) {
        throw new Error(res.error || '获取 Community Dragon 符文数据失败')
      }
      return res.data
    },
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false
  })
}

/**
 * 符文图标查询（版本化）
 */
export function usePerkIcons(): UseQueryReturnType<Record<number, string>, Error> {
  const { data: version } = useGameVersion()

  return useQuery({
    queryKey: computed(() => ['static', 'perkIcons', version.value] as const),
    queryFn: () => invoke<Record<number, string>>('get_lcu_perk_icon'),
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value)
  })
}

/**
 * 召唤师技能查询（版本化）
 */
export function useSummonerSpells(): UseQueryReturnType<unknown[], Error> {
  const { data: version } = useGameVersion()

  return useQuery({
    queryKey: computed(() => ['static', 'spells', version.value] as const),
    queryFn: () => invoke<unknown[]>('get_all_summoner_spell_data'),
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value)
  })
}

/**
 * 当前符文页查询（动态数据，不版本化）
 */
export function useCurrentRunePage(): UseQueryReturnType<unknown, Error> {
  return useQuery({
    queryKey: ['currentRunePage'],
    queryFn: () => invoke<unknown>('get_current_rune_page'),
    staleTime: 1000 * 60, // 1 分钟
    refetchOnWindowFocus: false
  })
}

/**
 * 队列目录（Community Dragon zh_cn）
 * 不依赖 LCU 连接，启动即可拉取并写入名称缓存
 */
export function useQueues(): UseQueryReturnType<CDragonQueue[], Error> {
  const query = useQuery({
    queryKey: ['static', 'queues', 'cdragon-zh_cn'] as const,
    queryFn: async () => {
      const result = await fetchQueues()
      if (!result.success || !result.data) {
        throw new Error(result.error || '获取队列数据失败')
      }
      return result.data
    },
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    retry: 2
  })

  watchEffect(() => {
    const queues = query.data.value
    if (!queues?.length) return
    setCdragonQueueNames(queues.map((q) => ({ id: q.id, name: q.name })))
  })

  return query
}

/**
 * 组合 hook：一次性获取所有静态数据
 */
export function useStaticData() {
  const versionQuery = useGameVersion()
  const championsQuery = useChampions()
  const runeStylesQuery = useRuneStyles()
  const perksQuery = usePerks()
  const perkIconsQuery = usePerkIcons()
  const spellsQuery = useSummonerSpells()

  const isLoading = computed(
    () =>
      versionQuery.isLoading.value ||
      championsQuery.isLoading.value ||
      runeStylesQuery.isLoading.value ||
      perksQuery.isLoading.value ||
      perkIconsQuery.isLoading.value ||
      spellsQuery.isLoading.value
  )

  const error = computed(
    () =>
      versionQuery.error.value ||
      championsQuery.error.value ||
      runeStylesQuery.error.value ||
      perksQuery.error.value ||
      perkIconsQuery.error.value ||
      spellsQuery.error.value
  )

  const isReady = computed(
    () =>
      !!versionQuery.data.value &&
      !!championsQuery.data.value &&
      !!runeStylesQuery.data.value &&
      !!perksQuery.data.value &&
      !!perkIconsQuery.data.value &&
      !!spellsQuery.data.value
  )

  return {
    versionQuery,
    championsQuery,
    runeStylesQuery,
    perksQuery,
    perkIconsQuery,
    spellsQuery,
    isLoading,
    error,
    isReady
  }
}

/**
 * 手动刷新所有静态数据（用于检测到版本变化时）
 */
export function useRefreshStaticData() {
  const queryClient = useQueryClient()

  return async () => {
    // 先刷新版本号
    await queryClient.invalidateQueries({ queryKey: ['gameVersion'] })
    // 版本变化后，其他查询会自动失效并重新获取
  }
}

/**
 * 获取单个英雄信息（从缓存中查找）
 */
export function useChampionById(championId: Ref<number | null>) {
  const { data: champions } = useChampions()

  const champion = computed(() => {
    if (!championId.value || !champions.value) return null
    return (
      champions.value.find((c: ChampionInfo) => c.id === championId.value) ||
      champions.value.find((c: ChampionInfo) => c.alias === String(championId.value))
    )
  })

  return { champion }
}

/**
 * 获取单个英雄信息（按名字）
 */
export function useChampionByName(name: Ref<string | null>) {
  const { data: champions } = useChampions()

  const champion = computed(() => {
    if (!name.value || !champions.value) return null
    const searchName = name.value.toLowerCase()
    return champions.value.find((c: ChampionInfo) => {
      const cName = c.name?.toLowerCase() || ''
      const cAlias = c.alias?.toLowerCase() || ''
      return cName === searchName || cAlias === searchName
    })
  })

  return { champion }
}

/**
 * 按需加载的 OP.GG 数据（带版本号）
 */
export function useOpggBuild(
  championId: Ref<number>,
  position: Ref<string>,
  options?: {
    enabled?: Ref<boolean>
  }
) {
  const { data: version } = useGameVersion()

  return useQuery({
    queryKey: computed(() => ['opgg', 'build', championId.value, position.value, version.value] as const),
    queryFn: () =>
      invoke('get_opgg_champion_build', {
        championId: championId.value,
        position: position.value || 'MIDDLE',
        region: 'cn',
        mode: 'ranked',
        tier: 'platinum_plus'
      }),
    staleTime: 1000 * 60 * 60, // 1 小时（OP.GG 数据可能更新）
    gcTime: 1000 * 60 * 60 * 24, // 24 小时
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value && championId.value > 0 && (options?.enabled?.value ?? true))
  })
}

/**
 * OP.GG 英雄强度榜（带版本号）
 */
export function useOpggTierList() {
  const { data: version } = useGameVersion()

  return useQuery({
    queryKey: computed(() => ['opgg', 'tierList', version.value] as const),
    queryFn: () =>
      invoke('get_opgg_tier_list', {
        region: 'cn',
        mode: 'ranked',
        tier: 'platinum_plus'
      }),
    staleTime: 1000 * 60 * 60, // 1 小时
    gcTime: 1000 * 60 * 60 * 24,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value)
  })
}

// 导入 QueryClient 类型
import { useQueryClient } from '@tanstack/vue-query'
