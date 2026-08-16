<template>
  <div v-if="featuredCards.length || minorCards.length" class="space-y-3">
    <div class="space-y-1">
      <h4 class="text-base font-semibold flex items-center">
        <UserCheck class="h-4 w-4 mr-2 text-muted-foreground" />
        召唤师特征
      </h4>
      <p class="text-xs text-muted-foreground">{{ sectionHint }}</p>
    </div>

    <div class="space-y-2">
      <div v-for="card in featuredCards" :key="card.key" class="surface-inset flex items-center gap-3 px-3 py-2.5">
        <div
          class="h-10 w-10 shrink-0 rounded-lg flex items-center justify-center text-sm font-bold bg-muted text-muted-foreground overflow-hidden"
        >
          <img v-if="card.iconUrl" :src="card.iconUrl" :alt="card.name" class="h-6 w-6 object-contain" />
          <template v-else>{{ card.iconText }}</template>
        </div>

        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-1.5 min-w-0">
            <span class="truncate text-sm font-medium">
              {{ card.name }}
            </span>
            <Badge v-if="card.key === primaryKey" variant="secondary" class="text-xs px-1.5 py-0 h-5 shrink-0">
              主要
            </Badge>
          </div>
          <p class="mt-0.5 flex flex-wrap items-baseline gap-x-2.5 gap-y-0.5 text-xs tabular-nums">
            <span class="text-muted-foreground">
              {{ card.games }}场 ·
              <span class="text-green-600 dark:text-green-400">{{ card.wins }}胜</span>
              <span class="text-red-600 dark:text-red-400">{{ card.losses }}负</span>
            </span>
            <span class="text-muted-foreground">
              KDA
              <span class="ml-1 font-medium text-foreground">{{ card.kdaText }}</span>
            </span>
            <span v-if="card.focusLabel" class="text-muted-foreground">
              {{ card.focusLabel }}
              <span class="ml-1 font-medium text-foreground">{{ card.focusValue }}</span>
            </span>
          </p>
        </div>

        <div class="shrink-0 text-right pl-1 min-w-13">
          <div
            class="text-lg font-semibold tabular-nums leading-none tracking-tight"
            :class="winRateClass(card.winRate)"
          >
            {{ card.winRate.toFixed(0) }}%
          </div>
          <div class="text-xs text-muted-foreground mt-1">胜率</div>
        </div>
      </div>
    </div>

    <p v-if="minorCards.length" class="text-xs text-muted-foreground px-0.5">
      <span class="text-foreground/70">也打过</span>
      <span v-for="(card, idx) in minorCards" :key="card.key">
        <span v-if="idx > 0"> · </span>
        {{ card.name }} {{ card.games }}场
        <span class="tabular-nums" :class="winRateClass(card.winRate)"> ({{ card.winRate.toFixed(0) }}%) </span>
      </span>
    </p>
  </div>
</template>

<script setup lang="ts">
import { UserCheck } from 'lucide-vue-next'
import { getPositionLabel } from '@/common/positionLabels'
import type { PerformanceCategory } from '@/common/performanceScope'
import { getRoleIconUrl } from '@/lib'

/** 少于该场数不当完整身份卡（1–2 场波动太大） */
const MIN_FEATURED_GAMES = 3

interface IdentityCard {
  key: string
  name: string
  iconText: string
  iconUrl?: string
  games: number
  wins: number
  losses: number
  winRate: number
  kdaText: string
  focusLabel: string
  focusValue: string
}

const props = defineProps<{
  analysisTraits?: DeterministicTrait[] | null
  matchStatistics?: PlayerMatchStats | null
  positionStats?: PositionStats[] | null
  mainPosition?: string | null
  performanceCategory?: PerformanceCategory
}>()

const hasPositionStats = computed(() =>
  (props.positionStats || []).some((p) => p.position !== 'UNKNOWN' && p.games > 0)
)

const usePositionIdentity = computed(() => props.performanceCategory === 'ranked' && hasPositionStats.value)

