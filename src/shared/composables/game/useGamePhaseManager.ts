import { APP_ROUTES } from '@/router/appRoutes'

// 专门处理游戏阶段变化的逻辑
export function useGamePhaseManager() {
  const gameStore = useGameStore()
  const autoFunctionStore = useAutoFunctionStore()
  const { handleAcceptMatch } = useMatchmaking()
  const { preparePostGameRefresh, refreshMatchesAfterGame } = useSummonerAndMatchUpdater()
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
        void refreshMatchesAfterGame()
        break
      case 'Lobby':
        gameStore.clearChampSelect()
        break
      case 'Matchmaking':
        gameStore.clearChampSelect()
        if (router.currentRoute.value.name !== APP_ROUTES.liveAnalysis.name) {
          console.log('[🎮 GamePhaseManager] 开始匹配，自动跳转到对局分析页面')
          void router.push({ name: APP_ROUTES.liveAnalysis.name })
        }
        break
      case 'ReadyCheck':
        gameStore.clearChampSelect()
        break
      case 'ChampSelect':
        break
      case 'InProgress':
        break
      case 'WaitingForStats':
        break
      case 'EndOfGame':
        void refreshMatchesAfterGame()
        break
    }

    if (previousPhase === 'InProgress' && nextPhase !== 'InProgress') {
      console.log('[🎮 GamePhaseManager] 🏁 检测到游戏退出，清理选人和房间状态')
      gameStore.clearChampSelect()
      gameStore.updateLobbyInfo(null)
      preparePostGameRefresh()
      if (nextPhase === 'EndOfGame' || nextPhase === 'None') {
        void refreshMatchesAfterGame()
      }
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
        } catch (error) {
          console.error('[🤖 GamePhaseManager] ❌ 自动接受对局失败:', error)
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
