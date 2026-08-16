<script setup lang="ts">
import { computed, inject } from 'vue'
import { appContextKey, type AppContext } from '@/types'
import CompactProfileHeader from '@/features/dashboard/components/CompactProfileHeader.vue'
import GameDetailDialog from '@/features/dashboard/components/detail/GameDetailDialog.vue'
import SummonerPerformancePanel from '@/features/summoner-performance/SummonerPerformancePanel.vue'
import { useSummonerAnalysisQuery } from '@/features/summoner-performance/composables/useSummonerAnalysisQuery'
import { buildSummonerRankPresentation } from '@/shared/utils/summonerRankPresentation'
import type { PerformanceScope } from '@/common/performanceScope'

const { isConnected } = inject(appContextKey) as AppContext
const settingsStore = useSettingsStore()
const performanceScope = computed<PerformanceScope>({
  get: () => settingsStore.performanceScope,
  set: (scope) => settingsStore.setPerformanceScope(scope)
})

const {
  onSearch,
  currentIndex,
  names,
  searchText,
  loading: identityLoading,
  error: identityError,
  currentResult,
  result
} = useSearchMatches()
const analysisQuery = useSummonerAnalysisQuery({
  puuid: () => currentResult.value?.puuid,
  scope: performanceScope,
  enabled: () => !!currentResult.value
})

const analysis = computed(() => analysisQuery.data.value ?? null)
const loading = computed(() => identityLoading.value || analysisQuery.isFetching.value)
const analysisError = computed(() => {
  const cause = analysisQuery.error.value
  return cause instanceof Error ? cause.message : cause ? String(cause) : ''
})
const errorMessage = computed(() => identityError.value || analysisError.value)
const emptyTodayMatches = { total: 0, wins: 0, losses: 0 }
const dialogOpen = ref(false)
const selectedGame = ref<MatchPerformance | null>(null)
const selectedGamePuuid = ref<string | null>(null)

const soloRank = computed(() => {
  const info = currentResult.value
  return buildSummonerRankPresentation({
    tier: info?.soloRankTier,
    division: info?.soloRankDivision,
    leaguePoints: info?.soloRankLp,
    wins: info?.soloRankWins,
    losses: info?.soloRankLosses
  })
})

const flexRank = computed(() => {
  const info = currentResult.value
  return buildSummonerRankPresentation({
    tier: info?.flexRankTier,
    division: info?.flexRankDivision,
    leaguePoints: info?.flexRankLp,
    wins: info?.flexRankWins,
    losses: info?.flexRankLosses
  })
})

function resultLabel(index: number): string {
  return result.value[index]?.displayName || names.value[index] || `召唤师 ${index + 1}`
}

function openGameDetail(game: MatchPerformance, puuid = currentResult.value?.puuid ?? '') {
  const normalizedPuuid = puuid.trim()
  if (!normalizedPuuid) return
  selectedGame.value = game
  selectedGamePuuid.value = normalizedPuuid
  dialogOpen.value = true
}

watch(
  () => currentResult.value?.puuid ?? null,
  (puuid, previousPuuid) => {
    if (!dialogOpen.value || puuid === previousPuuid) return
    dialogOpen.value = false
    selectedGame.value = null
    selectedGamePuuid.value = null
  }
)
</script>

<template>
  <div class="flex flex-col gap-6">
    <div v-if="!currentResult" class="flex min-h-[calc(100dvh-12rem)] flex-col items-center justify-center gap-4">
      <div class="space-y-1 text-center">
        <h1 class="text-lg font-medium text-foreground">战绩查询</h1>
        <p class="text-sm text-muted-foreground">输入召唤师名称，使用与仪表板相同的分析标准查看表现</p>
      </div>
      <SummonerSearchBox
        v-model:summoner-name="searchText"
        show-history
        :loading="identityLoading"
        @on-search="onSearch"
      />
      <p v-if="identityError" class="max-w-xl text-center text-sm text-destructive">{{ identityError }}</p>
    </div>

    <template v-else>
      <SummonerSearchBox
        v-model:summoner-name="searchText"
        compact
        class="min-w-0"
        :loading="identityLoading"
        @on-search="onSearch"
      />

      <div v-if="result.length > 1" class="surface-chip flex flex-wrap items-center gap-1 p-1" role="tablist">
        <button
          v-for="(_, index) in result"
          :key="result[index]?.puuid || index"
          type="button"
          role="tab"
          :aria-selected="index === currentIndex"
          class="rounded-lg px-3 py-1.5 text-sm font-medium outline-none transition-colors focus-visible:ring-3 focus-visible:ring-ring/50"
          :class="
            index === currentIndex
              ? 'bg-primary/15 text-primary'
              : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
          "
          @click="currentIndex = index"
        >
          {{ resultLabel(index) }}
        </button>
      </div>

      <CompactProfileHeader
        :summoner-info="currentResult"
        :today-matches="emptyTodayMatches"
        :solo-rank="soloRank"
        :flex-rank="flexRank"
        :show-today="false"
      />

      <SummonerPerformancePanel
        :is-connected="isConnected"
        :match-history-loading="loading"
        :error="errorMessage"
        :match-statistics="analysis?.overallStats ?? null"
        :analysis-traits="analysis?.traits"
        :position-stats="analysis?.positionStats"
        :main-position="analysis?.mainPosition"
        :scope="performanceScope"
        @scope-change="performanceScope = $event"
        @fetch-match-history="analysisQuery.refetch()"
        @open-game-detail="openGameDetail"
      />
    </template>

    <GameDetailDialog
      v-model:visible="dialogOpen"
      :selected-game="selectedGame"
      :analysis-puuid="selectedGamePuuid"
      @open-game-detail="openGameDetail"
    />
  </div>
</template>
