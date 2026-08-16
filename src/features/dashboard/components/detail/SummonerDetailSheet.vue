<template>
  <Sheet v-model:open="open">
    <SheetContent
      side="right"
      class="flex h-full w-full flex-col gap-0 overflow-hidden p-0 sm:w-[min(1100px,90vw)] sm:max-w-none"
    >
      <div
        class="shrink-0 border-b border-border/60 bg-background/95 px-6 py-4 pr-12 backdrop-blur supports-[backdrop-filter]:bg-background/60"
      >
        <SheetHeader class="space-y-1 text-left">
          <SheetTitle class="text-left text-lg font-bold">{{ displayName }}</SheetTitle>
          <SheetDescription>召唤师资料与近期战绩</SheetDescription>
        </SheetHeader>
      </div>

      <ScrollArea class="min-h-0 flex-1 border-none">
        <div v-if="isLoading" class="flex items-center justify-center gap-3 px-6 py-16">
          <Spinner class="size-5 text-primary" />
          <span class="text-sm text-muted-foreground">正在查询召唤师战绩…</span>
        </div>

        <div v-else-if="currentResult" class="space-y-4 px-6 py-4 pb-6">
          <CompactProfileHeader
            :summoner-info="currentResult"
            :today-matches="emptyTodayMatches"
            :solo-rank="soloRank"
            :flex-rank="flexRank"
            :show-today="false"
          />
          <SummonerPerformancePanel
            :is-connected="true"
            :match-history-loading="false"
            :error="analysisError"
            :match-statistics="analysis?.overallStats ?? null"
            :analysis-traits="analysis?.traits"
            :position-stats="analysis?.positionStats"
            :main-position="analysis?.mainPosition"
            :scope="performanceScope"
            @scope-change="performanceScope = $event"
            @fetch-match-history="analysisQuery.refetch()"
            @open-game-detail="openGameDetail"
          />
        </div>

        <div v-else class="flex items-center justify-center px-6 py-16">
          <div class="text-center">
            <Info class="h-10 w-10 text-muted-foreground mx-auto mb-3" />
            <h3 class="text-base font-semibold mb-1 text-foreground">暂无战绩数据</h3>
            <p class="text-sm text-muted-foreground">未能获取到该召唤师的战绩信息</p>
          </div>
        </div>
      </ScrollArea>
    </SheetContent>
  </Sheet>
</template>

<script setup lang="ts">
import { Info } from 'lucide-vue-next'
import CompactProfileHeader from '../CompactProfileHeader.vue'
import SummonerPerformancePanel from '@/features/summoner-performance/SummonerPerformancePanel.vue'
import { useSummonerAnalysisQuery } from '@/features/summoner-performance/composables/useSummonerAnalysisQuery'
import { buildSummonerRankPresentation } from '@/shared/utils/summonerRankPresentation'
import type { PerformanceScope } from '@/common/performanceScope'

const props = defineProps<{
  selectedPlayer: { displayName: string } | null
  currentResult: SummonerInfo | null
  loading: boolean
}>()

const open = defineModel<boolean>('open', { required: true })
const emit = defineEmits<{
  (e: 'open-game-detail', game: MatchPerformance, puuid: string): void
}>()

const settingsStore = useSettingsStore()
const performanceScope = computed<PerformanceScope>({
  get: () => settingsStore.performanceScope,
  set: (scope) => settingsStore.setPerformanceScope(scope)
})
const analysisQuery = useSummonerAnalysisQuery({
  puuid: () => props.currentResult?.puuid,
  scope: performanceScope,
  enabled: () => open.value && !!props.currentResult
})
const analysis = computed(() => analysisQuery.data.value ?? null)
const isLoading = computed(() => props.loading || analysisQuery.isFetching.value)
const analysisError = computed(() => {
  const cause = analysisQuery.error.value
  return cause instanceof Error ? cause.message : cause ? String(cause) : ''
})

const displayName = computed(
  () => props.currentResult?.displayName || props.selectedPlayer?.displayName || '召唤师详情'
)
const emptyTodayMatches = { total: 0, wins: 0, losses: 0 }

function openGameDetail(game: MatchPerformance) {
  const puuid = props.currentResult?.puuid?.trim()
  if (!puuid) return
  emit('open-game-detail', game, puuid)
}

const soloRank = computed(() => {
  const info = props.currentResult
  return buildSummonerRankPresentation({
    tier: info?.soloRankTier,
    division: info?.soloRankDivision,
    leaguePoints: info?.soloRankLp,
    wins: info?.soloRankWins,
    losses: info?.soloRankLosses
  })
})

const flexRank = computed(() => {
  const info = props.currentResult
  return buildSummonerRankPresentation({
    tier: info?.flexRankTier,
    division: info?.flexRankDivision,
    leaguePoints: info?.flexRankLp,
    wins: info?.flexRankWins,
    losses: info?.flexRankLosses
  })
})
</script>
