<template>
  <div v-if="featuredCards.length || minorCards.length" class="space-y-3">
    <div class="flex items-center justify-between gap-2">
      <h4 class="font-semibold flex items-center">
        <UserCheck class="h-4 w-4 mr-2 text-muted-foreground" />
        召唤师特征
      </h4>
      <span class="text-xs text-muted-foreground">{{ sectionHint }}</span>
    </div>

    <div class="space-y-2">
      <div
        v-for="card in featuredCards"
        :key="card.key"
        class="flex items-center gap-3 rounded-xl border px-3 py-2.5"
        :class="
          card.key === primaryKey
            ? 'border-primary/45 bg-primary/5'
            : 'border-border/70 bg-muted/10'
        "
      >
        <div
          class="h-10 w-10 shrink-0 rounded-lg flex items-center justify-center text-sm font-bold"
          :class="
            card.key === primaryKey
              ? 'bg-primary text-primary-foreground'
              : 'bg-muted text-muted-foreground'
          "
        >
          {{ card.iconText }}
        </div>

        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-1.5 min-w-0">
            <span
              class="truncate"
              :class="card.key === primaryKey ? 'text-sm font-semibold' : 'text-sm font-medium'"
            >
              {{ card.name }}
            </span>
            <Badge
              v-if="card.key === primaryKey"
              variant="secondary"
              class="text-[10px] px-1.5 py-0 h-4 shrink-0"
            >
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

        <div class="shrink-0 text-right pl-1 min-w-[3.25rem]">
          <div
            class="text-xl font-semibold tabular-nums leading-none tracking-tight"
            :class="winRateClass(card.winRate)"
          >
            {{ card.winRate.toFixed(0) }}%
          </div>
          <div class="text-[10px] text-muted-foreground mt-1">胜率</div>
        </div>
      </div>
    </div>

    <p v-if="minorCards.length" class="text-xs text-muted-foreground px-0.5">
      <span class="text-foreground/70">也打过</span>
      <span v-for="(card, idx) in minorCards" :key="card.key">
        <span v-if="idx > 0"> · </span>
        {{ card.name }} {{ card.games }}场
        <span class="tabular-nums" :class="winRateClass(card.winRate)">
          ({{ card.winRate.toFixed(0) }}%)
        </span>
      </span>
    </p>
  </div>
</template>

<script setup lang="ts">
import { UserCheck } from 'lucide-vue-next'
import { getPositionLabel } from '@/common/positionLabels'
import { isMatchModeKey, type MatchModeKey } from '@/common/queueCatalog'

/** 少于该场数不当完整身份卡（1–2 场波动太大） */
const MIN_FEATURED_GAMES = 3

interface IdentityCard {
  key: string
  name: string
  iconText: string
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
  filterMode?: string | null
}>()

/** 排位筛选才用分路身份；娱乐 / 全部 / 未知分路走模式身份 */
const usePositionIdentity = computed(() => {
  const mode = props.filterMode
  if (!mode || !isMatchModeKey(mode)) return false
  const key = mode as MatchModeKey
  return key === 'mixedRanked' || key === '420' || key === '440'
})

const sectionHint = computed(() => (usePositionIdentity.value ? '分路近况' : '模式近况'))

/** 胜率以 50% 为界：正绿负红，刚好 50 用正文色 */
const winRateClass = (rate: number): string => {
  if (rate > 50) return 'text-emerald-600 dark:text-emerald-400'
  if (rate < 50) return 'text-rose-600 dark:text-rose-400'
  return 'text-foreground'
}

const positionIcon = (code: string): string => {
  const icons: Record<string, string> = {
    TOP: '上',
    JUNGLE: '野',
    MID: '中',
    ADC: '下',
    SUPPORT: '辅',
    UNKNOWN: '?'
  }
  return icons[code] || getPositionLabel(code).slice(0, 1)
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
      iconText: positionIcon(pos.position),
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
    (t) =>
      t.supportsConclusion &&
      t.key.startsWith('mode_affinity') &&
      t.key !== 'mode_affinity_ranked'
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

const identityCards = computed(() =>
  positionCards.value.length ? positionCards.value : modeIdentityCards.value
)

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
