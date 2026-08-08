/**
 * 选人与房间管理 Composable
 * 职责：
 * - 处理房间（Lobby）相关的事件和日志
 * - 处理选人会话（ChampSelect）并触发自动选人
 */
export function useChampSelectManager() {
  const gameStore = useGameStore()
  const activityLogger = useActivityLogger()

  // 房间变更处理
  const handleLobbyChange = (lobby: LobbyInfo | null) => {
    gameStore.updateLobbyInfo(lobby)
    if (lobby) {
      activityLogger.log.info(`进入房间: ${lobby.partyType} (${lobby.members.length}人)`, 'game')
    } else {
      activityLogger.log.info('离开房间', 'game')
    }
  }

  // 选人会话变更处理（触发自动选人）
  const handleChampSelectChange = (session: ChampSelectSession | null) => {
    if (!session) {
      console.log('[ChampSelectManager] 选人会话已清空')
      gameStore.updateChampSelectSession(null)
      return
    }

    console.log('[ChampSelectManager] 选人会话更新，触发自动选人检查')
    gameStore.updateChampSelectSession(session)
  }

  return {
    handleLobbyChange,
    handleChampSelectChange
  }
}
