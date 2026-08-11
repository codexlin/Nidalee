import { invoke } from '@tauri-apps/api/core'
import { getChampionName } from '@/lib'
import type { OpggConfig } from './useOpggData'
import { buildRequestPosition } from '../types/modes'

export function useOpggRunes() {
  const applying = ref(false)
  const applyError = ref<string | null>(null)
  const applySuccess = ref<string | null>(null)

  // 应用最佳符文
  const applyBestRunes = async (championId: number, config: OpggConfig) => {
    if (!championId) {
      applyError.value = '请先选择英雄'
      return
    }

    await applySpecificRunes(0, championId, config) // 应用第一个（最佳）符文配置
  }

  // 应用特定符文
  const applySpecificRunes = async (runeIndex: number, championId: number, config: OpggConfig) => {
    if (!championId) {
      applyError.value = '请先选择英雄'
      return
    }

    applying.value = true
    applyError.value = null
    applySuccess.value = null

    try {
      const championName = getChampionName(championId)

      // 调用Tauri命令应用符文
      if (config.mode === 'arena') {
        throw new Error('竞技场模式无符文数据，无法套用')
      }

      await invoke<string>('apply_opgg_runes', {
        region: config.region,
        mode: config.mode,
        championId: championId,
        championName: championName,
        position: buildRequestPosition(config.mode, config.position),
        tier: config.tier,
        buildIndex: runeIndex
      })

      applySuccess.value = `✨ 符文配置应用成功！\n🎯 来源：Nidalee + ${championName}\n🔮 已创建专属符文页面`
    } catch (err) {
      applyError.value = err instanceof Error ? err.message : '应用符文失败'
    } finally {
      applying.value = false
    }
  }

  // 清空应用状态
  const clearApplyState = () => {
    applyError.value = null
    applySuccess.value = null
  }

  return {
    // 状态
    applying: readonly(applying),
    applyError: readonly(applyError),
    applySuccess: readonly(applySuccess),

    // 方法
    applyBestRunes,
    applySpecificRunes,
    clearApplyState
  }
}
