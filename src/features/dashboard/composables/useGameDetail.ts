import { useDataStore } from '@/shared/stores/core/dataStore'
import { usePersonalMatchAnalysisStore } from '@/shared/stores/features/personalMatchAnalysisStore'
import { createLatestRequestGuard } from '@/shared/utils/latestRequest'
import { invoke } from '@tauri-apps/api/core'
import { computed, ref, toValue, watch, type MaybeRefOrGetter } from 'vue'

type FirstMarker = {
  key: string
  label: string
  title: string
}

const FIRST_DEFS: Array<{
  key: string
  label: string
  pick: (t: TeamInfo | undefined) => boolean | null | undefined
}> = [
  { key: 'blood', label: '一血', pick: (t) => t?.firstBlood },
  { key: 'tower', label: '一塔', pick: (t) => t?.firstTower },
  { key: 'dragon', label: '首条小龙', pick: (t) => t?.firstDragon },
  { key: 'herald', label: '首条先锋', pick: (t) => t?.firstRiftHerald },
  { key: 'baron', label: '首条大龙', pick: (t) => t?.firstBaron },
  { key: 'inhib', label: '首座水晶', pick: (t) => t?.firstInhibitor }
]

function markersForTeam(team: TeamInfo | undefined, side: '蓝队' | '红队'): FirstMarker[] {
  if (!team) return []
  return FIRST_DEFS.filter((def) => !!def.pick(team)).map((def) => ({
    key: def.key,
    label: def.label,
    title: `${side}拿下${def.label}`
  }))
}

/**
 * 对局详情请求与派生数据：负责 get_game_detail + 旧请求丢弃，不负责弹窗 UI。
 */
export function useGameDetail(
  selectedGame: MaybeRefOrGetter<MatchPerformance | null>,
  analysisPuuid: MaybeRefOrGetter<string | null>
) {
  const dataStore = useDataStore()
  const analysisStore = usePersonalMatchAnalysisStore()

  const loading = ref(false)
  const gameDetailData = ref<GameDetail | null>(null)
  const gameDetailRequests = createLatestRequestGuard()

  const gameVersion = computed(() => dataStore.gameVersion)
  const selected = computed(() => toValue(selectedGame))
  const requestedPuuid = computed(() => toValue(analysisPuuid)?.trim() || null)

  const processPuuid = computed(() => requestedPuuid.value)

  const isRankedProcessReview = computed(() => {
    const q = selected.value?.queueId
    return q === 420 || q === 440
  })

  const cachedMatchEvidence = computed(() => {
    const gameId = selected.value?.gameId
    if (gameId === null || gameId === undefined) return null
    if (!requestedPuuid.value || requestedPuuid.value !== analysisStore.lastPuuid) return null
    return analysisStore.getMatchEvidence(gameId)
  })

  const getTeamResult = (teamId: string) => {
    if (!gameDetailData.value) return '未知'
    const team = gameDetailData.value.teams.find((t) => t.teamId && t.teamId.toString() === teamId)
    if (!team || !team.win) return '未知'
    return team.win === 'Win' ? '胜利' : '失败'
  }

  const blueWon = computed(() => {
    const result = getTeamResult('100')
    if (result === '胜利') return true
    if (result === '失败') return false
    return !!selected.value?.win
  })

  const teamObjectives = (teamId: number) => {
    const team = gameDetailData.value?.teams?.find((t) => t.teamId === teamId)
    return {
      dragon: team?.dragonKills ?? 0,
      baron: team?.baronKills ?? 0,
      tower: team?.towerKills ?? 0,
      inhibitor: team?.inhibitorKills ?? 0,
      herald: team?.riftHeraldKills ?? 0,
      horde: team?.hordeKills ?? 0
    }
  }

  const blueObjectives = computed(() => teamObjectives(100))
  const redObjectives = computed(() => teamObjectives(200))

  const blueFirstMarkers = computed(() => {
    const blue = gameDetailData.value?.teams?.find((t) => t.teamId === 100)
    return markersForTeam(blue, '蓝队')
  })

  const redFirstMarkers = computed(() => {
    const red = gameDetailData.value?.teams?.find((t) => t.teamId === 200)
    return markersForTeam(red, '红队')
  })

  const myParticipantId = computed(() => {
    const game = selected.value
    const detail = gameDetailData.value
    if (!game || !detail?.participants?.length) return null

    const exact = detail.participants.find(
      (p) =>
        p.championId === game.championId &&
        p.stats.kills === game.kills &&
        p.stats.deaths === game.deaths &&
        p.stats.assists === game.assists
    )
    if (exact) return exact.participantId

    const byChamp = detail.participants.find((p) => p.championId === game.championId)
    return byChamp?.participantId ?? null
  })

  const getTeamBans = (teamId: string) => {
    const teams = gameDetailData.value?.teams
    if (!teams) return []
    const team = teams.find((t) => t.teamId && t.teamId.toString() === teamId)
    return team?.bans || []
  }

  const getTeamParticipants = (teamId: string) => {
    if (!gameDetailData.value?.participants) return []
    return gameDetailData.value.participants.filter((p) => p.teamId.toString() === teamId)
  }

  watch(
    () => toValue(selectedGame)?.gameId,
    async (gameId, _previousGameId, onCleanup) => {
      const request = gameDetailRequests.begin()
      onCleanup(request.invalidate)

      if (gameId === null || gameId === undefined) {
        gameDetailData.value = null
        loading.value = false
        return
      }

      loading.value = true
      gameDetailData.value = null
      try {
        const result = await invoke<GameDetail>('get_game_detail', { gameId })
        if (!request.isCurrent() || toValue(selectedGame)?.gameId !== gameId) return
        gameDetailData.value = result
      } catch (err: unknown) {
        if (!request.isCurrent() || toValue(selectedGame)?.gameId !== gameId) return
        const message = err instanceof Error ? err.message : String(err)
        console.error('获取游戏详细信息失败:', message)
        gameDetailData.value = null
      } finally {
        if (request.isCurrent() && toValue(selectedGame)?.gameId === gameId) {
          loading.value = false
        }
      }
    },
    { immediate: true }
  )

  return {
    loading,
    gameDetailData,
    gameVersion,
    processPuuid,
    isRankedProcessReview,
    cachedMatchEvidence,
    blueWon,
    blueObjectives,
    redObjectives,
    blueFirstMarkers,
    redFirstMarkers,
    myParticipantId,
    getTeamBans,
    getTeamParticipants
  }
}
