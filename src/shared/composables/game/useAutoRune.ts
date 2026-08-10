import { computed, shallowRef, watch, type WatchStopHandle } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useUserRuneStore, type RuneConfig } from '@/shared/stores/features/userRuneStore'
import { useGameStore } from '@/shared/stores/features/gameStore'
import { useMatchAnalysisStore } from '@/features/match-analysis/store'
import { getChampionName } from '@/lib'

const AUTO_RUNE_DELAY_MS = 1500

/**
 * 自动符文应用 Composable
 *
 * 功能：
 * 1. 监听英雄选择变化
 * 2. 根据策略自动应用符文
 * 3. 优先级：用户自定义 > OP.GG 推荐
 */
export function useAutoRune() {
  const userRuneStore = useUserRuneStore()
  const gameStore = useGameStore()
  const matchAnalysisStore = useMatchAnalysisStore()

  // 状态
  const isApplying = shallowRef(false)
  const lastAppliedChampionId = shallowRef<number>(0)
  const lastAppliedPosition = shallowRef<string>('')
  const lastError = shallowRef<string | null>(null)
  const lastSuccess = shallowRef<string | null>(null)
  let stopWatcher: WatchStopHandle | null = null
  let pendingApplyTimer: ReturnType<typeof setTimeout> | null = null

  const localPlayer = computed(() => {
    const teamData = matchAnalysisStore.myTeamData
    if (!teamData) return null
    return teamData.players.find((player) => player.cellId === teamData.localPlayerCellId) ?? null
  })

  const lockedChampionId = computed(() => {
    const session = gameStore.champSelectSession
    if (!session?.actions || session.localPlayerCellId === undefined) return 0

    const completedPick = session.actions
      .flat()
      .find(
        (action) =>
          action.actorCellId === session.localPlayerCellId &&
          action.type === 'pick' &&
          action.completed &&
          typeof action.championId === 'number' &&
          action.championId > 0
      )

    return completedPick?.championId ?? 0
  })

  const clearPendingApply = () => {
    if (pendingApplyTimer !== null) {
      clearTimeout(pendingApplyTimer)
      pendingApplyTimer = null
    }
  }

  /**
   * 应用用户自定义符文
   */
  const applyUserConfig = async (config: RuneConfig, _championId: number, championName: string): Promise<void> => {
    console.log('[AutoRune] 应用用户自定义符文:', config)

    // 调用后端应用自定义符文
    await invoke<string>('apply_custom_runes', {
      championName: championName,
      primaryStyleId: config.primaryStyleId,
      subStyleId: config.subStyleId,
      selectedPerkIds: config.selectedPerkIds
    })

    console.log('[AutoRune] 用户自定义符文应用成功')
  }

  /**
   * 应用 OP.GG 推荐符文
   */
  const applyOpggRunes = async (championId: number, championName: string, position?: string): Promise<void> => {
    console.log('[AutoRune] 应用 OP.GG 符文:', { championId, championName, position })

    const autoApply = userRuneStore.autoApply

    // 调用后端应用符文
    await invoke<string>('apply_opgg_runes', {
      region: 'kr', // 默认韩服
      mode: 'ranked', // 默认排位
      championId: championId,
      championName: championName,
      position: position || null,
      tier: autoApply.opggTier,
      buildIndex: 0 // 应用第一个（最佳）符文配置
    })

    lastSuccess.value = `✨ 自动应用符文成功！\n🎯 英雄：${championName}\n📍 位置：${position || '通用'}\n🔮 来源：OP.GG (${autoApply.opggTier})`
  }

  /**
   * 自动应用符文的核心逻辑
   */
  const autoApplyRune = async (championId: number, position?: string) => {
    // 1. 检查是否启用自动应用
    if (!userRuneStore.autoApply.enabled) {
      console.log('[AutoRune] 自动应用未启用，跳过')
      return
    }

    // 2. 防止重复应用
    if (lastAppliedChampionId.value === championId && lastAppliedPosition.value === (position || '')) {
      console.log('[AutoRune] 已经为此英雄+位置应用过符文，跳过')
      return
    }

    // 3. 验证 championId
    if (!championId || championId <= 0) {
      console.log('[AutoRune] 无效的英雄 ID，跳过')
      return
    }

    isApplying.value = true
    lastError.value = null
    lastSuccess.value = null

    try {
      const championName = getChampionName(championId)
      const strategy = userRuneStore.autoApply.strategy

      console.log('[AutoRune] 开始自动应用符文:', {
        championId,
        championName,
        position,
        strategy
      })

      let applied = false

      // 策略 1: 优先用户自定义（如果策略允许）
      if (strategy === 'auto' || strategy === 'custom') {
        const userConfig = userRuneStore.findBestMatch(championId, position)

        if (userConfig) {
          console.log('[AutoRune] 找到用户自定义配置:', userConfig)

          try {
            await applyUserConfig(userConfig, championId, championName)

            // 增加使用次数
            await userRuneStore.incrementUsageCount(userConfig.id)

            lastSuccess.value = `✨ 自动应用符文成功！\n🎯 英雄：${championName}\n📍 位置：${position || '通用'}\n🔮 来源：自定义 (${userConfig.name})`
            applied = true
          } catch (err) {
            console.warn('[AutoRune] 应用用户自定义符文失败，将回退到 OP.GG:', err)
            // 继续尝试 OP.GG
          }
        }
      }

      // 策略 2: 回退到 OP.GG（如果策略允许且用户配置未应用）
      if (!applied && (strategy === 'auto' || strategy === 'opgg')) {
        console.log('[AutoRune] 使用 OP.GG 推荐符文')
        await applyOpggRunes(championId, championName, position)
        applied = true
      }

      if (!applied) {
        throw new Error('未找到可用的符文配置')
      }

      // 记录已应用
      lastAppliedChampionId.value = championId
      lastAppliedPosition.value = position || ''

      // 显示 Toast（如果启用）
      if (userRuneStore.autoApply.showToast && lastSuccess.value) {
        // TODO: 集成 Toast 通知组件
        console.log('[AutoRune] 应用成功:', lastSuccess.value)
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : '应用符文失败'
      lastError.value = message
      console.error('[AutoRune] 自动应用符文失败:', err)

      // 显示错误 Toast
      if (userRuneStore.autoApply.showToast) {
        // TODO: 集成 Toast 通知组件
        console.error('[AutoRune] 应用失败:', lastError.value)
      }
    } finally {
      isApplying.value = false
    }
  }

  /**
   * 启动自动符文监听
   *
   * 监听逻辑：
   * 1. 监听 matchAnalysisStore.myTeamData 的变化
   * 2. 从中提取当前玩家的 championId 和 position
   * 3. 触发自动应用
   */
  const startAutoRuneWatch = () => {
    if (stopWatcher) return

    console.log('[AutoRune] 启动自动符文监听')

    stopWatcher = watch(
      [
        () => lockedChampionId.value,
        () => localPlayer.value?.championId ?? 0,
        () => localPlayer.value?.position ?? '',
        () => matchAnalysisStore.currentPhase
      ],
      ([confirmedChampionId, analyzedChampionId, position, phase]) => {
        clearPendingApply()

        if (phase !== 'ChampSelect') {
          if (phase === 'None') {
            lastAppliedChampionId.value = 0
            lastAppliedPosition.value = ''
          }
          return
        }

        if (
          typeof confirmedChampionId !== 'number' ||
          confirmedChampionId <= 0 ||
          analyzedChampionId !== confirmedChampionId
        ) {
          lastAppliedChampionId.value = 0
          lastAppliedPosition.value = ''
          return
        }

        console.log('[AutoRune] 检测到英雄选择:', {
          championId: confirmedChampionId,
          position,
          playerName: localPlayer.value?.displayName
        })

        pendingApplyTimer = setTimeout(() => {
          pendingApplyTimer = null
          void autoApplyRune(confirmedChampionId, position || undefined)
        }, AUTO_RUNE_DELAY_MS)
      },
      { immediate: true }
    )
  }

  const stopAutoRuneWatch = () => {
    clearPendingApply()
    stopWatcher?.()
    stopWatcher = null
  }

  /**
   * 手动应用符文（用于测试或手动触发）
   */
  const manualApplyRune = async (championId: number, position?: string) => {
    // 重置状态以允许重新应用
    lastAppliedChampionId.value = 0
    lastAppliedPosition.value = ''

    await autoApplyRune(championId, position)
  }

  /**
   * 重置状态
   */
  const reset = () => {
    lastAppliedChampionId.value = 0
    lastAppliedPosition.value = ''
    lastError.value = null
    lastSuccess.value = null
  }

  return {
    // 状态
    isApplying,
    lastError,
    lastSuccess,

    // 方法
    startAutoRuneWatch,
    stopAutoRuneWatch,
    manualApplyRune,
    reset
  }
}
