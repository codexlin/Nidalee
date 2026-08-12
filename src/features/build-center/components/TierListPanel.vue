<template>
  <div
    :class="
      embedded
        ? 'flex h-full min-h-0 min-w-0 flex-1 flex-col overflow-hidden'
        : 'flex flex-col overflow-hidden rounded-xl border border-border bg-card text-card-foreground shadow-sm'
    "
  >
    <div class="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b border-border/60 px-4 py-3 sm:px-5">
      <div class="flex min-w-0 items-baseline gap-2">
        <h2 class="text-lg font-medium leading-tight">强度榜</h2>
        <span v-if="tierList?.meta?.version" class="truncate text-sm tabular-nums text-muted-foreground">
          v{{ tierList.meta.version }}
          <template v-if="mode === 'urf'"> · 限时模式可能停更</template>
          · 点英雄或行查看方案
        </span>
      </div>
      <div v-if="usesLanePosition(mode)" class="flex flex-wrap gap-0.5 rounded-full surface-inset p-0.5">
        <button
          v-for="pos in positionFilters"
          :key="pos.value"
          type="button"
          class="rounded-full px-3 py-1.5 text-sm font-medium transition-colors"
          :class="
            activePosition === pos.value ? 'bg-primary/15 text-primary' : 'text-muted-foreground hover:text-foreground'
          "
          @click="activePosition = pos.value"
        >
          {{ pos.label }}
        </button>
      </div>
    </div>

    <!-- 骨架加载 -->
    <div v-if="loading" class="min-h-0 flex-1 overflow-hidden">
      <table class="w-full text-sm">
        <thead class="border-b border-border/50">
          <tr class="text-left text-sm text-muted-foreground">
            <th class="w-12 px-4 py-2.5 font-medium">#</th>
            <th class="px-3 py-2.5 font-medium">英雄</th>
            <th class="w-14 px-3 py-2.5 font-medium">Tier</th>
            <th class="w-[4.5rem] px-3 py-2.5 font-medium">{{ primaryRateLabel(mode) }}</th>
            <th class="w-[4.5rem] px-3 py-2.5 font-medium">选取</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="i in 8" :key="i" class="border-b border-border/30">
            <td class="px-4 py-2.5">
              <div class="h-3.5 w-4 animate-pulse rounded bg-muted" />
            </td>
            <td class="px-3 py-2.5">
              <div class="flex items-center gap-3">
                <div class="size-9 animate-pulse rounded-lg bg-muted" />
                <div class="h-3.5 w-24 animate-pulse rounded bg-muted" />
              </div>
            </td>
            <td class="px-3 py-2.5">
              <div class="h-6 w-8 animate-pulse rounded-md bg-muted" />
            </td>
            <td class="px-3 py-2.5">
              <div class="h-3.5 w-12 animate-pulse rounded bg-muted" />
            </td>
            <td class="px-3 py-2.5">
              <div class="h-3.5 w-12 animate-pulse rounded bg-muted" />
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div
      v-else-if="!rows.length"
      class="flex min-h-0 flex-1 flex-col items-center justify-center gap-1 px-4 text-center"
    >
      <p class="text-sm text-muted-foreground">
        {{ mode === 'hextech' ? '暂无海克斯强度榜' : '暂无强度榜数据' }}
      </p>
      <p class="text-xs text-muted-foreground">
        {{ mode === 'hextech' ? '稍后重试，或切换到 OP.GG 查看其它模式' : '调整区域 / 模式后刷新，或从上方切换数据源' }}
      </p>
    </div>

    <div v-else class="min-h-0 flex-1 overflow-auto">
      <table class="w-full text-sm tabular-nums">
        <thead class="sticky top-0 z-10 border-b border-border/50 bg-card">
          <tr class="text-left text-sm text-muted-foreground">
            <th class="w-12 px-2 py-2.5 sm:px-4">
              <button
                type="button"
                class="inline-flex items-center gap-0.5 font-medium hover:text-foreground"
                :class="sortKey === 'rank' ? 'text-foreground' : 'text-muted-foreground'"
                title="按排名排序"
                @click="toggleSort('rank')"
              >
                #
                <ChevronsUpDown v-if="sortKey !== 'rank'" class="size-3.5 opacity-50" />
                <ChevronDown
                  v-else
                  class="size-3.5 transition-transform"
                  :class="sortDir === 'asc' ? '' : 'rotate-180'"
                />
              </button>
            </th>
            <th class="px-3 py-2.5 font-medium">英雄</th>
            <th class="w-14 px-3 py-2.5 font-medium">Tier</th>
            <th class="w-[4.5rem] px-3 py-2.5">
              <button
                type="button"
                class="inline-flex items-center gap-0.5 font-medium hover:text-foreground"
                :class="sortKey === 'winRate' ? 'text-foreground' : 'text-muted-foreground'"
                :title="`按${primaryRateLabel(mode)}排序`"
                @click="toggleSort('winRate')"
              >
                {{ primaryRateLabel(mode) }}
                <ChevronsUpDown v-if="sortKey !== 'winRate'" class="size-3.5 opacity-50" />
                <ChevronDown
                  v-else
                  class="size-3.5 transition-transform"
                  :class="sortDir === 'asc' ? '' : 'rotate-180'"
                />
              </button>
            </th>
            <th class="w-[4.5rem] px-3 py-2.5">
              <button
                type="button"
                class="inline-flex items-center gap-0.5 font-medium hover:text-foreground"
                :class="sortKey === 'pickRate' ? 'text-foreground' : 'text-muted-foreground'"
                title="按选取排序"
                @click="toggleSort('pickRate')"
              >
                选取
                <ChevronsUpDown v-if="sortKey !== 'pickRate'" class="size-3.5 opacity-50" />
                <ChevronDown
                  v-else
                  class="size-3.5 transition-transform"
                  :class="sortDir === 'asc' ? '' : 'rotate-180'"
                />
              </button>
            </th>
            <th v-if="showBanColumn(mode)" class="w-[4.5rem] px-3 py-2.5 font-medium">禁用</th>
            <th class="w-[4.5rem] px-3 py-2.5">
              <button
                type="button"
                class="inline-flex items-center gap-0.5 font-medium hover:text-foreground"
                :class="sortKey === 'play' ? 'text-foreground' : 'text-muted-foreground'"
                title="按场次排序"
                @click="toggleSort('play')"
              >
                场次
                <ChevronsUpDown v-if="sortKey !== 'play'" class="size-3.5 opacity-50" />
                <ChevronDown
                  v-else
                  class="size-3.5 transition-transform"
                  :class="sortDir === 'asc' ? '' : 'rotate-180'"
                />
              </button>
            </th>
            <th v-if="showRoleColumn(mode)" class="px-3 py-2.5 font-medium">定位</th>
            <th v-if="showCounterColumn(mode)" class="px-3 py-2.5 font-medium">克制</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="row in sortedRows"
            :key="`${row.championId}-${row.position}`"
            class="cursor-pointer border-b border-border/30 transition-colors hover:bg-muted/35"
            @click="emit('select-champion', row.championId, row.position)"
          >
            <td class="px-4 py-2.5 text-muted-foreground">{{ row.rank }}</td>
            <td class="px-3 py-2.5">
              <div class="flex min-w-0 items-center gap-3">
                <img :src="getChampionIconUrl(row.championId)" alt="" class="size-9 rounded-lg ring-1 ring-border/50" />
                <span class="truncate font-medium">{{ getChampionName(row.championId) }}</span>
              </div>
            </td>
            <td class="px-3 py-2.5">
              <span
                class="inline-flex min-w-8 justify-center rounded-md px-1.5 py-0.5 text-sm font-bold tabular-nums"
                :class="tierClass(row.tier)"
              >
                {{ formatTier(row.tier) }}
              </span>
            </td>
            <td class="px-3 py-2.5 font-medium text-foreground">{{ formatPct(row.winRate) }}</td>
            <td class="px-3 py-2.5 text-muted-foreground">{{ formatPct(row.pickRate) }}</td>
            <td v-if="showBanColumn(mode)" class="px-3 py-2.5 text-muted-foreground">
              {{ formatPct(row.banRate) }}
            </td>
            <td class="px-3 py-2.5 text-muted-foreground" :title="String(row.play)">
              {{ formatPlay(row.play) }}
            </td>
            <td v-if="showRoleColumn(mode)" class="px-3 py-2.5">
              <div class="flex flex-wrap gap-1">
                <span
                  v-for="role in row.roles.slice(0, 2)"
                  :key="role.name"
                  class="rounded bg-muted px-1.5 py-0.5 text-xs font-medium text-muted-foreground"
                  :title="`${formatPct(role.roleRate)} · 胜率 ${formatPct(role.winRate)}`"
                >
                  {{ roleLabel(role.name) }}
                </span>
              </div>
            </td>
            <td v-if="showCounterColumn(mode)" class="px-3 py-2.5">
              <div class="flex items-center -space-x-1">
                <img
                  v-for="c in row.counters.slice(0, 4)"
                  :key="c.championId"
                  :src="getChampionIconUrl(c.championId)"
                  :title="getChampionName(c.championId)"
                  alt=""
                  class="size-7 rounded-full ring-2 ring-card"
                />
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ChevronDown, ChevronsUpDown } from 'lucide-vue-next'
import { getChampionIconUrl, getChampionName } from '@/lib'
import { primaryRateLabel, showBanColumn, showCounterColumn, showRoleColumn, usesLanePosition } from '../types/modes'

