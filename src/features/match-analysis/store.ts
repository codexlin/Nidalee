import { getCompactQueueDisplayName } from '@/common/queueCatalog'
import type { GamePhase } from '@/types/match-analysis'

/**
 * 实时对局分析唯一状态源。
 *
 * 后端每次发布完整的 TeamAnalysisData；前端只替换该快照，其余状态全部派生，
 * 避免玩家列表、战绩数组和队列信息分别更新后产生错位。
 */
export const useMatchAnalysisStore = defineStore('matchAnalysis', () => {
  const gameStore = useGameStore()
  const teamAnalysisData = shallowRef<TeamAnalysisData | null>(null)

  const currentPhase = computed(() => gameStore.currentPhase as GamePhase)
  const shouldShowAnalysis = computed(() => currentPhase.value === 'ChampSelect' || currentPhase.value === 'InProgress')

  const myTeam = computed(() => teamAnalysisData.value?.myTeam ?? [])
  const enemyTeam = computed(() => teamAnalysisData.value?.enemyTeam ?? [])
  const localPlayerCellId = computed(() => teamAnalysisData.value?.localPlayerCellId ?? -1)
  const queueId = computed(() => Number(teamAnalysisData.value?.queueId ?? 0))
  const isCustomGame = computed(() => teamAnalysisData.value?.isCustomGame ?? false)

  const hasMyTeamData = computed(() => myTeam.value.length > 0)
  const hasEnemyTeamData = computed(() => enemyTeam.value.length > 0)
  const isRankedGame = computed(() => queueId.value === 420 || queueId.value === 440)
  const queueTypeLabel = computed(() => {
    if (isCustomGame.value) return '自定义游戏'
    return queueId.value > 0 ? getCompactQueueDisplayName(queueId.value) : '未知模式'
  })

  function setTeamAnalysisData(data: TeamAnalysisData | null): void {
    teamAnalysisData.value = data
  }

  function clearAllData(): void {
    teamAnalysisData.value = null
  }

  function $reset(): void {
    clearAllData()
  }

  return {
    teamAnalysisData,
    currentPhase,
    shouldShowAnalysis,
    myTeam,
    enemyTeam,
    localPlayerCellId,
    queueId,
    isCustomGame,
    hasMyTeamData,
    hasEnemyTeamData,
    isRankedGame,
    queueTypeLabel,
    setTeamAnalysisData,
    clearAllData,
    $reset
  }
})