const sectionHint = computed(() =>
  usePositionIdentity.value ? '按分路近况归纳主要身份' : '按当前模式近况归纳主要身份'
)

/** 胜率以 50% 为界：正绿负红，刚好 50 用正文色 */
const winRateClass = (rate: number): string => {
  if (rate > 50) return 'text-emerald-600 dark:text-emerald-400'
  if (rate < 50) return 'text-rose-600 dark:text-rose-400'
  return 'text-foreground'
}

const modeIcon = (key: string, name: string): string => {
  if (key.includes('hextech') || key.includes('2400')) return '海'
  if (key.includes('aram') || key.includes('450')) return '乱'
  if (key.includes('fun')) return '娱'
  return name.slice(0, 1)
}

const focusForPosition = (pos: PositionStats): { label: string; value: string } => {
  if (pos.position === 'SUPPORT') {
    return { label: '视野/分', value: pos.stats.vspm.toFixed(2) }
  }
  if (pos.position === 'JUNGLE') {
    return { label: '补刀/分', value: pos.stats.cspm.toFixed(1) }
  }
  return { label: 'CS/min', value: pos.stats.cspm.toFixed(1) }
}

const positionCards = computed((): IdentityCard[] => {
  if (!usePositionIdentity.value) return []
  const list = (props.positionStats || []).filter((p) => p.position !== 'UNKNOWN')
  return list.map((pos) => {
    const focus = focusForPosition(pos)
    return {
      key: `pos_${pos.position}`,
      name: getPositionLabel(pos.position),
      iconText: getPositionLabel(pos.position).slice(0, 1),
      iconUrl: getRoleIconUrl(pos.position) || undefined,
      games: pos.games,
      wins: pos.wins,
      losses: pos.games - pos.wins,
      winRate: pos.winRate,
      kdaText: pos.stats.avgKda.toFixed(2),
      focusLabel: focus.label,
      focusValue: focus.value
    }
  })
})

/**
 * 娱乐 / 全部：模式亲和当身份（海克斯/乱斗/娱乐）
 * 不展示「排位为主」，也不展示过程向娱乐标签
 */
const modeIdentityCards = computed((): IdentityCard[] => {
  if (positionCards.value.length) return []
  const traits = (props.analysisTraits || []).filter(
    (t) => t.supportsConclusion && t.key.startsWith('mode_affinity') && t.key !== 'mode_affinity_ranked'
  )
  const stats = props.matchStatistics
  const total = stats?.totalGames || 0
  const wins = stats?.wins || 0
  const losses = stats?.losses ?? Math.max(0, total - wins)
  const winRate = stats?.winRate ?? 0

  return traits.map((t) => {
    const sample = t.sampleCount || total
    const sharePct = Math.round((t.frequency || 0) * 100)
    return {
      key: t.key,
      name: t.name,
      iconText: modeIcon(t.key, t.name),
      games: sample,
      wins,
      losses,
      winRate,
      kdaText: (stats?.avgKda ?? 0).toFixed(2),
      focusLabel: total ? '占比' : '',
      focusValue: total ? `${sharePct}%` : ''
    }
  })
})

const identityCards = computed(() => (positionCards.value.length ? positionCards.value : modeIdentityCards.value))

const primaryKey = computed(() => {
  if (usePositionIdentity.value && props.mainPosition && props.mainPosition !== 'UNKNOWN') {
    return `pos_${props.mainPosition}`
  }
  return identityCards.value[0]?.key ?? ''
})

const featuredCards = computed(() => {
  const cards = identityCards.value
  if (!positionCards.value.length) return cards
  return cards.filter((c) => c.key === primaryKey.value || c.games >= MIN_FEATURED_GAMES)
})

const minorCards = computed(() => {
  if (!positionCards.value.length) return []
  return identityCards.value.filter((c) => c.key !== primaryKey.value && c.games < MIN_FEATURED_GAMES)
})
</script>
