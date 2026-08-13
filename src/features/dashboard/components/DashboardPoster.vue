<template>
  <div
    ref="rootRef"
    class="dashboard-poster w-180 box-border bg-background text-foreground p-6 space-y-5"
    data-dashboard-poster
  >
    <div class="surface-raised p-5 space-y-5">
      <!-- 品牌条 -->
      <div class="flex items-end justify-between gap-3 border-b border-border/60 pb-3">
        <div>
          <p class="text-lg font-bold tracking-tight">Nidalee</p>
          <p class="text-xs text-muted-foreground mt-0.5">战绩海报</p>
        </div>
        <p class="text-xs text-muted-foreground tabular-nums">{{ modeCaption }}</p>
      </div>

      <!-- 召唤师 -->
      <div class="flex items-center gap-4">
        <img
          v-if="summonerInfo?.profileIconId"
          :src="getProfileIconUrl(summonerInfo.profileIconId)"
          crossorigin="anonymous"
          class="h-16 w-16 rounded-full ring-2 ring-border/50 shadow-md"
          alt=""
        />
        <div v-else class="h-16 w-16 rounded-full bg-muted ring-2 ring-border/50 flex items-center justify-center">
          <User class="h-7 w-7 text-muted-foreground" />
        </div>
        <div class="min-w-0 flex-1">
          <div class="flex items-baseline gap-2 min-w-0">
            <h2 class="text-lg font-bold truncate">
              {{ summonerInfo?.gameName || summonerInfo?.displayName || '未知召唤师' }}
            </h2>
            <span v-if="summonerInfo?.tagLine" class="text-sm text-muted-foreground shrink-0"
              >#{{ summonerInfo.tagLine }}</span
            >
          </div>
          <p class="mt-1 text-sm text-muted-foreground tabular-nums flex items-center gap-1.5 flex-wrap">
            <span>等级 {{ summonerInfo?.summonerLevel || 0 }}</span>
            <span class="text-border">·</span>
            <img v-if="challengeCrystalIconUrl" :src="challengeCrystalIconUrl" alt="" class="h-3.5 w-3.5 shrink-0" />
            <span>挑战积分 {{ challengePointsLabel }}</span>
          </p>
        </div>
      </div>

      <!-- 段位 + 今日 -->
      <div class="grid grid-cols-3 gap-3">
        <div class="surface-inset px-3 py-2.5 space-y-1">
          <p class="text-xs text-muted-foreground">单双排位</p>
          <p class="text-sm font-semibold truncate">
            {{ formatRankLine(soloRank) }}
          </p>
          <p class="text-sm tabular-nums">
            <span class="font-bold">{{ soloRank.leaguePoints }}</span>
            <span class="text-xs text-muted-foreground ml-0.5">LP</span>
            <span class="mx-1 text-border">·</span>
            <span :class="rateClass(soloRank.winRate)">{{ soloRank.winRate }}%</span>
          </p>
        </div>
        <div class="surface-inset px-3 py-2.5 space-y-1">
          <p class="text-xs text-muted-foreground">灵活组排</p>
          <p class="text-sm font-semibold truncate">
            {{ formatRankLine(flexRank) }}
          </p>
          <p class="text-sm tabular-nums">
            <span class="font-bold">{{ flexRank.leaguePoints }}</span>
            <span class="text-xs text-muted-foreground ml-0.5">LP</span>
            <span class="mx-1 text-border">·</span>
            <span :class="rateClass(flexRank.winRate)">{{ flexRank.winRate }}%</span>
          </p>
        </div>
        <div class="surface-inset px-3 py-2.5 space-y-1">
          <p class="text-xs text-muted-foreground">今日</p>
          <p class="text-lg font-semibold tabular-nums leading-none">{{ todayMatches.total }} 场</p>
          <p class="text-sm tabular-nums">
            <span class="text-emerald-600">{{ todayMatches.wins }}</span>
            <span class="text-muted-foreground mx-0.5">/</span>
            <span class="text-rose-600">{{ todayMatches.losses }}</span>
            <span class="mx-1 text-border">·</span>
            <span :class="todayWinRateClass">{{ todayWinRateLabel }}</span>
          </p>
        </div>
      </div>

      <!-- 概览 KPI -->
      <div v-if="matchStatistics" class="surface-inset px-4 py-3.5">
        <div class="flex flex-wrap items-end gap-x-6 gap-y-3">
          <div>
            <p class="text-xs text-muted-foreground mb-1">胜率</p>
            <p class="text-2xl font-bold tabular-nums leading-none" :class="winRateToneClass">
              {{ (matchStatistics.winRate || 0).toFixed(0) }}%
            </p>
          </div>
          <div class="h-9 w-px bg-border/70 mb-0.5" />
          <div>
            <p class="text-xs text-muted-foreground mb-1">战绩</p>
            <p class="text-lg font-semibold tabular-nums leading-none">
              <span class="text-emerald-600">{{ matchStatistics.wins || 0 }}</span>
              <span class="text-muted-foreground/70 font-normal mx-1">-</span>
              <span class="text-rose-600">{{ matchStatistics.losses || 0 }}</span>
              <span class="ml-1.5 text-xs font-normal text-muted-foreground tabular-nums">
                {{ matchStatistics.totalGames || 0 }} 场
              </span>
            </p>
          </div>
          <div class="h-9 w-px bg-border/70 mb-0.5" />
          <div>
            <p class="text-xs text-muted-foreground mb-1">平均 KDA</p>
            <p class="text-lg font-semibold tabular-nums leading-none">
              {{ (matchStatistics.avgKda || 0).toFixed(2) }}
            </p>
          </div>
        </div>
      </div>

      <SummonerTraits
        :analysis-traits="analysisTraits"
        :match-statistics="matchStatistics"
        :position-stats="positionStats"
        :main-position="mainPosition"
        :filter-mode="selectedMatchMode"
      />

      <div v-if="favoriteChampions.length" class="space-y-3">
        <div class="space-y-1">
          <h4 class="text-base font-semibold flex items-center">
            <Star class="h-5 w-5 mr-2 text-muted-foreground" />
            常用英雄
          </h4>
          <p class="text-xs text-muted-foreground">按最近游玩场次排序</p>
        </div>
        <div class="grid grid-cols-5 gap-2">
          <div
            v-for="champion in favoriteChampions"
            :key="champion.championId"
            class="surface-inset flex flex-col items-center px-2 py-2.5"
          >
            <img
              v-if="champion.championId"
              :src="getChampionIconUrl(champion.championId)"
              crossorigin="anonymous"
              alt=""
              class="h-9 w-9 rounded-full border border-primary/20"
            />
            <p class="text-xs font-medium text-center mt-1 truncate w-full">
              {{ resolveChampionName(champion.championId, champion.championName) }}
            </p>
            <p class="text-sm font-bold tabular-nums" :class="champWinClass(champion.winRate)">
              {{ champion.winRate.toFixed(0) }}%
            </p>
            <p class="text-xs text-muted-foreground tabular-nums">{{ champion.games }}场</p>
          </div>
        </div>
      </div>

      <div v-if="recentGames.length" class="space-y-3">
        <div class="space-y-1">
          <h4 class="text-base font-semibold flex items-center">
            <Calendar class="h-5 w-5 mr-2 text-muted-foreground" />
            最近对局
          </h4>
          <p class="text-xs text-muted-foreground">右下角为自研评级（S+～D）</p>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <div v-for="game in recentGames" :key="game.gameCreation" class="surface-inset relative flex overflow-hidden">
            <div :class="game.win ? 'bg-emerald-600' : 'bg-rose-600'" class="w-1 shrink-0" />
            <span
              class="pointer-events-none absolute -right-1 -bottom-2 z-0 select-none font-black leading-none tabular-nums -rotate-12 origin-bottom-right"
              :class="[gradeWatermarkSizeClass(displayGrade(game)), gradeWatermarkClass(displayGrade(game))]"
              aria-hidden="true"
            >
              {{ displayGrade(game) }}
            </span>
            <div class="relative z-10 flex-1 p-3">
              <div class="flex items-center justify-between mb-2 gap-2">
                <div class="flex items-center gap-2 min-w-0">
                  <img
                    v-if="game.championId"
                    :src="getChampionIconUrl(game.championId)"
                    crossorigin="anonymous"
                    alt=""
                    class="h-9 w-9 shrink-0 rounded-full border-2 border-primary/20"
                  />
                  <span class="font-semibold text-sm truncate">{{
                    resolveChampionName(game.championId, game.championName)
                  }}</span>
                </div>
                <div class="flex items-center gap-2 shrink-0">
                  <span class="flex items-center gap-1 text-xs text-muted-foreground tabular-nums">
                    <Timer class="w-3 h-3 shrink-0" />
                    {{ formatGameTime(game.gameDuration ?? 0) }}
                  </span>
                  <span
                    class="h-5 px-1.5 inline-flex items-center rounded-md text-xs font-medium text-white"
                    :class="game.win ? 'bg-emerald-600' : 'bg-rose-600'"
                  >
                    {{ game.win ? '胜' : '负' }}
                  </span>
                </div>
              </div>
              <div class="pr-6">
                <span class="font-mono font-bold text-base tabular-nums leading-none">
                  <span class="text-red-500">{{ game.kills }}</span>
                  <span class="text-muted-foreground/50">/</span>
                  <span class="text-muted-foreground">{{ game.deaths }}</span>
                  <span class="text-muted-foreground/50">/</span>
                  <span class="text-blue-500">{{ game.assists }}</span>
                </span>
                <div class="flex items-center gap-2 mt-2 text-xs text-muted-foreground">
                  <span class="flex items-center gap-1">
                    <Clock class="w-3 h-3 shrink-0" />
                    {{ formatRelativeTime(game.gameCreation ?? 0) }}
                  </span>
                  <span class="text-border">·</span>
                  <span class="truncate">{{ getQueueName(game.queueId ?? 0) }}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <p class="text-xs text-muted-foreground pt-1 border-t border-border/50">
        {{ exportedAtLabel }} · Nidalee · 自研评级非官方
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Calendar, Clock, Star, Timer, User } from 'lucide-vue-next'
import {
  getChallengeCrystalIconUrl,
  getChampionIconUrl,
  getProfileIconUrl,
  getQueueName,
  resolveChampionName
} from '@/lib'
import { getMatchModeLabel, type MatchModeKey } from '@/common/queueCatalog'
import SummonerTraits from './SummonerTraits.vue'
import { displayGrade, gradeWatermarkClass, gradeWatermarkSizeClass } from '@/shared/utils/matchGrade'

