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
        <div v-if="loading" class="flex items-center justify-center gap-3 px-6 py-16">
          <Spinner class="size-5 text-primary" />
          <span class="text-sm text-muted-foreground">正在查询召唤师战绩…</span>
        </div>

        <div v-else-if="currentResult" class="space-y-4 px-6 py-4 pb-6">
          <CompactProfileHeader
            :summoner-info="currentResult.summonerInfo"
            :today-matches="emptyTodayMatches"
            :solo-rank="soloRank"
            :flex-rank="flexRank"
            :show-today="false"
          />
          <GameStats
            :is-connected="true"
            :match-history-loading="false"
            :match-statistics="currentResult.matches"
            :ranked-stats="currentResult.positionAnalysis?.rankedStats"
            :other-stats="currentResult.positionAnalysis?.otherStats"
            :position-stats="currentResult.positionAnalysis?.positionStats"
            :main-position="currentResult.positionAnalysis?.mainPosition"
            :display-games="currentResult.matches.totalGames"
            @fetch-match-history="emit('refresh')"
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
import GameStats from '../GameStats.vue'
import { buildSummonerRankPresentation } from '@/shared/utils/summonerRankPresentation'

const props = defineProps<{
  selectedPlayer: { displayName: string } | null
  currentResult: SummonerWithMatches | null
  loading: boolean
}>()

const emit = defineEmits<{ refresh: [] }>()
const open = defineModel<boolean>('open', { required: true })

const displayName = computed(
  () => props.currentResult?.displayName || props.selectedPlayer?.displayName || '召唤师详情'
)
const emptyTodayMatches = { total: 0, wins: 0, losses: 0 }

const soloRank = computed(() => {
  const info = props.currentResult?.summonerInfo
  return buildSummonerRankPresentation({
    tier: info?.soloRankTier,
    division: info?.soloRankDivision,
    leaguePoints: info?.soloRankLp,
    wins: info?.soloRankWins,
    losses: info?.soloRankLosses
  })
})

const flexRank = computed(() => {
  const info = props.currentResult?.summonerInfo
  return buildSummonerRankPresentation({
    tier: info?.flexRankTier,
    division: info?.flexRankDivision,
    leaguePoints: info?.flexRankLp,
    wins: info?.flexRankWins,
    losses: info?.flexRankLosses
  })
})
</script>
