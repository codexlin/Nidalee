/**
 * 版本化的静态数据查询
 *
 * 身份目录（英雄 / 召唤师技能）：Rust 权威 → IPC → 会话投影
 * 展示目录（队列 / 符文 UI / 物品）：前端按版本缓存（localStorage）
 */

import { invoke } from '@tauri-apps/api/core'
import { computed, watchEffect } from 'vue'
import type { Ref } from 'vue'
import { useQuery } from '@tanstack/vue-query'
import type { UseQueryReturnType, useQueryClient } from '@tanstack/vue-query'
import { setChampionCatalog, setSummonerSpellCatalog } from '@/lib'
import { fetchChampionDetails, fetchCommunityDragonPerks, fetchItems, fetchQueues } from '@/lib/dataApi'
import type { CDragonQueue, CommunityDragonChampion, CommunityDragonPerk, DDragonItemsResponse } from '@/lib/dataApi'
import { setCdragonQueueNames } from '@/common/queueCatalog'
import { readVersionedCache, writeVersionedCache } from '@/shared/utils/versionedCache'

const QUEUE_CACHE_KEY = 'nidalee-static-queues'
const RUNE_PERKS_CACHE_KEY = 'nidalee-static-cdragon-perks'
const ITEMS_CACHE_KEY = 'nidalee-static-items'

const staticCatalogMetaQueryOptions = {
  queryKey: ['staticCatalogMeta'] as const,
  queryFn: () => invoke<StaticCatalogMeta>('get_static_catalog_meta'),
  staleTime: Infinity,
  gcTime: Infinity,
  refetchOnWindowFocus: false,
  retry: 3
}

/**
 * 静态包元信息（含游戏版本）。不依赖 LCU，启动即可用。
 */
export function useStaticCatalogMeta(): UseQueryReturnType<StaticCatalogMeta, Error> {
  return useQuery(staticCatalogMetaQueryOptions)
}

/**
 * 游戏版本：与 Rust 静态包同源（共享 staticCatalogMeta 查询缓存）
 */
export function useGameVersion(): UseQueryReturnType<string, Error> {
  return useQuery({
    ...staticCatalogMetaQueryOptions,
    select: (meta: StaticCatalogMeta) => meta.version
  })
}

/**
 * 英雄列表（IPC，含 Jade 600xx）
 */
export function useChampions(): UseQueryReturnType<ChampionInfo[], Error> {
  const { data: version } = useGameVersion()

  const query = useQuery({
    queryKey: computed(() => ['static', 'champions', version.value] as const),
    queryFn: async () => {
      const champions = await invoke<ChampionInfo[]>('get_all_champion_data')
      setChampionCatalog(champions)
      return champions
    },
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value),
    retry: 3
  })

  watchEffect(() => {
    if (query.data.value?.length) {
      setChampionCatalog(query.data.value)
    }
  })

  return query
}

/**
 * 召唤师技能（IPC，不再前端直连 CDragon）
 */
export function useSummonerSpells(): UseQueryReturnType<SummonerSpellInfo[], Error> {
  const { data: version } = useGameVersion()

  const query = useQuery({
    queryKey: computed(() => ['static', 'summonerSpells', version.value] as const),
    queryFn: async () => {
      const spells = await invoke<SummonerSpellInfo[]>('get_all_summoner_spell_data')
      setSummonerSpellCatalog(spells)
      return spells
    },
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value),
    retry: 3
  })

  watchEffect(() => {
    if (query.data.value?.length) {
      setSummonerSpellCatalog(query.data.value)
    }
  })

  return query
}

/**
 * 单个英雄详情（Community Dragon，按需 + 长缓存）
 */
export function useChampionDetails(
  championId: Ref<number | null>
): UseQueryReturnType<CommunityDragonChampion | null, Error> {
  const { data: version } = useGameVersion()

  return useQuery({
    queryKey: computed(() => ['static', 'championDetails', version.value ?? 'latest', championId.value] as const),
    queryFn: async () => {
      const id = championId.value
      if (!id || id <= 0) return null

      const result = await fetchChampionDetails(id)
      if (!result.success || !result.data) {
        throw new Error(result.error || '获取英雄详情失败')
      }
      return result.data
    },
    staleTime: Infinity,
    gcTime: 1000 * 60 * 60 * 24,
    refetchOnWindowFocus: false,
    enabled: computed(() => championId.value !== null && championId.value > 0)
  })
}