interface RankInfo {
  tier: string
  rank: string
  leaguePoints: number
  winRate: number
}

interface TodayMatches {
  total: number
  wins: number
  losses: number
}

const props = defineProps<{
  summonerInfo: SummonerInfo | null
  todayMatches: TodayMatches
  soloRank: RankInfo
  flexRank: RankInfo
  matchStatistics: PlayerMatchStats | null
  analysisTraits?: DeterministicTrait[] | null
  positionStats?: PositionStats[] | null
  mainPosition?: string | null
  selectedMatchMode?: MatchModeKey
  matchCount?: number | null
  /** 最近对局展示条数上限 */
  recentLimit?: number
}>()

const rootRef = ref<HTMLElement | null>(null)
defineExpose({
  getRoot: () => rootRef.value
})

const { formatChallengePoints, formatGameTime, formatRelativeTime } = useFormatters()

const exportedAtLabel = computed(() => {
  const d = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
})

const modeCaption = computed(() => {
  const mode = props.selectedMatchMode ? getMatchModeLabel(props.selectedMatchMode) : '全部模式'
  const count = props.matchCount && props.matchCount > 0 ? `${props.matchCount} 场样本` : ''
  return count ? `${mode} · ${count}` : mode
})

const favoriteChampions = computed(() => (props.matchStatistics?.favoriteChampions || []).slice(0, 5))

