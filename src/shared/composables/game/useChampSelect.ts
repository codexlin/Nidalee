import { invoke } from '@tauri-apps/api/core'

/** LCU Champ Select commands. Automation scheduling lives in useChampSelectAutomation. */
export function useChampSelect() {
  async function pickChampion(actionId: number, championId: number, completed: boolean = false) {
    try {
      console.log('[⭐ ChampSelect] 🎯 选择英雄操作:', { actionId, championId, completed })
      const result = await invoke('pick_champion', { actionId, championId, completed })
      console.log('pick_champion:', result)
      console.log(`[⭐ ChampSelect] ✅ ${completed ? '英雄已锁定' : '英雄已Hover'}`)
    } catch (error) {
      console.error('[⭐ ChampSelect] ❌ 选择英雄失败:', error)
      throw new Error(`选择英雄失败: ${error}`)
    }
  }

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

  return {
    pickChampion,
    banChampion
  }
}
