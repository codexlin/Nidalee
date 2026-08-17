<template>
  <div class="flex flex-col gap-4" v-if="isConnected">
    <div v-if="pageLoading" class="flex min-h-screen w-auto items-center justify-center gap-6">
      <Spinner class="size-6 text-primary" />
    </div>
    <template v-else>
      <CompactProfileHeader
        :is-connected="isConnected"
        :summoner-info="summonerInfo"
        :today-matches="todayMatches"
        :solo-rank="soloRank"
        :flex-rank="flexRank"
      />

      <!-- Dashboard 不展示策略/降级诊断（开发向）；仅在 AI 可用或已有解读时出现 -->
      <Card v-if="showAiPanel" class="border-dashed">
        <CardContent class="py-3 space-y-2 text-sm">
          <div v-if="showAiAction" class="flex flex-wrap items-center gap-2">
            <Button size="sm" variant="outline" :disabled="aiLoading || matchHistoryLoading" @click="runAiInsight">
              {{ aiLoading ? 'AI 解读中…' : '生成 AI 解读' }}
            </Button>
            <span v-if="aiError" class="text-xs text-destructive">{{ aiError }}</span>
          </div>
          <div
            v-if="aiInsight"
            class="space-y-2"
            :class="showAiAction ? 'pt-1 border-t border-dashed border-border' : ''"
          >
            <p class="font-medium text-foreground">{{ aiInsight.summary }}</p>
            <p class="text-xs text-muted-foreground">置信度 {{ Math.round(aiInsight.confidence * 100) }}%</p>
            <ul v-if="aiInsight.findings?.length" class="list-disc pl-4 space-y-1 text-muted-foreground">
              <li v-for="(f, i) in aiInsight.findings" :key="i">
                <span class="text-foreground">{{ f.title }}</span> — {{ f.detail }}
              </li>
            </ul>
            <ul v-if="aiInsight.suggestions?.length" class="list-disc pl-4 space-y-1 text-muted-foreground">
              <li v-for="(s, i) in aiInsight.suggestions" :key="`s-${i}`">
                <span class="text-foreground">{{ s.title }}</span>
                <span v-if="s.actions?.length">：{{ s.actions.join('；') }}</span>
              </li>
            </ul>
          </div>
        </CardContent>
      </Card>

      <SummonerPerformancePanel
        :is-connected="isConnected"
        :match-history-loading="matchHistoryLoading"
        :error="personalAnalysis.error"
        :match-statistics="matchStatistics"
        :analysis-traits="analysisResult?.traits"
        :position-stats="analysisResult?.positionStats"
        :main-position="analysisResult?.mainPosition"
        :scope="performanceScope"
        :ai-ready="aiReady"
        :can-export-poster="canExportPoster"
        :poster-exporting="posterExporting"
        @fetch-match-history="handleFetchMatchHistory"
        @scope-change="handleScopeChange"
        @export-poster="handleExportPoster"
        @open-game-detail="openGameDetail"
      />

      <!-- 离屏海报稿：固定宽 720，不截整页 -->
      <div
        v-if="canExportPoster"
        class="pointer-events-none fixed top-0 left-[-10000px] z-[-1] w-180"
        aria-hidden="true"
      >
        <DashboardPoster
          ref="posterRef"
          :summoner-info="summonerInfo"
          :today-matches="todayMatches"
          :solo-rank="soloRank"
          :flex-rank="flexRank"
          :match-statistics="matchStatistics"
          :analysis-traits="analysisResult?.traits"
          :position-stats="analysisResult?.positionStats"
          :main-position="analysisResult?.mainPosition"
          :scope="performanceScope"
          :recent-limit="10"
        />
      </div>

      <GameDetailDialog
        v-model:visible="dialogOpen"
        :selected-game="selectedGame"
        :analysis-puuid="selectedGamePuuid"
        @open-game-detail="openGameDetail"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import CompactProfileHeader from './components/CompactProfileHeader.vue'
import DashboardPoster from './components/DashboardPoster.vue'
import GameDetailDialog from './components/detail/GameDetailDialog.vue'
import SummonerPerformancePanel from '@/features/summoner-performance/SummonerPerformancePanel.vue'
import { performanceScopeKey, type PerformanceScope } from '@/common/performanceScope'
import { useMatchAnalysis } from '@/shared/composables/game/useMatchAnalysis'
import { useAiAnalysis } from '@/shared/composables/game/useAiAnalysis'
import { usePersonalMatchAnalysisStore } from '@/shared/stores/features/personalMatchAnalysisStore'
import { useDashboardPosterExport } from './composables/useDashboardPosterExport'
import { buildSummonerRankPresentation } from '@/shared/utils/summonerRankPresentation'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'

const settingsStore = useSettingsStore()
const personalAnalysis = usePersonalMatchAnalysisStore()
const { analyzeMatches } = useMatchAnalysis()
const { loading: aiLoading, error: aiError, analyzeWithAi, ensureSynced: ensureAiSynced, aiSettings } = useAiAnalysis()
const { exporting: posterExporting, exportPoster } = useDashboardPosterExport()

const posterRef = ref<{ getRoot: () => HTMLElement | null } | null>(null)
const dialogOpen = ref(false)
const selectedGame = ref<MatchPerformance | null>(null)
const selectedGamePuuid = ref<string | null>(null)
const isViewActive = ref(false)
let lastEnsuredContextKey: string | null = null

