import { invoke } from '@tauri-apps/api/core'
import type { MatchModeKey } from '@/common/queueCatalog'
import { useMatchAnalysis } from '@/shared/composables/game/useMatchAnalysis'

/**
 * 统一更新召唤师信息和战绩信息
 *
 * 战绩路径已收敛到 `analyze_matches`（单次查询）。
 */
export function useSummonerAndMatchUpdater() {
  const dataStore = useDataStore()
  const activityStore = useActivityStore()
  const { analyzeMatches } = useMatchAnalysis()

  const updateSummonerInfo = async () => {
    try {
      dataStore.startLoadingSummoner()
      const summonerInfo = await invoke<SummonerInfo>('get_current_summoner')
      if (summonerInfo) {
        dataStore.setSummonerInfo(summonerInfo)
        activityStore.addActivity('info', '召唤师信息已更新', 'data')
      }
    } catch (error) {
      console.error('[Updater] 获取召唤师信息失败:', error)
      dataStore.clearSummonerInfo()
    }
  }

  /**
   * 更新战绩信息（单次 analyze_matches）
   * 无参时与仪表盘共用 settingsStore.lastMatchMode / lastMatchCount
   */
  const updateMatchHistory = async (mode?: MatchModeKey, countOverride?: number) => {
    await analyzeMatches(mode, countOverride)
  }

  const updateSummonerAndMatches = async () => {
    await Promise.all([updateSummonerInfo(), updateMatchHistory()])
  }

  return {
    updateSummonerAndMatches,
    updateSummonerInfo,
    updateMatchHistory
  }
}
