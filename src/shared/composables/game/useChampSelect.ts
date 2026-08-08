import { invoke } from '@tauri-apps/api/core'
import type { AutoFunctions } from '@/shared/stores/features/autoFunctionStore'

export function useChampSelect() {
  /**
   * 获取当前选人会话信息
   * @returns Promise<ChampSelectSession> 选人会话数据
   */
  async function getChampSelectSession(): Promise<ChampSelectSession> {
    try {
      console.log('[🔍 ChampSelect] 📡 开始获取选人会话信息...')
      const session = await invoke<ChampSelectSession>('get_champ_select_session')
      console.log('[🔍 ChampSelect] ✅ 获取选人会话成功')
      return session
    } catch (error) {
      console.error('[🔍 ChampSelect] ❌ 获取选人会话失败:', error)
      throw new Error(`获取选人会话失败: ${error}`)
    }
  }

  /**
   * 选择英雄 (hover 或 lock)
   * @param actionId 操作ID
   * @param championId 英雄ID
   * @param completed 是否完成(锁定)
   */
  async function pickChampion(actionId: number, championId: number, completed: boolean = false) {
    try {
      console.log('[⭐ ChampSelect] 🎯 选择英雄操作:', {
        actionId,
        championId,
        completed
      })

      const result = await invoke('pick_champion', { actionId, championId, completed })
      console.log('pick_champion:', result)
      console.log(`[⭐ ChampSelect] ✅ ${completed ? '英雄已锁定' : '英雄已Hover'}`)
    } catch (error) {
      console.error('[⭐ ChampSelect] ❌ 选择英雄失败:', error)
      throw new Error(`选择英雄失败: ${error}`)
    }
  }

  /**
   * 禁用英雄
   * @param actionId 操作ID
   * @param championId 英雄ID
   */
  async function banChampion(actionId: number, championId: number) {
    try {
      console.log('[🚫 ChampSelect] 🎯 禁用英雄操作:', { actionId, championId })
      const result = await invoke('ban_champion', { actionId, championId })
      console.log('ban_champion:', result)
      console.log('[🚫 ChampSelect] ✅ 英雄已被禁用')
    } catch (error) {
      console.error('[🚫 ChampSelect] ❌ 禁用英雄失败:', error)
      throw new Error(`禁用英雄失败: ${error}`)
    }
  }

  /**
   * 检查并执行当前玩家的自动操作
   * @param session 选人会话数据
   * @param autoFunctions 自动功能配置
   * @param executedActions 已执行的操作记录
   * @returns 返回是否安排了操作执行
   */
  // 添加原子锁字段
  type ExecutedActions = {
    banChampion: boolean
    selectChampion: boolean
    lockInProgress: boolean
  }
  /**
   * 判断是否需要自动跳转 OPGG 并应用符文
   * @param session 当前选人会话
   * @param autoFunctions 自动功能配置
   * @returns {number | false} 选完英雄返回 championId，否则返回 false
   */
  function getAutoOpggChampionId(session: ChampSelectSession, autoFunctions: AutoFunctions): number | false {
    if (!session?.actions || session.localPlayerCellId === undefined) return false
    if (!autoFunctions?.runeConfig?.enabled) return false

    const localPlayerCellId = session.localPlayerCellId
    const myPickAction = session.actions
      .flat()
      .find((action) => action.actorCellId === localPlayerCellId && action.type === 'pick')

    if (myPickAction?.completed && myPickAction.championId) {
      return myPickAction.championId
    }
    return false
  }
  async function checkAndExecuteAutoActions(
    session: ChampSelectSession,
    autoFunctions: AutoFunctions,
    executedActions: ExecutedActions
  ): Promise<boolean> {
    console.log('[🤖 AutoChampSelect] ===== 检查自动选人操作 =====')
    console.log('[🤖 AutoChampSelect] 会话阶段:', session?.timer?.phase)
    console.log('[🤖 AutoChampSelect] 已执行操作:', executedActions)

    if (session?.timer?.phase === 'FINALIZATION') {
      executedActions.banChampion = false
      executedActions.selectChampion = false
      executedActions.lockInProgress = false
      return false
    }

    if (!session?.actions || session.localPlayerCellId === undefined) {
      console.log('[🤖 AutoChampSelect] ⚠️ 缺少必要的会话数据')
      return false
    }

    const pickedChampionIds = new Set<number>()
    const bannedChampionIds = new Set<number>()
    const preselectedChampionIds = new Set<number>()

    for (const group of session.actions) {
      for (const action of group) {
        if (action.completed && action.championId && action.championId > 0) {
          if (action.type === 'pick') pickedChampionIds.add(action.championId)
          if (action.type === 'ban') bannedChampionIds.add(action.championId)
        }
        if (action.type === 'pick' && action.isInProgress && action.championId && action.championId > 0) {
          preselectedChampionIds.add(action.championId)
        }
      }
    }

    const localPlayerCellId = session.localPlayerCellId
    let hasScheduledAction = false

    if (session.timer.phase === 'BAN_PICK') {
      for (const group of session.actions) {
        for (const action of group) {
          if (Number(action.actorCellId) !== localPlayerCellId || !action.isInProgress) continue

          console.log('[🤖 AutoChampSelect] 🎯 当前玩家操作:', action)

          if (
            await tryAutoBan(
              action,
              autoFunctions,
              pickedChampionIds,
              bannedChampionIds,
              preselectedChampionIds,
              executedActions
            )
          ) {
            hasScheduledAction = true
          }

          if (
            await tryAutoPick(
              action,
              autoFunctions,
              pickedChampionIds,
              bannedChampionIds,
              preselectedChampionIds,
              executedActions
            )
          ) {
            hasScheduledAction = true
          }
        }
      }
    }

    console.log('[🤖 AutoChampSelect] ===== 自动选人操作检查完成 =====\n')
    return hasScheduledAction
  }
  async function tryAutoBan(
    action: ChampSelectAction,
    autoFunctions: AutoFunctions,
    picked: Set<number>,
    banned: Set<number>,
    preselected: Set<number>,
    executedActions: ExecutedActions
  ) {
    if (action.type !== 'ban' || executedActions.banChampion || action.completed) return false
    if (!autoFunctions.banChampion?.enabled || !Array.isArray(autoFunctions.banChampion.championList)) return false

    const banList = autoFunctions.banChampion.championList
    const nextBan = banList.find((c: ChampionInfo) => !picked.has(c.id) && !banned.has(c.id) && !preselected.has(c.id))

    if (!nextBan) {
      console.log('[🤖 AutoChampSelect] 🚫 没有可禁用英雄')
      return false
    }

    const delay = autoFunctions.banChampion.delay || 500
    console.log('[🤖 AutoChampSelect] 🚫 安排自动禁用英雄:', nextBan)

    // 立即标记，防止重复安排
    executedActions.banChampion = true

    setTimeout(async () => {
      try {
        await banChampion(action.id, nextBan.id)
        console.log('[🤖 AutoChampSelect] ✅ 自动禁用成功')
      } catch (err) {
        console.error('[🤖 AutoChampSelect] ❌ 自动禁用失败:', err)
        // 如果失败可以考虑恢复标记
        executedActions.banChampion = false
      }
    }, delay)

    return true
  }
  async function tryAutoPick(
    action: ChampSelectAction,
    autoFunctions: AutoFunctions,
    picked: Set<number>,
    banned: Set<number>,
    preselected: Set<number>,
    executedActions: ExecutedActions
  ) {
    if (action.type !== 'pick' || executedActions.selectChampion || executedActions.lockInProgress || action.completed)
      return false
    if (!autoFunctions.selectChampion?.enabled || !Array.isArray(autoFunctions.selectChampion.championList))
      return false

    const pickList = autoFunctions.selectChampion.championList
    const nextPick = pickList.find(
      (c: ChampionInfo) => !picked.has(c.id) && !banned.has(c.id) && !preselected.has(c.id)
    )

    if (!nextPick) {
      console.log('[🤖 AutoChampSelect] ⭐ 没有可选英雄')
      return false
    }

    const delay = autoFunctions.selectChampion.delay || 500
    executedActions.lockInProgress = true

    console.log('[🤖 AutoChampSelect] ⭐ 安排自动选择英雄:', nextPick)

    setTimeout(async () => {
      try {
        await pickChampion(action.id, nextPick.id, false)
        console.log('[🤖 AutoChampSelect] 👁️ 自动Hover成功')

        setTimeout(async () => {
          try {
            await pickChampion(action.id, nextPick.id, true)
            executedActions.selectChampion = true
            console.log('[🤖 AutoChampSelect] ✅ 自动锁定成功')
          } catch (err) {
            console.error('[🤖 AutoChampSelect] ❌ 自动锁定失败:', err)
          } finally {
            executedActions.lockInProgress = false
          }
        }, 1000)
      } catch (err) {
        console.error('[🤖 AutoChampSelect] ❌ 自动Hover失败:', err)
        executedActions.lockInProgress = false
      }
    }, delay)

    return true
  }

  return {
    getChampSelectSession,
    pickChampion,
    banChampion,
    checkAndExecuteAutoActions,
    getAutoOpggChampionId
  }
}
