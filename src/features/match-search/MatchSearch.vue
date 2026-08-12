<template>
  <div class="flex flex-col gap-6">
    <!-- 空态 -->
    <div v-if="!currentResult" class="flex min-h-[calc(100dvh-12rem)] flex-col items-center justify-center gap-4">
      <div class="space-y-1 text-center">
        <h1 class="text-lg font-medium text-foreground">战绩查询</h1>
        <p class="text-sm text-muted-foreground">输入召唤师名称，查看近期表现与位置倾向</p>
      </div>
      <SummonerSearchBox v-model:summoner-name="searchText" show-history :loading="loading" @on-search="onSearch" />
      <label class="mx-auto flex max-w-xl cursor-pointer items-center gap-2 px-0.5 text-xs text-muted-foreground">
        <Switch :model-value="applyDefaultFilterOnSearch" @update:model-value="setApplyDefaultFilterOnSearch" />
        <span>
          查询时跟随仪表盘模式
          <span class="text-muted-foreground/80">（当前：{{ dashboardModeLabel }}）</span>
        </span>
      </label>
    </div>

    <!-- 有结果：对齐仪表盘主路径 -->
    <template v-else>
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between sm:gap-3">
        <SummonerSearchBox
          v-model:summoner-name="searchText"
          compact
          class="min-w-0 flex-1"
          :loading="loading"
          @on-search="onSearch"
        />
        <label class="flex shrink-0 cursor-pointer items-center gap-2 text-xs text-muted-foreground">
          <Switch :model-value="applyDefaultFilterOnSearch" @update:model-value="setApplyDefaultFilterOnSearch" />
          <span>
            跟随仪表盘
            <span class="text-muted-foreground/80">（{{ dashboardModeLabel }}）</span>
          </span>
        </label>
      </div>

      <div v-if="names.length > 1" class="surface-chip flex flex-wrap items-center gap-1 p-1">
        <button
          v-for="(name, idx) in names"
          :key="name"
          type="button"
          class="rounded-lg px-3 py-1.5 text-sm font-medium outline-none transition-colors focus-visible:ring-ring/50 focus-visible:ring-[3px]"
          :class="
            idx === currentIndex
              ? 'bg-primary/15 text-primary'
              : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
          "
          @click="currentIndex = idx"
        >
          {{ name }}
        </button>
      </div>

      <CompactProfileHeader
        :summoner-info="currentResult.summonerInfo"
        :today-matches="emptyTodayMatches"
        :solo-rank="soloRank"
        :flex-rank="flexRank"
        :show-today="false"
      />

      <GameStats
        :is-connected="isConnected"
        :match-history-loading="loading"
        :match-statistics="filteredCurrentMatches || currentResult.matches"
        :ranked-stats="searchPositionAnalysis?.rankedStats"
        :other-stats="searchPositionAnalysis?.otherStats"
        :position-stats="searchPositionAnalysis?.positionStats"
        :main-position="searchPositionAnalysis?.mainPosition"
        :display-games="(filteredCurrentMatches || currentResult.matches)?.totalGames"
        @fetch-match-history="onSearch"
      />
    </template>
  </div>
</template>

<script lang="ts" setup>
import { appContextKey, type AppContext } from '@/types'
import CompactProfileHeader from '@/features/dashboard/components/CompactProfileHeader.vue'
import { getMatchModeLabel } from '@/common/queueCatalog'
import { buildSummonerRankPresentation } from '@/shared/utils/summonerRankPresentation'
import { storeToRefs } from 'pinia'

const { isConnected } = inject(appContextKey) as AppContext

const settingsStore = useSettingsStore()
const { applyDefaultFilterOnSearch, lastMatchMode } = storeToRefs(settingsStore)
const { setApplyDefaultFilterOnSearch } = settingsStore
const dashboardModeLabel = computed(() => getMatchModeLabel(lastMatchMode.value))

const { onSearch, currentIndex, names, searchText, loading, currentResult, filteredCurrentMatches } = useSearchMatches()

const searchPositionAnalysis = computed(() => currentResult.value?.positionAnalysis ?? null)

const emptyTodayMatches = { total: 0, wins: 0, losses: 0 }

const soloRank = computed(() => {
  const info = currentResult.value?.summonerInfo
  return buildSummonerRankPresentation({
    tier: info?.soloRankTier,
    division: info?.soloRankDivision,
    leaguePoints: info?.soloRankLp,
    wins: info?.soloRankWins,
    losses: info?.soloRankLosses
  })
})

const flexRank = computed(() => {
  const info = currentResult.value?.summonerInfo
  return buildSummonerRankPresentation({
    tier: info?.flexRankTier,
    division: info?.flexRankDivision,
    leaguePoints: info?.flexRankLp,
    wins: info?.flexRankWins,
    losses: info?.flexRankLosses
  })
})
</script>
