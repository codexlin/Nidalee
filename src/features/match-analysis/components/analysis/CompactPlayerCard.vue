<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import { getChampionIconUrl, getChampionName, getRoleIconUrl, getSpellMeta } from '@/lib'
import type { UIPlayerData } from '@/types/match-analysis'
import PlayerInsightSummary from './PlayerInsightSummary.vue'
import PlayerRankBadges from './PlayerRankBadges.vue'
import RecentPerformanceGrid from './RecentPerformanceGrid.vue'

type CompactPlayer = UIPlayerData & { assignedPosition?: string | null }

const props = defineProps<{
  player: CompactPlayer
  playerStats?: PlayerMatchStats | null
  isLocal?: boolean
  isAlly?: boolean
}>()

const emit = defineEmits<{ select: [player: CompactPlayer] }>()

const identityPending = computed(() => {
  if (props.player.isBot) return false
  const puuid = props.player.puuid?.trim()
  if (puuid) return false
  const name = props.player.displayName?.trim()
  return !name || name === '未知召唤师'
})

const nameLabel = computed(() => {
  if (props.player.isBot) return props.player.displayName?.trim() || '机器人'
  if (identityPending.value) return props.isAlly === false ? '敌方选手' : '匿名玩家'
  return props.player.displayName
})

const canSelect = computed(() => !props.player.isBot && !identityPending.value)
const performanceView = shallowRef<'recent' | 'sample'>('recent')
const analysisSample = computed(() => props.playerStats?.recentPerformance ?? [])
const visiblePerformance = computed(() =>
  performanceView.value === 'recent' ? props.player.recentMatches : analysisSample.value
)
const analysisBasis = computed(() => props.player.analysisBasis)
const analysisStatus = computed(() => props.player.analysisStatus)

const scopeLabels: Record<PlayerAnalysisScope, string> = {
  soloRanked: '单双排',
  flexRanked: '灵活排位',
  ranked: '近期排位',
  summonersRift: '召唤师峡谷',
  aram: '极地大乱斗',
  currentMode: '当前模式',
  recentOverall: '近期综合'
}

const analysisScopeLabel = computed(() => {
  const scope = analysisBasis.value?.primaryScope
  return scope ? scopeLabels[scope] : '近期综合'
})

const confidenceLabel = computed(() => {
  if (analysisBasis.value?.confidence === 'high') return '高可信'
  if (analysisBasis.value?.confidence === 'medium') return '中可信'
  return '低可信'
})

const confidenceIndicatorClass = computed(() => {
  if (analysisBasis.value?.confidence === 'high') return 'bg-emerald-500/80'
  if (analysisBasis.value?.confidence === 'medium') return 'bg-amber-500/80'
  return 'bg-rose-500/80'
})

const confidenceExplanation = computed(() => {
  const basis = analysisBasis.value
  if (!basis) return ''
  if (basis.fallbackUsed) {
    if (basis.primaryScope === 'ranked') return '本队列不足，合并排位样本'
    if (basis.primaryScope === 'summonersRift') return '排位不足，扩展峡谷样本'
    if (basis.primaryScope === 'recentOverall') return '采用全部模式的近期样本'
  }
  if (basis.confidence === 'high') return '命中样本充分'
  if (basis.confidence === 'medium') return '样本量一般'
  return '样本较少，仅供参考'
})

const analysisBasisSummary = computed(() => {
  const basis = analysisBasis.value
  if (!basis) return ''
  return `分析范围：${analysisScopeLabel.value} ${basis.primaryGames} 场 · ${confidenceLabel.value} · ${confidenceExplanation.value}`
})

const positionLabels: Record<string, string> = {
  TOP: '上路',
  JUNGLE: '打野',
  MIDDLE: '中路',
  MID: '中路',
  BOTTOM: '下路',
  ADC: '下路',
  UTILITY: '辅助',
  SUPPORT: '辅助'
}

function positionLabel(position: string): string {
  return positionLabels[position.toUpperCase()] ?? position
}

function selectPlayer(): void {
  if (canSelect.value) emit('select', props.player)
}

function setPerformanceView(view: 'recent' | 'sample'): void {
  performanceView.value = view
}
</script>