const recentGames = computed(() => {
  const limit = props.recentLimit && props.recentLimit > 0 ? props.recentLimit : 10
  return (props.matchStatistics?.recentPerformance || []).slice(0, limit)
})

const challengePointsLabel = computed(() => {
  const raw = props.summonerInfo?.challengePoints
  if (raw === null || raw === undefined || raw === '') return '—'
  const num = Number(raw)
  if (!Number.isFinite(num)) return formatChallengePoints(String(raw))
  if (num <= 0) return '—'
  return num >= 1000 ? formatChallengePoints(String(Math.trunc(num))) : num.toLocaleString()
})

const challengeCrystalIconUrl = computed(() => getChallengeCrystalIconUrl(props.summonerInfo?.challengeCrystalLevel))

const formatRankTierShort = (tier: string): string => {
  const tierMap: Record<string, string> = {
    IRON: '黑铁',
    BRONZE: '青铜',
    SILVER: '白银',
    GOLD: '黄金',
    PLATINUM: '铂金',
    EMERALD: '翡翠',
    DIAMOND: '钻石',
    MASTER: '大师',
    GRANDMASTER: '宗师',
    CHALLENGER: '王者'
  }
  return tierMap[tier] || tier
}

const formatRankLine = (rank: RankInfo) => {
  if (rank.tier === 'UNRANKED') return '未定级'
  const base = formatRankTierShort(rank.tier)
  return rank.rank ? `${base} ${rank.rank}` : base
}

const rateClass = (rate: number) => (rate >= 50 ? 'text-emerald-600' : 'text-rose-600')

const todayWinRate = computed(() => {
  const total = props.todayMatches?.total || 0
  if (total <= 0) return null
  return Math.round(((props.todayMatches?.wins || 0) / total) * 100)
})

const todayWinRateLabel = computed(() => (todayWinRate.value === null ? '—' : `${todayWinRate.value}%`))

const todayWinRateClass = computed(() => {
  if (todayWinRate.value === null) return 'text-muted-foreground'
  if (todayWinRate.value > 50) return 'text-emerald-600'
  if (todayWinRate.value < 50) return 'text-rose-600'
  return 'text-foreground'
})

const winRateToneClass = computed(() => {
  const rate = props.matchStatistics?.winRate ?? 0
  if (rate > 50) return 'text-emerald-600'
  if (rate < 50) return 'text-rose-600'
  return 'text-foreground'
})

const champWinClass = (winRate: number) => {
  if (winRate >= 60) return 'text-emerald-600'
  if (winRate >= 50) return 'text-yellow-600'
  return 'text-rose-600'
}
</script>