type SortKey = 'rank' | 'winRate' | 'pickRate' | 'play'
type SortDir = 'asc' | 'desc'

const props = withDefaults(
  defineProps<{
    tierList: OpggTierList | null
    loading?: boolean
    defaultPosition?: string
    mode?: string
    /** 嵌在左右分栏里时去掉外层 Card */
    embedded?: boolean
  }>(),
  { mode: 'ranked', embedded: false }
)

const emit = defineEmits<{
  'select-champion': [championId: number, position: string]
  'update:position': [position: string]
}>()

const positionFilters = [
  { value: 'all', label: '全部' },
  { value: 'TOP', label: '上单' },
  { value: 'JUNGLE', label: '打野' },
  { value: 'MID', label: '中单' },
  { value: 'ADC', label: '下路' },
  { value: 'SUPPORT', label: '辅助' }
] as const

const activePosition = ref(usesLanePosition(props.mode) ? props.defaultPosition || 'MID' : 'all')
const sortKey = ref<SortKey>('rank')
const sortDir = ref<SortDir>('asc')

const resetSort = () => {
  sortKey.value = 'rank'
  sortDir.value = 'asc'
}

/** 首次点某列：rank 升序，其余降序；同列再点切换方向；点 # 且已是 rank 升序时保持默认 */
const toggleSort = (key: SortKey) => {
  if (key === 'rank' && sortKey.value === 'rank' && sortDir.value === 'asc') {
    sortDir.value = 'desc'
    return
  }
  if (key === 'rank') {
    resetSort()
    return
  }
  if (sortKey.value === key) {
    sortDir.value = sortDir.value === 'asc' ? 'desc' : 'asc'
    return
  }
  sortKey.value = key
  sortDir.value = 'desc'
}