<template>
  <article
    class="group flex h-full min-w-0 flex-col gap-1.5 overflow-hidden rounded-xl border bg-card/75 p-2 transition-colors"
    :class="[
      isLocal ? 'border-primary/65 shadow-[inset_3px_0_0_var(--color-primary)]' : 'border-border/50',
      canSelect ? 'cursor-pointer hover:border-primary/40 hover:bg-card' : 'cursor-default',
      player.isBot ? 'opacity-75 grayscale' : ''
    ]"
    @click="selectPlayer"
  >
    <header class="flex min-w-0 items-center gap-2">
      <div class="relative flex-none">
        <div class="size-9 overflow-hidden rounded-lg ring-1 ring-border/60">
          <img
            v-if="player.championId || player.championPickIntent"
            :src="getChampionIconUrl(player.championId || player.championPickIntent || 0)"
            :alt="getChampionName(player.championId || player.championPickIntent || 0)"
            class="size-full object-cover"
            :class="!player.championId ? 'opacity-50' : ''"
          />
          <div v-else class="size-full bg-muted" />
        </div>
        <span
          v-if="isLocal || player.isBot || identityPending"
          class="absolute -right-1 -top-1 rounded-md px-1 py-0.5 text-[8px] font-bold text-white"
          :class="isLocal ? 'bg-primary' : 'bg-muted-foreground'"
        >
          {{ isLocal ? '我' : player.isBot ? '机器人' : '匿名' }}
        </span>
      </div>

      <div class="min-w-0 flex-1">
        <div class="w-full max-w-36 truncate text-xs font-bold text-foreground" :title="nameLabel || undefined">
          {{ nameLabel }}
        </div>
        <div class="flex min-w-0 items-center gap-1 text-[10px] text-muted-foreground">
          <span class="truncate">
            {{
              player.championId
                ? getChampionName(player.championId)
                : player.championPickIntent
                  ? `预选 ${getChampionName(player.championPickIntent)}`
                  : '未选英雄'
            }}
          </span>
          <span v-if="player.position" class="inline-flex flex-none items-center gap-0.5 rounded bg-muted/50 px-1">
            <img
              v-if="getRoleIconUrl(player.position)"
              :src="getRoleIconUrl(player.position)"
              alt=""
              class="size-3 object-contain opacity-80"
            />
            {{ positionLabel(player.position) }}
          </span>
        </div>
      </div>

      <div class="flex flex-none gap-1">
        <div
          v-for="(spellId, index) in [player.spell1Id, player.spell2Id]"
          :key="index"
          class="size-6 overflow-hidden rounded-md bg-muted ring-1 ring-border/40"
        >
          <img
            v-if="spellId && getSpellMeta(spellId)?.icon"
            :src="getSpellMeta(spellId).icon"
            :alt="getSpellMeta(spellId).label"
            class="size-full object-cover"
          />
        </div>
      </div>
    </header>

    <PlayerRankBadges
      v-if="!player.isBot && !identityPending"
      :solo-rank="player.soloRank"
      :flex-rank="player.flexRank"
    />

    <div v-if="player.isBot" class="flex flex-1 items-center justify-center text-xs text-muted-foreground">
      机器人，无需分析
    </div>
    <div v-else-if="identityPending" class="flex flex-1 items-center justify-center text-xs text-muted-foreground">
      选人阶段匿名，进入游戏后自动补全
    </div>
    <div
      v-else-if="analysisStatus === 'loading'"
      class="flex flex-1 items-center justify-center text-xs text-muted-foreground"
    >
      正在分析近期战绩
    </div>
    <div
      v-else-if="analysisStatus === 'insufficientData' && !playerStats"
      class="flex flex-1 items-center justify-center text-xs text-muted-foreground"
    >
      最近 50 场中没有足够的可分析数据
    </div>
    <div
      v-else-if="analysisStatus === 'unavailable' && !playerStats"
      class="flex flex-1 items-center justify-center text-xs text-muted-foreground"
    >
      战绩服务暂不可用
    </div>
    <template v-else-if="playerStats">
      <PlayerInsightSummary :stats="playerStats" :ranked-rating="player.rankedRating" />
      <Tooltip v-if="analysisBasis">
        <TooltipTrigger as-child>
          <div
            class="flex min-w-0 cursor-help items-center gap-1 rounded-md bg-muted/15 px-1.5 py-1 text-[9px] leading-4"
          >
            <span class="text-muted-foreground">分析样本</span>
            <span class="font-medium text-foreground">{{ analysisBasis.primaryGames }}场</span>
            <i class="size-1 rounded-full" :class="confidenceIndicatorClass" />
            <span class="font-medium text-foreground/85">{{ confidenceLabel }}</span>
          </div>
        </TooltipTrigger>
        <TooltipContent class="max-w-72 text-xs">{{ analysisBasisSummary }}</TooltipContent>
      </Tooltip>
      <RecentPerformanceGrid
        v-if="player.recentMatches.length || analysisSample.length"
        :matches="visiblePerformance"
        :view="performanceView"
        :scope-label="performanceView === 'recent' ? '全部模式' : analysisScopeLabel"
        :recent-count="player.recentMatches.length"
        :sample-count="analysisSample.length"
        @update:view="setPerformanceView"
      />
    </template>
  </article>
</template>
