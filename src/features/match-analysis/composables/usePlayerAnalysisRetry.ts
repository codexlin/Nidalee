import { invoke } from '@tauri-apps/api/core'
import { shallowRef } from 'vue'
import { toast } from 'vue-sonner'
import type { UIPlayerData } from '@/types/match-analysis'

export function usePlayerAnalysisRetry() {
  const retryingPuuids = shallowRef<ReadonlySet<string>>(new Set())

  function isRetrying(player: UIPlayerData): boolean {
    return !!player.puuid && retryingPuuids.value.has(player.puuid)
  }

  async function retryPlayer(player: UIPlayerData): Promise<void> {
    const puuid = player.puuid?.trim()
    if (!puuid || retryingPuuids.value.has(puuid)) return

    retryingPuuids.value = new Set(retryingPuuids.value).add(puuid)
    try {
      const status = await invoke<PlayerAnalysisStatus>('retry_player_analysis', { puuid })
      if (status === 'ready') {
        toast.success(`已重新分析 ${player.displayName}`)
      } else if (status === 'insufficientData') {
        toast.info('分析完成，但近期有效样本不足')
      }
    } catch (error) {
      toast.error('重新分析失败', { description: String(error) })
    } finally {
      const next = new Set(retryingPuuids.value)
      next.delete(puuid)
      retryingPuuids.value = next
    }
  }

  return { isRetrying, retryPlayer }
}
