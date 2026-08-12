import { computed, shallowRef, watch, type WatchStopHandle } from 'vue'
import { toast } from 'vue-sonner'
import { fetchOpggChampionBuild } from '@/lib/dataApi'
import { getChampionName } from '@/lib'
import { useGameStore } from '@/shared/stores/features/gameStore'
import { useBuildPresetStore } from '@/shared/stores/features/buildPresetStore'
import { useMatchAnalysisStore } from '@/features/match-analysis/store'
import { getOpggTierLabel } from '@/shared/utils/opggTier'
import { normalizeBuildPosition, positionLabel } from '@/shared/models/buildPreset'
import { runeSnapshotFromOpgg, useBuildApplication } from './useBuildApplication'

const AUTO_BUILD_DELAY_MS = 1500

/** Automatically applies one resolved build after the local player's pick is locked. */
export function useAutoBuild() {
  const presetStore = useBuildPresetStore()
  const gameStore = useGameStore()
  const matchAnalysisStore = useMatchAnalysisStore()
  const { applyPreset, applyRecommendation } = useBuildApplication()

  const isApplying = shallowRef(false)
  const lastAppliedKey = shallowRef('')
  const lastError = shallowRef<string | null>(null)
  const lastSuccess = shallowRef<string | null>(null)
  let stopWatcher: WatchStopHandle | null = null
  let pendingApplyTimer: ReturnType<typeof setTimeout> | null = null

  const localPlayer = computed(() => {
    const teamData = matchAnalysisStore.myTeamData
    return teamData?.players.find((player) => player.cellId === teamData.localPlayerCellId) ?? null
  })

  const lockedChampionId = computed(() => {
    const session = gameStore.champSelectSession
    if (!session?.actions || session.localPlayerCellId === undefined) return 0
    return (
      session.actions
        .flat()
        .find(
          (action) =>
            action.actorCellId === session.localPlayerCellId &&
            action.type === 'pick' &&
            action.completed &&
            typeof action.championId === 'number' &&
            action.championId > 0
        )?.championId ?? 0
    )
  })

  function clearPendingApply(): void {
    if (pendingApplyTimer === null) return
    clearTimeout(pendingApplyTimer)
    pendingApplyTimer = null
  }

  async function applyRecommended(championId: number, position?: string): Promise<string> {
    const tier = presetStore.autoBuild.opggTier
    const normalizedPosition = normalizeBuildPosition(position)
    const response = await fetchOpggChampionBuild({
      region: 'kr',
      mode: 'ranked',
      champion_id: championId,
      position: normalizedPosition ?? undefined,
      tier
    })
    const perk = response.data?.perks?.[0]
    if (!response.success || !perk) throw new Error(response.error || '推荐方案暂不可用')
    const snapshot = runeSnapshotFromOpgg(perk, {
      championId,
      position: normalizedPosition,
      region: 'kr',
      mode: 'ranked',
      tier
    })
    await applyRecommendation(snapshot)
    return `OP.GG ${getOpggTierLabel(tier)}`
  }

  async function autoApplyBuild(championId: number, position?: string): Promise<void> {
    const policy = presetStore.autoBuild
    if (!policy.enabled || championId <= 0) return

    const normalizedPosition = normalizeBuildPosition(position)
    const applyKey = `${matchAnalysisStore.currentPhase}:${championId}:${normalizedPosition ?? ''}`
    if (lastAppliedKey.value === applyKey || isApplying.value) return

    isApplying.value = true
    lastError.value = null
    lastSuccess.value = null
    const championName = getChampionName(championId)

    try {
      let sourceLabel: string | null = null
      if (policy.strategy !== 'recommended-only') {
        const preset = presetStore.findMatchingPreset(championId, normalizedPosition ?? undefined)
        if (preset) {
          await applyPreset(preset)
          sourceLabel = `我的方案「${preset.name}」`
        } else if (policy.strategy === 'saved-only') {
          throw new Error('没有匹配当前英雄与位置的已保存方案')
        }
      }

      if (!sourceLabel && policy.strategy !== 'saved-only') {
        sourceLabel = await applyRecommended(championId, normalizedPosition ?? undefined)
      }
      if (!sourceLabel) throw new Error('没有可应用的构建方案')

      lastAppliedKey.value = applyKey
      lastSuccess.value = `${championName} · ${positionLabel(normalizedPosition)} · ${sourceLabel}`
      if (policy.showToast) toast.success('自动构建已应用', { description: lastSuccess.value, duration: 4000 })
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : String(error)
      if (policy.showToast) toast.error('自动构建应用失败', { description: lastError.value, duration: 5000 })
    } finally {
      isApplying.value = false
    }
  }

  function startAutoBuildWatch(): void {
    if (stopWatcher) return
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
          lastAppliedKey.value = ''
          return
        }
        if (confirmedChampionId <= 0 || analyzedChampionId !== confirmedChampionId) {
          lastAppliedKey.value = ''
          return
        }

        pendingApplyTimer = setTimeout(() => {
          pendingApplyTimer = null
          void autoApplyBuild(confirmedChampionId, position || undefined)
        }, AUTO_BUILD_DELAY_MS)
      },
      { immediate: true }
    )
  }

  function stopAutoBuildWatch(): void {
    clearPendingApply()
    stopWatcher?.()
    stopWatcher = null
  }

  function reset(): void {
    lastAppliedKey.value = ''
    lastError.value = null
    lastSuccess.value = null
  }

  return {
    isApplying,
    lastError,
    lastSuccess,
    startAutoBuildWatch,
    stopAutoBuildWatch,
    reset
  }
}
