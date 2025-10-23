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

      <!-- ⭐ v3.0: 智能建议面板 -->
      <AdvicePanel
        v-if="matchStatistics && !matchHistoryLoading"
        :advice="filteredAdvice"
        :perspective="selectedPerspective"
        :title="advicePanelTitle"
        :subtitle="advicePanelSubtitle"
        @perspective-change="handlePerspectiveChange"
      />

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

// 当前选中的建议视角
const selectedPerspective = ref<'self-improvement' | 'targeting' | 'collaboration'>('self-improvement')

// ⭐ v3.4: 位置分析
const { positionAnalysis, selectedPosition, fetchPositionAnalysis, selectPosition, clearSelectedPosition } =
  usePositionAnalysis()

const showPositionDetails = ref(false)

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
  // 同时刷新位置分析
  await fetchPositionAnalysis(30, selectedQueueId.value)
  toggle()
}

const handleQueueChange = async (queueId: number | null) => {
  toggle()
  selectedQueueId.value = queueId
  activityLogger.log.info(`切换队列类型: ${queueId || '全部'}`, 'data')
  await updateMatchHistory(queueId)
  // 同时刷新位置分析
  await fetchPositionAnalysis(30, queueId)
  toggle()
}

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

const handlePerspectiveChange = (perspective: 'self-improvement' | 'targeting' | 'collaboration') => {
  selectedPerspective.value = perspective
  activityLogger.log.info(`切换建议视角: ${perspective}`, 'data')
}

const matchHistoryLoading = computed(() => isDataLoading.value)

// 根据视角过滤建议
const filteredAdvice = computed(() => {
  if (!matchStatistics.value?.advice) return []

  // 目前后端只生成self-improvement视角的建议
  // 这里只是前端准备好了切换逻辑，后端需要扩展支持
  return matchStatistics.value.advice.filter(
    (advice: any) =>
      advice.perspective ===
      (selectedPerspective.value === 'self-improvement'
        ? 'SelfImprovement'
        : selectedPerspective.value === 'targeting'
          ? 'Targeting'
          : 'Collaboration')
  )
})

// 动态标题和副标题
const advicePanelTitle = computed(() => {
  switch (selectedPerspective.value) {
    case 'self-improvement':
      return '💡 提升建议'
    case 'targeting':
      return '🎯 战术建议'
    case 'collaboration':
      return '🤝 协作建议'
    default:
      return '💡 智能建议'
  }
})

const advicePanelSubtitle = computed(() => {
  switch (selectedPerspective.value) {
    case 'self-improvement':
      return '基于你的近20场数据分析，帮助你变得更强'
    case 'targeting':
      return '识别对手弱点，制定针对性战术'
    case 'collaboration':
      return '了解队友特点，优化团队配合'
    default:
      return '基于你的近期数据分析'
  }
})

// 初始加载位置分析
watch(
  () => isConnected.value,
  async (connected) => {
    if (connected && !positionAnalysis.value) {
      await fetchPositionAnalysis(30, null)
    }
  },
  { immediate: true }
)
</script>
