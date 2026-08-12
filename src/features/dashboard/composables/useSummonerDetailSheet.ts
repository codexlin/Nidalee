import { useSearchMatches } from '@/shared/composables/game/useSearchMatches'
import { shallowRef, watch } from 'vue'

/**
 * 对局详情里点击参赛者打开的旁路召唤师 Sheet（与主抽屉并列，避免嵌套）。
 */
export function useSummonerDetailSheet() {
  const isOpen = shallowRef(false)
  const selectedPlayer = shallowRef<{ displayName: string } | null>(null)
  const { fetchSummonerInfo, currentResult, clearSummonerInfo, loading } = useSearchMatches()

  async function openByDisplayName(displayName: string) {
    clearSummonerInfo()
    selectedPlayer.value = { displayName }
    isOpen.value = true
    if (!displayName || displayName === '未知玩家' || displayName === '未知召唤师') return
    await fetchSummonerInfo([displayName])
  }

  async function refresh() {
    const displayName = selectedPlayer.value?.displayName
    if (!displayName || displayName === '未知玩家' || displayName === '未知召唤师') return
    await fetchSummonerInfo([displayName])
  }

  async function openFromParticipant(participant: ParticipantInfo) {
    await openByDisplayName(participant.summonerName)
  }

  watch(isOpen, (open) => {
    if (open) return
    clearSummonerInfo()
    selectedPlayer.value = null
  })

  return {
    isOpen,
    selectedPlayer,
    currentResult,
    loading,
    openByDisplayName,
    refresh,
    openFromParticipant
  }
}
