import { useSearchMatches } from '@/shared/composables/game/useSearchMatches'
import { ref } from 'vue'

/**
 * 对局详情里点击参赛者打开的旁路召唤师 Sheet（与主抽屉并列，避免嵌套）。
 */
export function useSummonerDetailSheet() {
  const isOpen = ref(false)
  const selectedPlayer = ref<{ displayName: string } | null>(null)
  const { fetchSummonerInfo, currentResult, clearSummonerInfo, loading } = useSearchMatches()

  async function openFromParticipant(participant: ParticipantInfo) {
    clearSummonerInfo()
    selectedPlayer.value = {
      displayName: participant.summonerName
    }
    isOpen.value = true
    if (
      participant.summonerName &&
      participant.summonerName !== '未知玩家' &&
      participant.summonerName !== '未知召唤师'
    ) {
      await fetchSummonerInfo([participant.summonerName])
    }
  }

  return {
    isOpen,
    selectedPlayer,
    currentResult,
    loading,
    openFromParticipant
  }
}
