<template>
  <div class="flex flex-col gap-4" v-if="isConnected">
    <div v-if="loading" className="flex w-auto min-h-screen items-center justify-center gap-6">
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

      <GameStats
        :is-connected="isConnected"
        :match-history-loading="matchHistoryLoading"
        :match-statistics="matchStatistics"
        :ranked-stats="personalAnalysis.rankedStats"
        :other-stats="personalAnalysis.otherStats"
        :analysis-traits="personalAnalysis.traits"
        :position-stats="positionAnalysis?.positionStats"
        :main-position="positionAnalysis?.mainPosition"
        :selected-match-mode="selectedMatchMode"
        :match-count="selectedMatchCount"
        :remember-preferences="settingsStore.rememberMatchPreferences"
        :scanned-games="selectedMatchCount"
        :ai-ready="aiReady"
        :display-games="displayGames"
        :can-export-poster="canExportPoster"
        :poster-exporting="posterExporting"
        @fetch-match-history="handleFetchMatchHistory"
        @mode-change="handleModeChange"
        @count-change="handleCountChange"
        @remember-change="handleRememberChange"
        @export-poster="handleExportPoster"
      />

      <!-- 离屏海报稿：固定宽 720，不截整页 -->
      <div
        v-if="canExportPoster"
        class="pointer-events-none fixed top-0 -left-[10000px] z-[-1] w-[720px]"
        aria-hidden="true"
      >
        <DashboardPoster
          ref="posterRef"
          :summoner-info="summonerInfo"
          :today-matches="todayMatches"
          :solo-rank="soloRank"
          :flex-rank="flexRank"
          :match-statistics="matchStatistics"
          :analysis-traits="personalAnalysis.traits"
          :position-stats="positionAnalysis?.positionStats"
          :main-position="positionAnalysis?.mainPosition"
          :selected-match-mode="selectedMatchMode"
          :match-count="selectedMatchCount"
          :recent-limit="10"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import CompactProfileHeader from './components/CompactProfileHeader.vue'
import DashboardPoster from './components/DashboardPoster.vue'
import GameStats from './components/GameStats.vue'
import { normalizeMatchModeKey, type MatchModeKey } from '@/common/queueCatalog'
import { useMatchAnalysis } from '@/shared/composables/game/useMatchAnalysis'
import { useAiAnalysis } from '@/shared/composables/game/useAiAnalysis'
import { usePersonalMatchAnalysisStore } from '@/shared/stores/features/personalMatchAnalysisStore'
import { useDashboardPosterExport } from './composables/useDashboardPosterExport'
import { buildSummonerRankPresentation } from '@/shared/utils/summonerRankPresentation'
import { Card, CardContent } from '@/components/ui/card'
import { Button } from '@/components/ui/button'

const { loading, toggle } = useLoading()
const settingsStore = useSettingsStore()
const personalAnalysis = usePersonalMatchAnalysisStore()
const { analyzeMatches } = useMatchAnalysis()
const {
  loading: aiLoading,
  error: aiError,
  analyzeWithAi,
  ensureSynced: ensureAiSynced,
  aiSettings,
  aiInsight
} = useAiAnalysis()
const { exporting: posterExporting, exportPoster } = useDashboardPosterExport()

const posterRef = ref<{ getRoot: () => HTMLElement | null } | null>(null)

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
const activityLogger = useActivityLogger()

const { summonerInfo, matchStatistics, isDataLoading } = storeToRefs(dataStore)
const { isConnected } = storeToRefs(connectionStore)

const resolveInitialMode = (): MatchModeKey => {
  if (!settingsStore.rememberMatchPreferences) return 'all'
  return normalizeMatchModeKey(settingsStore.lastMatchMode)
}

const resolveInitialCount = (): number => {
  const raw = settingsStore.rememberMatchPreferences ? settingsStore.lastMatchCount : 20
  return (settingsStore.allowedMatchCounts as readonly number[]).includes(raw) ? raw : 20
}

const selectedMatchMode = ref<MatchModeKey>(resolveInitialMode())
const selectedMatchCount = ref<number>(resolveInitialCount())
settingsStore.setLastMatchMode(selectedMatchMode.value)
settingsStore.setLastMatchCount(selectedMatchCount.value)

const positionAnalysis = computed(() => personalAnalysis.multiPositionView)
const capabilities = computed(() => personalAnalysis.capabilities)
const displayGames = computed(() => personalAnalysis.result?.displayGames ?? 0)

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

/** 始终同步到 store；刷新只走一次 analyze_matches */
const syncFetchPreferences = () => {
  settingsStore.setLastMatchMode(selectedMatchMode.value)
  settingsStore.setLastMatchCount(selectedMatchCount.value)
}

const refreshAnalysis = async () => {
  syncFetchPreferences()
  await analyzeMatches({ mode: selectedMatchMode.value, count: selectedMatchCount.value })
}

const handleFetchMatchHistory = async () => {
  toggle()
  activityLogger.log.info('手动刷新对局历史', 'data')
  await refreshAnalysis()
  toggle()
}

const handleModeChange = async (mode: MatchModeKey) => {
  toggle()
  selectedMatchMode.value = mode
  activityLogger.log.info(`切换战绩模式: ${mode}`, 'data')
  await refreshAnalysis()
  toggle()
}

const handleCountChange = async (count: number) => {
  toggle()
  selectedMatchCount.value = count
  activityLogger.log.info(`切换对局数量: ${count}`, 'data')
  await refreshAnalysis()
  toggle()
}

const handleRememberChange = (enabled: boolean) => {
  settingsStore.setRememberMatchPreferences(enabled)
  syncFetchPreferences()
  activityLogger.log.info(enabled ? '已开启记住战绩选择' : '已关闭记住战绩选择（下次启动恢复默认）', 'data')
}

const matchHistoryLoading = computed(() => isDataLoading.value || personalAnalysis.loading)

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
  activityLogger.log.info('导出 Dashboard 战绩海报', 'data')
  await exportPoster(posterRef.value?.getRoot() ?? null, posterFileStem.value)
}

const runAiInsight = async () => {
  await analyzeWithAi()
}

onMounted(() => {
  void ensureAiSynced()
})
</script>
