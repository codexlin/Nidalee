import type { GamePhase, TeamData, EnrichedPlayerMatchStats } from '@/types/match-analysis'
import { getCompactQueueDisplayName } from '@/common/queueCatalog'

/**
 * 对局分析 Store
 * 职责：管理对局分析相关的核心状态和数据
 */
export const useMatchAnalysisStore = defineStore('matchAnalysis', () => {
  const gameStore = useGameStore()
  const connectionStore = useConnectionStore()

  // === 核心状态（从其他 store 派生） ===
  const currentPhase = computed(() => gameStore.currentPhase as GamePhase)
  const isConnected = computed(() => connectionStore.isConnected)

  // === 数据状态 ===
  const myTeamData = ref<TeamData | null>(null)
  const myTeamStats = ref<(EnrichedPlayerMatchStats | null)[]>([])
  const enemyTeamData = ref<TeamData | null>(null)
  const enemyTeamStats = ref<(EnrichedPlayerMatchStats | null)[]>([])

  // === 对局信息 ===
  const queueId = ref<number>(0) // 队列类型ID：420=单排, 440=灵活排位, 450=大乱斗等
  const isCustomGame = ref<boolean>(false) // 是否自定义游戏

  // === 控制状态 ===
  const isLoading = ref(false)
  const isEnemyTeamLoading = ref(false) // 专门用于敌方队伍的 Loading 状态
  const phase = ref<GamePhase>('None')
  const isDestroyed = ref(false)

  // === 计算属性（Getters） ===
  const shouldShowAnalysis = computed(() => {
    // 使用 currentPhase (gameStore 的阶段) 而不是内部 phase 状态
    return currentPhase.value === 'ChampSelect' || currentPhase.value === 'InProgress'
  })

  const hasMyTeamData = computed(() => {
    const result = myTeamData.value !== null && myTeamData.value.players.length > 0
    console.log('[MatchAnalysisStore] hasMyTeamData 计算:', {
      myTeamDataIsNull: myTeamData.value === null,
      playersLength: myTeamData.value?.players.length,
      result
    })
    return result
  })

  const hasEnemyTeamData = computed(() => {
    return enemyTeamData.value !== null && enemyTeamData.value.players.length > 0
  })

  // 对局类型标签和图标
  const queueTypeLabel = computed(() => {
    if (isCustomGame.value) return '自定义游戏'
    return queueId.value > 0 ? getCompactQueueDisplayName(queueId.value) : '未知模式'
  })

  const queueTypeIcon = computed(() => {
    if (isCustomGame.value) return '🎮'
    switch (queueId.value) {
      case 420:
      case 440:
        return '🏆'
      case 450:
        return '❄️'
      case 400:
      case 430:
        return '🎲'
      case 900:
        return '🔥'
      case 1020:
        return '♟️'
      case 4310:
        return '⚔️'
      default:
        return '🎮'
    }
  })

  const isRankedGame = computed(() => {
    return queueId.value === 420 || queueId.value === 440
  })

  // === Actions（数据操作方法） ===
  const setTeamAnalysisData = (data: TeamAnalysisData | null) => {
    if (data) {
      console.log('[MatchAnalysisStore] 更新战绩分析数据... data:', data)

      // 1. 转换并更新队伍数据和战绩统计
      myTeamData.value = {
        players: data.myTeam.map((p) => ({ ...p, spells: [p.spell1Id || 0, p.spell2Id || 0] as [number, number] })),
        localPlayerCellId: data.localPlayerCellId
      }
      enemyTeamData.value = {
        players: data.enemyTeam.map((p) => ({ ...p, spells: [p.spell1Id || 0, p.spell2Id || 0] as [number, number] })),
        localPlayerCellId: -1
      }

      // 保留与 players 同序的槽位；附带 puuid/cellId 供 TeamAnalysisCard 匹配
      myTeamStats.value = data.myTeam.map((p) =>
        p.matchStats
          ? {
              displayName: p.displayName,
              puuid: p.puuid,
              cellId: p.cellId,
              ...p.matchStats
            }
          : null
      ) as (EnrichedPlayerMatchStats | null)[]

      enemyTeamStats.value = data.enemyTeam.map((p) =>
        p.matchStats
          ? {
              displayName: p.displayName,
              puuid: p.puuid,
              cellId: p.cellId,
              ...p.matchStats
            }
          : null
      ) as (EnrichedPlayerMatchStats | null)[]

      // 2. 更新队列信息
      queueId.value = Number(data.queueId)
      isCustomGame.value = data.isCustomGame

      console.log('[MatchAnalysisStore] ✅ 战绩分析数据更新完成')
    } else {
      // 如果数据为 null，则清空所有数据
      clearAllData()
    }
  }

  const setPhase = (newPhase: GamePhase) => {
    phase.value = newPhase
  }

  const setLoading = (loading: boolean) => {
    isLoading.value = loading
  }

  const setEnemyTeamLoading = (loading: boolean) => {
    isEnemyTeamLoading.value = loading
  }

  const setDestroyed = (destroyed: boolean) => {
    isDestroyed.value = destroyed
  }

  // === 单独设置方法（用于 InProgress 阶段的增量更新） ===
  const setMyTeamData = (data: TeamData | null) => {
    console.log('[MatchAnalysisStore] setMyTeamData:', data)
    myTeamData.value = data
  }

  const setMyTeamStats = (stats: (EnrichedPlayerMatchStats | null)[]) => {
    console.log('[MatchAnalysisStore] setMyTeamStats:', stats?.length)
    myTeamStats.value = stats
  }

  const setEnemyTeamData = (data: TeamData | null) => {
    console.log('[MatchAnalysisStore] setEnemyTeamData:', data)
    enemyTeamData.value = data
  }

  const setEnemyTeamStats = (stats: (EnrichedPlayerMatchStats | null)[]) => {
    console.log('[MatchAnalysisStore] setEnemyTeamStats:', stats?.length)
    enemyTeamStats.value = stats
  }

  const setQueueInfo = (queue: number, isCustom: boolean) => {
    console.log('[MatchAnalysisStore] setQueueInfo:', queue, isCustom)
    queueId.value = queue
    isCustomGame.value = isCustom
  }

  // === 清理方法 ===
  const clearAllData = () => {
    console.log('[MatchAnalysisStore] 清理所有数据')
    myTeamData.value = null
    myTeamStats.value = []
    enemyTeamData.value = null
    enemyTeamStats.value = []
    queueId.value = 0
    isCustomGame.value = false
    phase.value = 'None'
    isLoading.value = false
    isEnemyTeamLoading.value = false
  }

  const $reset = () => {
    console.log('[MatchAnalysisStore] 重置 Store')
    clearAllData()
    isDestroyed.value = false
  }

  return {
    // 状态
    currentPhase,
    isConnected,
    isLoading,
    isEnemyTeamLoading,
    phase,
    isDestroyed,

    // 数据
    myTeamData,
    myTeamStats,
    enemyTeamData,
    enemyTeamStats,

    // 对局信息
    queueId,
    isCustomGame,

    // 计算属性
    shouldShowAnalysis,
    hasMyTeamData,
    hasEnemyTeamData,
    queueTypeLabel,
    queueTypeIcon,
    isRankedGame,

    // Actions
    setTeamAnalysisData,
    setMyTeamData,
    setMyTeamStats,
    setEnemyTeamData,
    setEnemyTeamStats,
    setQueueInfo,
    setPhase,
    setLoading,
    setEnemyTeamLoading,
    setDestroyed,
    clearAllData,
    $reset
  }
})