/**
 * Community Dragon 符文元数据（图标路径等）— 按版本持久化
 */
export function useCommunityDragonPerksQuery(): UseQueryReturnType<CommunityDragonPerk[], Error> {
  const { data: version } = useGameVersion()

  return useQuery({
    queryKey: computed(() => ['static', 'communityDragonPerks', version.value] as const),
    queryFn: async () => {
      const v = version.value
      if (!v) throw new Error('游戏版本未知，无法加载符文元数据')
      const cached = readVersionedCache<CommunityDragonPerk[]>(RUNE_PERKS_CACHE_KEY, v)
      if (cached?.length) return cached

      const res = await fetchCommunityDragonPerks()
      if (!res.success || !res.data) {
        throw new Error(res.error || '获取 Community Dragon 符文数据失败')
      }
      writeVersionedCache(RUNE_PERKS_CACHE_KEY, v, res.data)
      return res.data
    },
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value)
  })
}

/**
 * 队列目录（CDragon）— 启动可预热，按版本持久化
 */
export function useQueues(): UseQueryReturnType<CDragonQueue[], Error> {
  const { data: version } = useGameVersion()

  const query = useQuery({
    queryKey: computed(() => ['static', 'queues', version.value] as const),
    queryFn: async () => {
      const v = version.value
      if (!v) throw new Error('游戏版本未知，无法加载队列目录')
      const cached = readVersionedCache<CDragonQueue[]>(QUEUE_CACHE_KEY, v)
      if (cached?.length) return cached

      const result = await fetchQueues()
      if (!result.success || !result.data) {
        throw new Error(result.error || '获取队列数据失败')
      }
      writeVersionedCache(QUEUE_CACHE_KEY, v, result.data)
      return result.data
    },
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    enabled: computed(() => !!version.value),
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
 * 物品表（DDragon）— 首访拉取后按版本持久化，不预加载
 */
export function useItems(): UseQueryReturnType<DDragonItemsResponse, Error> {
  const { data: version } = useGameVersion()

  return useQuery({
    queryKey: computed(() => ['static', 'items', version.value] as const),
    queryFn: async () => {
      const v = version.value
      if (!v) throw new Error('游戏版本未知')

      const cached = readVersionedCache<DDragonItemsResponse>(ITEMS_CACHE_KEY, v)
      if (cached?.data) return cached

      const result = await fetchItems(v)
      if (!result.success || !result.data) {
        throw new Error(result.error || '获取物品数据失败')
      }
      writeVersionedCache(ITEMS_CACHE_KEY, v, result.data)
      return result.data
    },
    staleTime: Infinity,
    gcTime: Infinity,
    refetchOnWindowFocus: false,
    // 仅在页面调用 useItems 时拉取；不要放进启动 bootstrap
    enabled: computed(() => !!version.value)
  })
}

/**
 * 启动时 hydrate：元信息 + 英雄 + 技能 + 队列
 */
export function useBootstrapStaticData() {
  const metaQuery = useStaticCatalogMeta()
  const championsQuery = useChampions()
  const spellsQuery = useSummonerSpells()
  const queuesQuery = useQueues()

  const isReady = computed(
    () =>
      !!metaQuery.data.value &&
      !!championsQuery.data.value?.length &&
      !!spellsQuery.data.value?.length &&
      !!queuesQuery.data.value?.length
  )

  return {
    metaQuery,
    championsQuery,
    spellsQuery,
    queuesQuery,
    isReady
  }
}

/**
 * @deprecated 已拆为 useBootstrapStaticData；保留空壳避免误用 LCU 符文 hooks
 */
export function useStaticData() {
  return useBootstrapStaticData()
}

/**
 * Connected 时：后端按版本刷新静态包，再 invalidate 前端 static 查询
 */
export async function refreshStaticCatalogsOnVersionChange(queryClient: ReturnType<typeof useQueryClient>) {
  const refreshed = await invoke<boolean>('refresh_static_catalogs')
  const meta = await invoke<StaticCatalogMeta>('get_static_catalog_meta')
  queryClient.setQueryData(['staticCatalogMeta'], meta)
  if (refreshed) {
    await queryClient.invalidateQueries({ queryKey: ['static'] })
  }
  return { refreshed, meta }
}

/** OP.GG / 海克斯查询请用 `src/features/build-center/composables`（key 含 region/mode/tier） */