watch(activePosition, (pos) => {
  resetSort()
  if (pos !== 'all') emit('update:position', pos)
})

watch(
  () => props.mode,
  (m) => {
    activePosition.value = usesLanePosition(m) ? props.defaultPosition || 'MID' : 'all'
    resetSort()
  }
)

watch(
  () => props.defaultPosition,
  (v) => {
    if (v && usesLanePosition(props.mode)) activePosition.value = v
  }
)

/** 供左侧网格选英雄时沿用当前分路筛选 */
defineExpose({
  activePosition
})

type TierRow = {
  championId: number
  position: string
  rank: number
  tier: number
  winRate: number
  pickRate: number
  banRate: number
  play: number
  counters: OpggTierCounter[]
  roles: OpggTierRole[]
}

const rows = computed((): TierRow[] => {
  const list = props.tierList?.data ?? []
  const pos = activePosition.value
  const laneMode = usesLanePosition(props.mode)

  if (!laneMode || pos === 'all') {
    return list
      .map((item) => ({
        championId: item.championId,
        position: item.positions[0]?.name || (laneMode ? 'MID' : 'none'),
        rank: item.averageStats.rank,
        tier: item.averageStats.tier,
        winRate: item.averageStats.winRate,
        pickRate: item.averageStats.pickRate,
        banRate: item.averageStats.banRate,
        play: item.averageStats.play,
        counters: item.positions[0]?.counters ?? [],
        roles: item.roles ?? []
      }))
      .filter((r) => r.rank > 0 || r.winRate > 0)
  }

  return list
    .flatMap((item) => {
      const p = item.positions.find((x) => x.name === pos)
      if (!p) return []
      return [
        {
          championId: item.championId,
          position: p.name,
          rank: p.stats.rank,
          tier: p.stats.tier,
          winRate: p.stats.winRate,
          pickRate: p.stats.pickRate,
          banRate: p.stats.banRate,
          play: p.stats.play,
          counters: p.counters,
          roles: item.roles ?? []
        }
      ]
    })
    .filter((r) => r.rank > 0)
})

const sortedRows = computed(() => {
  const list = [...rows.value]
  const dir = sortDir.value === 'asc' ? 1 : -1
  const key = sortKey.value
  list.sort((a, b) => {
    const av = a[key]
    const bv = b[key]
    if (av === bv) return (a.rank || 9999) - (b.rank || 9999)
    return (av < bv ? -1 : 1) * dir
  })
  return list
})

const formatPct = (n: number) => `${(n * 100).toFixed(2)}%`

const formatPlay = (n: number) => {
  if (!n || n <= 0) return '—'
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return String(n)
}

/**
 * OP.GG `tier_data.tier`：1=OP，2=1，3=2…（与官网徽章一致，不能原样输出数字）
 */
const formatTier = (tier: number) => {
  if (tier <= 0) return '—'
  if (tier === 1) return 'OP'
  return String(tier - 1)
}

/** Tier：仅 OP 用品牌色强调，其余同级中性底 */
const tierClass = (tier: number) => {
  if (tier === 1) return 'bg-primary/15 text-primary'
  return 'bg-muted text-muted-foreground'
}

const roleLabel = (name: string) => {
  const map: Record<string, string> = {
    TANK: '坦克',
    FIGHTER: '战士',
    MAGE: '法师',
    ASSASSIN: '刺客',
    MARKSMAN: '射手',
    SUPPORT: '辅助'
  }
  return map[name] || map[name.toUpperCase()] || name
}
</script>