const soloRank = computed(() => {
  const info = summonerInfo.value
  return buildSummonerRankPresentation({
    tier: info?.soloRankTier,
    division: info?.soloRankDivision,
    leaguePoints: info?.soloRankLp,
    wins: info?.soloRankWins,
    losses: info?.soloRankLosses
  })
})

const flexRank = computed(() => {
  const info = summonerInfo.value
  return buildSummonerRankPresentation({
    tier: info?.flexRankTier,
    division: info?.flexRankDivision,
    leaguePoints: info?.flexRankLp,
    wins: info?.flexRankWins,
    losses: info?.flexRankLosses
  })
})

const dataStore = useDataStore()
const connectionStore = useConnectionStore()

const { summonerInfo, isSummonerLoading } = storeToRefs(dataStore)
const { isConnected } = storeToRefs(connectionStore)

const performanceScope = computed(() => settingsStore.performanceScope)
const currentContextKey = computed(() => {
  const puuid = summonerInfo.value?.puuid?.trim()
  if (!isConnected.value || !puuid) return null
  return `${puuid}:${performanceScopeKey(performanceScope.value)}`
})
const analysisResult = computed(() => {
  const puuid = summonerInfo.value?.puuid
  if (!puuid || !personalAnalysis.hasResultFor(puuid, performanceScope.value)) return null
  return personalAnalysis.result
})
const matchStatistics = computed(() => analysisResult.value?.overallStats ?? null)

const capabilities = computed(() => analysisResult.value?.capabilities ?? null)
const displayGames = computed(() => analysisResult.value?.displayGames ?? 0)
const aiInsight = computed(() => analysisResult.value?.aiInsight ?? null)

/** 后端可跑 AI + 用户已开启并填过 Key，才算真正就绪 */
const aiReady = computed(() => !!capabilities.value?.localAi && aiSettings.enabled && aiSettings.hasApiKey)

const showAiAction = computed(() => displayGames.value > 0 && aiReady.value)

const showAiPanel = computed(() => displayGames.value > 0 && (showAiAction.value || !!aiInsight.value))

const todayMatches = computed(() => {
  const total = matchStatistics.value?.todayGames || 0
  const wins = matchStatistics.value?.todayWins || 0
  return {
    total,
    wins,
    losses: Math.max(0, total - wins)
  }
})

const ensureCurrentAnalysis = async (force = false) => {
  const contextKey = currentContextKey.value
  const puuid = summonerInfo.value?.puuid
  if (!isViewActive.value || !contextKey || !puuid) return
  if (!force && personalAnalysis.hasResultFor(puuid, performanceScope.value)) return
  if (!force && lastEnsuredContextKey === contextKey) return

  lastEnsuredContextKey = contextKey
  await analyzeMatches({ scope: performanceScope.value })
}

const handleFetchMatchHistory = async () => {
  await ensureCurrentAnalysis(true)
}

const handleScopeChange = async (scope: PerformanceScope) => {
  settingsStore.setPerformanceScope(scope)
  await ensureCurrentAnalysis(true)
}

const pageLoading = computed(() => isSummonerLoading.value && !summonerInfo.value)
const matchHistoryLoading = computed(() => personalAnalysis.loading)

const canExportPoster = computed(
  () => !!matchStatistics.value && (matchStatistics.value.totalGames || 0) > 0 && !matchHistoryLoading.value
)

const posterFileStem = computed(() => {
  const name = summonerInfo.value?.gameName || summonerInfo.value?.displayName || 'summoner'
  const tag = summonerInfo.value?.tagLine ? `-${summonerInfo.value.tagLine}` : ''
  const safe = `${name}${tag}`.replace(/[<>:"/\\|?*\s]+/g, '_')
  const d = new Date()
  const pad = (n: number) => String(n).padStart(2, '0')
  return `nidalee-${safe}-${d.getFullYear()}${pad(d.getMonth() + 1)}${pad(d.getDate())}`
})

const handleExportPoster = async () => {
  await exportPoster(posterRef.value?.getRoot() ?? null, posterFileStem.value)
}

const runAiInsight = async () => {
  await analyzeWithAi()
}

function openGameDetail(game: MatchPerformance, puuid = summonerInfo.value?.puuid ?? '') {
  const normalizedPuuid = puuid.trim()
  if (!normalizedPuuid) return
  selectedGame.value = game
  selectedGamePuuid.value = normalizedPuuid
  dialogOpen.value = true
}

function closeGameDetail() {
  dialogOpen.value = false
  selectedGame.value = null
  selectedGamePuuid.value = null
}

watch(currentContextKey, (contextKey, previousContextKey) => {
  if (contextKey === previousContextKey) return
  lastEnsuredContextKey = null
  if (isViewActive.value) void ensureCurrentAnalysis()
})

watch(
  () => [isConnected.value, summonerInfo.value?.puuid ?? null] as const,
  ([connected, puuid], [previousConnected, previousPuuid]) => {
    if (!connected || (previousConnected && previousPuuid && puuid !== previousPuuid)) closeGameDetail()
  }
)

onMounted(() => {
  isViewActive.value = true
  void ensureCurrentAnalysis()
  void ensureAiSynced()
})

onActivated(() => {
  if (isViewActive.value) return
  isViewActive.value = true
  lastEnsuredContextKey = null
  void ensureCurrentAnalysis()
})

onDeactivated(() => {
  isViewActive.value = false
})

onUnmounted(() => {
  isViewActive.value = false
})
</script>
