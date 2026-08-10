// 专门处理游戏阶段变化的逻辑
export function useGamePhaseManager() {
  const gameStore = useGameStore()
  const activityLogger = useActivityLogger()
  const autoFunctionStore = useAutoFunctionStore()
  const { handleAcceptMatch } = useMatchmaking()
  const { updateSummonerAndMatches } = useSummonerAndMatchUpdater()
  const router = useRouter()
  let acceptTimer: ReturnType<typeof setTimeout> | null = null

  const cancelPendingAutoAccept = () => {
    if (acceptTimer !== null) {
      clearTimeout(acceptTimer)
      acceptTimer = null
    }
  }

  // 游戏阶段变更处理
  const handleGamePhaseChange = (phase: string) => {
    const nextPhase = phase || 'None'
    const previousPhase = gameStore.currentPhase
    gameStore.updateGamePhase(nextPhase)
    if (nextPhase !== 'ReadyCheck') cancelPendingAutoAccept()
    if (previousPhase === nextPhase) return

    console.log('[🎮 GamePhaseManager] ===== 游戏阶段变更 =====')
    console.log('[🎮 GamePhaseManager] 上一个阶段:', previousPhase)
    console.log('[🎮 GamePhaseManager] 当前阶段:', nextPhase)
    console.log('[🎮 GamePhaseManager] 阶段变更时间:', new Date().toLocaleTimeString())

    if (nextPhase === 'ReadyCheck') handleAutoAcceptMatch()

    switch (nextPhase) {
      case 'None':
        activityLogger.log.info('返回客户端主界面', 'game')
        break
      case 'Lobby':
        activityLogger.log.info('进入房间', 'game')
        gameStore.clearChampSelect()
        break
      case 'Matchmaking':
        activityLogger.log.info('进入队列匹配中', 'game')
        gameStore.clearChampSelect()
        if (router.currentRoute.value.name !== 'match-analysis') {
          console.log('[🎮 GamePhaseManager] 开始匹配，自动跳转到对局分析页面')
          void router.push({ name: 'match-analysis' })
        }
        break
      case 'ReadyCheck':
        activityLogger.log.success('找到对局，等待接受', 'game')
        gameStore.clearChampSelect()
        break
      case 'ChampSelect':
        activityLogger.log.info('进入英雄选择阶段', 'game')
        break
      case 'InProgress':
        activityLogger.log.success('游戏开始', 'game')
        break
      case 'WaitingForStats':
        activityLogger.log.info('游戏结束', 'game')
        break
    }

    if (previousPhase === 'InProgress' && nextPhase !== 'InProgress') {
      console.log('[🎮 GamePhaseManager] 🏁 检测到游戏退出，清理选人和房间状态')
      gameStore.clearChampSelect()
      gameStore.updateLobbyInfo(null)
      activityLogger.log.info('游戏已结束，已清理游戏状态', 'game')
      void updateSummonerAndMatches()
    }
    console.log('[🎮 GamePhaseManager] ===== 阶段变更处理完成 =====\n')
  }

  const handleAutoAcceptMatch = () => {
    const { autoFunctions } = autoFunctionStore

    if (autoFunctions.acceptMatch.enabled) {
      console.log('[🤖 GamePhaseManager] ✅ 自动接受对局已启用，延迟', autoFunctions.acceptMatch.delay, 'ms后执行')

      cancelPendingAutoAccept()
      acceptTimer = setTimeout(async () => {
        acceptTimer = null
        if (gameStore.currentPhase !== 'ReadyCheck' || !autoFunctionStore.autoFunctions.acceptMatch.enabled) {
          return
        }

        try {
          console.log('[🤖 GamePhaseManager] 🚀 开始执行自动接受对局')
          await handleAcceptMatch()
          console.log('[🤖 GamePhaseManager] ✅ 自动接受对局执行成功')
          activityLogger.logAutoFunction.acceptMatch.success()
        } catch (error) {
          console.error('[🤖 GamePhaseManager] ❌ 自动接受对局失败:', error)
          activityLogger.logAutoFunction.acceptMatch.failed(String(error))
        }
      }, autoFunctions.acceptMatch.delay)
    } else {
      console.log('[🤖 GamePhaseManager] ⚪ 自动接受对局未启用')
    }
  }
  return {
    handleGamePhaseChange,
    cancelPendingAutoAccept
  }
}
