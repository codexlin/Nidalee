import type { AutoFunctions } from '@/shared/stores/features/autoFunctionStore'
import { useChampSelect } from './useChampSelect'

interface ExecutedChampSelectActions {
  banChampion: boolean
  selectChampion: boolean
  lockInProgress: boolean
}

interface UseChampSelectAutomationOptions {
  getCurrentSession: () => ChampSelectSession | null
  isChampSelectActive: () => boolean
}

type ActionType = 'ban' | 'pick'

export function useChampSelectAutomation({ getCurrentSession, isChampSelectActive }: UseChampSelectAutomationOptions) {
  const { pickChampion, banChampion } = useChampSelect()
  let generation = 0
  let banTimer: ReturnType<typeof setTimeout> | null = null
  let pickTimer: ReturnType<typeof setTimeout> | null = null
  let lockTimer: ReturnType<typeof setTimeout> | null = null
  let pendingBanActionId: number | null = null
  let pendingPickActionId: number | null = null

  const clearTimer = (timer: ReturnType<typeof setTimeout> | null) => {
    if (timer !== null) clearTimeout(timer)
  }

  const cancelAutoActions = () => {
    generation += 1
    clearTimer(banTimer)
    clearTimer(pickTimer)
    clearTimer(lockTimer)
    banTimer = null
    pickTimer = null
    lockTimer = null
    pendingBanActionId = null
    pendingPickActionId = null
  }

  const resetExecutedActions = (executed: ExecutedChampSelectActions) => {
    executed.banChampion = false
    executed.selectChampion = false
    executed.lockInProgress = false
  }

  const findCurrentAction = (actionId: number, type: ActionType) => {
    if (!isChampSelectActive()) return null
    const session = getCurrentSession()
    if (!session) return null

    return (
      session.actions
        .flat()
        .find(
          (action) =>
            action.id === actionId &&
            action.actorCellId === session.localPlayerCellId &&
            action.type === type &&
            action.isInProgress === true &&
            !action.completed
        ) ?? null
    )
  }

  const isChampionAvailable = (championId: number, currentActionId: number) => {
    const session = getCurrentSession()
    if (!session) return false

    return !session.actions.flat().some((action) => {
      if (action.id === currentActionId || action.championId !== championId) return false
      return action.completed || (action.type === 'pick' && action.isInProgress === true)
    })
  }

  const isActionCurrent = (actionId: number, type: ActionType, championId: number) => {
    const action = findCurrentAction(actionId, type)
    if (!action) return false
    if (action.championId && action.championId !== championId) return false
    return isChampionAvailable(championId, actionId)
  }

  const reconcilePendingActions = (session: ChampSelectSession, executed: ExecutedChampSelectActions) => {
    const localActionId =
      session.actions
        .flat()
        .find(
          (action) =>
            action.actorCellId === session.localPlayerCellId && action.isInProgress === true && !action.completed
        )?.id ?? null
    const hasStaleAction =
      (pendingBanActionId !== null && pendingBanActionId !== localActionId) ||
      (pendingPickActionId !== null && pendingPickActionId !== localActionId)

    if (hasStaleAction) {
      cancelAutoActions()
      resetExecutedActions(executed)
    }
  }

  const collectUnavailableChampions = (session: ChampSelectSession) => {
    const picked = new Set<number>()
    const banned = new Set<number>()
    const intended = new Set<number>()

    for (const action of session.actions.flat()) {
      const championId = action.championId
      if (!championId || championId <= 0) continue

      if (action.completed) {
        if (action.type === 'pick') picked.add(championId)
        if (action.type === 'ban') banned.add(championId)
      } else if (action.type === 'pick' && action.isInProgress === true) {
        intended.add(championId)
      }
    }

    return { picked, banned, intended }
  }

  const scheduleBan = (
    action: ChampSelectAction,
    autoFunctions: AutoFunctions,
    unavailable: ReturnType<typeof collectUnavailableChampions>,
    executed: ExecutedChampSelectActions
  ) => {
    if (executed.banChampion || action.completed || !autoFunctions.banChampion.enabled) return false

    const champion = autoFunctions.banChampion.championList.find(
      (candidate) =>
        !unavailable.picked.has(candidate.id) &&
        !unavailable.banned.has(candidate.id) &&
        !unavailable.intended.has(candidate.id)
    )
    if (!champion) return false

    executed.banChampion = true
    pendingBanActionId = action.id
    const taskGeneration = generation

    banTimer = setTimeout(async () => {
      banTimer = null
      const config = autoFunctions.banChampion
      const canRun =
        taskGeneration === generation &&
        config.enabled &&
        config.championList.some((candidate) => candidate.id === champion.id) &&
        isActionCurrent(action.id, 'ban', champion.id)

      if (!canRun) {
        if (taskGeneration === generation) {
          pendingBanActionId = null
          executed.banChampion = false
        }
        return
      }

      try {
        await banChampion(action.id, champion.id)
        if (taskGeneration === generation) console.log('[🤖 AutoChampSelect] ✅ 自动禁用成功')
      } catch (error) {
        if (taskGeneration === generation) {
          console.error('[🤖 AutoChampSelect] ❌ 自动禁用失败:', error)
          executed.banChampion = false
        }
      } finally {
        if (taskGeneration === generation) pendingBanActionId = null
      }
    }, autoFunctions.banChampion.delay ?? 500)

    return true
  }

  const schedulePick = (
    action: ChampSelectAction,
    autoFunctions: AutoFunctions,
    unavailable: ReturnType<typeof collectUnavailableChampions>,
    executed: ExecutedChampSelectActions
  ) => {
    if (executed.selectChampion || executed.lockInProgress || action.completed || !autoFunctions.selectChampion.enabled)
      return false

    const champion = autoFunctions.selectChampion.championList.find(
      (candidate) =>
        !unavailable.picked.has(candidate.id) &&
        !unavailable.banned.has(candidate.id) &&
        !unavailable.intended.has(candidate.id)
    )
    if (!champion) return false

    const releasePendingPick = (taskGeneration: number) => {
      if (taskGeneration !== generation) return
      pendingPickActionId = null
      executed.lockInProgress = false
    }

    executed.lockInProgress = true
    pendingPickActionId = action.id
    const taskGeneration = generation

    pickTimer = setTimeout(async () => {
      pickTimer = null
      const config = autoFunctions.selectChampion
      const canHover =
        taskGeneration === generation &&
        config.enabled &&
        config.championList.some((candidate) => candidate.id === champion.id) &&
        isActionCurrent(action.id, 'pick', champion.id)

      if (!canHover) {
        releasePendingPick(taskGeneration)
        return
      }

      try {
        await pickChampion(action.id, champion.id, false)
        if (taskGeneration !== generation || !isActionCurrent(action.id, 'pick', champion.id)) {
          releasePendingPick(taskGeneration)
          return
        }

        lockTimer = setTimeout(async () => {
          lockTimer = null
          const canLock =
            taskGeneration === generation &&
            config.enabled &&
            config.championList.some((candidate) => candidate.id === champion.id) &&
            isActionCurrent(action.id, 'pick', champion.id)

          if (!canLock) {
            releasePendingPick(taskGeneration)
            return
          }

          try {
            await pickChampion(action.id, champion.id, true)
            if (taskGeneration === generation) executed.selectChampion = true
          } catch (error) {
            if (taskGeneration === generation) console.error('[🤖 AutoChampSelect] ❌ 自动锁定失败:', error)
          } finally {
            releasePendingPick(taskGeneration)
          }
        }, 1000)
      } catch (error) {
        if (taskGeneration === generation) console.error('[🤖 AutoChampSelect] ❌ 自动Hover失败:', error)
        releasePendingPick(taskGeneration)
      }
    }, autoFunctions.selectChampion.delay ?? 500)

    return true
  }

  const checkAndScheduleAutoActions = (
    session: ChampSelectSession,
    autoFunctions: AutoFunctions,
    executed: ExecutedChampSelectActions
  ) => {
    if (session.timer.phase !== 'BAN_PICK') {
      cancelAutoActions()
      resetExecutedActions(executed)
      return false
    }

    reconcilePendingActions(session, executed)
    const action = session.actions
      .flat()
      .find(
        (candidate) =>
          candidate.actorCellId === session.localPlayerCellId && candidate.isInProgress === true && !candidate.completed
      )
    if (!action) return false
    if (action.championId && action.championId > 0) return false

    const unavailable = collectUnavailableChampions(session)
    if (action.type === 'ban') return scheduleBan(action, autoFunctions, unavailable, executed)
    if (action.type === 'pick') return schedulePick(action, autoFunctions, unavailable, executed)
    return false
  }

  return {
    checkAndScheduleAutoActions,
    cancelAutoActions
  }
}
