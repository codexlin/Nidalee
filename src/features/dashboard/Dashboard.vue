<template>
  <div class="flex flex-col gap-4" v-if="isConnected">
    <div v-if="loading" className="flex w-auto min-h-screen items-center justify-center gap-6">
      <Spinner class="size-6 text-primary" />
    </div>
    <template v-else>
      <SummonerCard :summoner-info="summonerInfo" is-dashboard />
      <StatisticsCards
        :is-connected="isConnected"
        :today-matches="todayMatches"
        :win-rate="winRate"
        :enabled-functions-count="enabledFunctionsCount"
      />

      <GameStats
        :is-connected="isConnected"
        :match-history-loading="matchHistoryLoading"
        :match-statistics="matchStatistics"
        :selected-queue-id="selectedQueueId"
        @fetch-match-history="handleFetchMatchHistory"
        @queue-change="handleQueueChange"
      />

      <!-- ⭐ v3.4: 位置分组统计 -->
      <PositionStatsCard
        v-if="positionAnalysis && !matchHistoryLoading"
        :position-stats="positionAnalysis.positionStats"
        :main-position="positionAnalysis.mainPosition"
        @view-details="handlePositionDetails"
      />

      <!-- 建议现在集成在位置详情中 -->

      <!-- 位置详情对话框 -->
      <PositionDetailsDialog
        v-if="selectedPosition"
        :open="showPositionDetails"
        :position-data="selectedPosition"
        @close="closePositionDetails"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import PositionStatsCard from '@/features/match-search/PositionStatsCard.vue'
import PositionDetailsDialog from '@/features/match-search/PositionDetailsDialog.vue'
import { AnalysisMode } from '@/shared/stores/features/analysisSettingsStore'

const { loading, toggle } = useLoading()
const { updateMatchHistory } = useSummonerAndMatchUpdater()

const dataStore = useDataStore()
const connectionStore = useConnectionStore()
const activityLogger = useActivityLogger()
const autoFunctionStore = useAutoFunctionStore()

const { summonerInfo, matchStatistics, isDataLoading } = storeToRefs(dataStore)
const { isConnected } = storeToRefs(connectionStore)
const { enabledFunctionsCount } = storeToRefs(autoFunctionStore)

// 当前选中的队列ID（null = 全部模式）
const selectedQueueId = ref<number | null>(null)

// ⭐ v3.4: 位置分析
const { positionAnalysis, selectedPosition, fetchPositionAnalysis, selectPosition, clearSelectedPosition } =
  usePositionAnalysis()

const showPositionDetails = ref(false)

// 当前选中的分析模式
const selectedAnalysisMode = ref<AnalysisMode>(AnalysisMode.MixedRanked)

// ✅ 直接从后端获取已计算好的数据
const todayMatches = computed(() => ({
  total: matchStatistics.value?.todayGames || 0,
  wins: matchStatistics.value?.todayWins || 0,
  losses: (matchStatistics.value?.todayGames || 0) - (matchStatistics.value?.todayWins || 0)
}))

const winRate = computed(() => matchStatistics.value?.winRate || 0)

const handleFetchMatchHistory = async () => {
  toggle()
  activityLogger.log.info('手动刷新对局历史', 'data')
  await updateMatchHistory(selectedQueueId.value)
  // 同时刷新位置分析（使用用户选择的模式）
  await fetchPositionAnalysis(30, selectedAnalysisMode.value)
  toggle()
}

const handleQueueChange = async (queueId: number | null) => {
  toggle()
  selectedQueueId.value = queueId

  let nextMode: AnalysisMode
  if (queueId === null) {
    nextMode = AnalysisMode.MixedRanked
  } else {
    switch (queueId) {
      case 420:
        nextMode = AnalysisMode.SoloRanked
        break
      case 440:
        nextMode = AnalysisMode.FlexRanked
        break
      case 450:
        nextMode = AnalysisMode.Aram
        break
      default:
        nextMode = AnalysisMode.AllModes
        break
    }
  }

  selectedAnalysisMode.value = nextMode
  activityLogger.log.info(`切换队列类型: ${queueId || '全部'}`, 'data')
  await updateMatchHistory(queueId)
  await fetchPositionAnalysis(30, nextMode)
  toggle()
}

// 处理分析模式切换（暂时保留，可能后续需要）
// const handleAnalysisModeChange = async (mode: AnalysisMode) => {
//   selectedAnalysisMode.value = mode
//   activityLogger.log.info(`切换分析模式: ${mode}`, 'data')
//   await fetchPositionAnalysis(30, mode)
// }

// 查看位置详情
const handlePositionDetails = (pos: PositionStats) => {
  selectPosition(pos)
  showPositionDetails.value = true
}

// 关闭位置详情
const closePositionDetails = () => {
  showPositionDetails.value = false
  setTimeout(() => {
    clearSelectedPosition()
  }, 300)
}

const matchHistoryLoading = computed(() => isDataLoading.value)

// 初始加载位置分析
watch(
  () => isConnected.value,
  async (connected) => {
    if (connected && !positionAnalysis.value) {
      await fetchPositionAnalysis(30, selectedAnalysisMode.value)
    }
  },
  { immediate: true }
)
</script>
