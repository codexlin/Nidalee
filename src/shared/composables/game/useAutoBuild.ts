import { computed, shallowRef, watch, type WatchStopHandle } from 'vue'
import { toast } from 'vue-sonner'
import { fetchOpggChampionBuild, fetchOpggTierList } from '@/lib/dataApi'
import { getChampionName } from '@/lib'
import { useGameStore } from '@/shared/stores/features/gameStore'
import { useBuildPresetStore } from '@/shared/stores/features/buildPresetStore'
import { getOpggTierLabel } from '@/shared/utils/opggTier'
import { scenarioLabel, type BuildPosition, type BuildPreset } from '@/shared/models/buildPreset'
import { resolveBuildContext, type ReadyBuildContext } from '@/shared/models/buildContext'
import { selectMainOpggPosition } from '@/shared/models/opggRecommendation'
import { runeSnapshotFromOpgg, useBuildApplication } from './useBuildApplication'

const AUTO_BUILD_DELAY_MS = 1500
const OPGG_REGION = 'kr'

interface AutoApplyRequest {
  generation: number
  key: string
  championId: number
  context: ReadyBuildContext
}

function presetFingerprint(preset: BuildPreset | null): string {
  if (!preset) return ''
  const runes = preset.components.runes
  return `preset:${preset.id}:${preset.updatedAt}:${runes.primaryStyleId}:${runes.subStyleId}:${runes.selectedPerkIds.join('.')}`
}

/** Automatically applies one resolved build after the local player's pick is locked. */
export function useAutoBuild() {
  const presetStore = useBuildPresetStore()
  const gameStore = useGameStore()
  const { applyPreset, applyRecommendation } = useBuildApplication()

  const isApplying = shallowRef(false)
  const lastAppliedKey = shallowRef('')
  const lastError = shallowRef<string | null>(null)
  const lastSuccess = shallowRef<string | null>(null)
  const reconciliationVersion = shallowRef(0)
  let stopWatcher: WatchStopHandle | null = null
  let pendingApplyTimer: ReturnType<typeof setTimeout> | null = null
  let requestGeneration = 0
  let inFlightCount = 0
  const activeApplications = new Map<string, Promise<void>>()
  let lcuWriteQueue: Promise<void> = Promise.resolve()

  const localPlayer = computed(() => {
    const session = gameStore.champSelectSession
    return session?.myTeam.find((player) => player.cellId === session.localPlayerCellId) ?? null
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

  function isCurrentIdentity(request: AutoApplyRequest): boolean {
    if (gameStore.currentPhase !== 'ChampSelect') return false
    if (lockedChampionId.value !== request.championId || localPlayer.value?.championId !== request.championId)
      return false
    return currentRequestKey(request.championId) === request.key
  }

  function isCurrentRequest(request: AutoApplyRequest): boolean {
    return request.generation === requestGeneration && isCurrentIdentity(request)
  }

  function currentRequestKey(championId: number): string {
    const context = resolveBuildContext({
      queueId: gameStore.champSelectSession?.queueId ?? 0,
      isCustomGame: gameStore.champSelectSession?.isCustomGame ?? false,
      position: localPlayer.value?.assignedPosition
    })
    if (context.status !== 'ready' || championId <= 0 || localPlayer.value?.championId !== championId) return ''
    const preset = presetStore.findAutoPreset(championId, context.scenario)
    const source = presetFingerprint(preset) || `opgg:${presetStore.autoBuild.opggTier}`
    return `${championId}:${context.scenario}:${source}`
  }

  async function writeToLcu<T>(request: AutoApplyRequest, write: () => Promise<T>): Promise<T> {
    const result = lcuWriteQueue
      .catch(() => {})
      .then(async () => {
        if (!isCurrentRequest(request)) throw new StaleAutoBuildRequest()
        return write()
      })
    lcuWriteQueue = result.then(
      () => {},
      () => {}
    )
    return result
  }

  async function resolveProviderPosition(request: AutoApplyRequest): Promise<BuildPosition | 'none'> {
    if (request.context.providerPosition !== 'main-position') return request.context.providerPosition

    const tier = presetStore.autoBuild.opggTier
    const response = await fetchOpggTierList({ region: OPGG_REGION, mode: 'ranked', tier })
    if (!isCurrentRequest(request)) throw new StaleAutoBuildRequest()
    if (!response.success || !response.data) throw new Error(response.error || '获取英雄主流位置失败')

    const position = selectMainOpggPosition(response.data, request.championId)
    if (!position) throw new Error('未找到该英雄的主流位置，已保留当前符文')
    return position
  }

  async function applyRecommended(request: AutoApplyRequest): Promise<string> {
    const tier = presetStore.autoBuild.opggTier
    const position = await resolveProviderPosition(request)
    if (!isCurrentRequest(request)) throw new StaleAutoBuildRequest()

    const response = await fetchOpggChampionBuild({
      region: OPGG_REGION,
      mode: request.context.providerMode,
      champion_id: request.championId,
      position,
      tier
    })
    if (!isCurrentRequest(request)) throw new StaleAutoBuildRequest()

    const perk = response.data?.perks?.[0]
    if (!response.success || !perk) throw new Error(response.error || '推荐方案暂不可用')
    const snapshot = runeSnapshotFromOpgg(perk, {
      target: {
        championId: request.championId,
        championName: getChampionName(request.championId),
        scenario: request.context.scenario
      },
      region: OPGG_REGION,
      mode: request.context.providerMode,
      tier
    })

    // The provider calls are the slow part. Revalidate immediately before the irreversible LCU write.
    if (!isCurrentRequest(request)) throw new StaleAutoBuildRequest()
    await writeToLcu(request, () => applyRecommendation(snapshot))
    return request.context.providerMode === 'aram' ? 'OP.GG 极地大乱斗' : `OP.GG ${getOpggTierLabel(tier)}`
  }

  async function autoApplyBuild(request: AutoApplyRequest): Promise<void> {
    const policy = presetStore.autoBuild
    if (!policy.enabled || request.championId <= 0 || !isCurrentRequest(request)) return
    if (lastAppliedKey.value === request.key) return

    inFlightCount += 1
    isApplying.value = true
    lastError.value = null
    lastSuccess.value = null
    const championName = getChampionName(request.championId)

    try {
      let sourceLabel: string
      const preset = presetStore.findAutoPreset(request.championId, request.context.scenario)
      if (preset) {
        if (!isCurrentRequest(request)) throw new StaleAutoBuildRequest()
        await writeToLcu(request, () => applyPreset(preset))
        sourceLabel = `我的方案「${preset.name}」`
      } else {
        sourceLabel = await applyRecommended(request)
      }
      if (!isCurrentRequest(request)) {
        // The same effective selection is current again (for example, enabled was toggled
        // during the IPC). The completed write already satisfies the newer generation.
        if (isCurrentIdentity(request)) {
          lastAppliedKey.value = request.key
        } else {
          // An obsolete write may have completed after a newer context became current.
          // Invalidate the success marker and wake the watcher so the current selection
          // is always serialized after it.
          lastAppliedKey.value = ''
          reconciliationVersion.value += 1
        }
        return
      }

      lastAppliedKey.value = request.key
      lastSuccess.value = `${championName} · ${scenarioLabel(request.context.scenario)} · ${sourceLabel}`
      if (policy.showToast) toast.success('自动构建已应用', { description: lastSuccess.value, duration: 4000 })
    } catch (error) {
      if (error instanceof StaleAutoBuildRequest || !isCurrentRequest(request)) return
      lastError.value = error instanceof Error ? error.message : String(error)
      if (policy.showToast) toast.error('自动构建应用失败', { description: lastError.value, duration: 5000 })
    } finally {
      inFlightCount -= 1
      isApplying.value = inFlightCount > 0
    }
  }

  async function enqueueAutoApply(request: AutoApplyRequest): Promise<void> {
    const active = activeApplications.get(request.key)
    if (active) {
      await active
      if (isCurrentRequest(request) && lastAppliedKey.value !== request.key) {
        await enqueueAutoApply(request)
      }
      return
    }

    const operation = autoApplyBuild(request)
    activeApplications.set(request.key, operation)
    try {
      await operation
    } finally {
      if (activeApplications.get(request.key) === operation) activeApplications.delete(request.key)
    }
  }

  function startAutoBuildWatch(): void {
    if (stopWatcher) return
    stopWatcher = watch(
      [
        () => currentRequestKey(lockedChampionId.value),
        () => gameStore.currentPhase,
        () => presetStore.autoBuild.enabled,
        () => reconciliationVersion.value
      ],
      ([requestKey, phase, enabled]) => {
        requestGeneration += 1
        clearPendingApply()
        if (phase !== 'ChampSelect') {
          lastAppliedKey.value = ''
          return
        }
        if (!enabled || !requestKey) return

        const confirmedChampionId = lockedChampionId.value
        const context = resolveBuildContext({
          queueId: gameStore.champSelectSession?.queueId ?? 0,
          isCustomGame: gameStore.champSelectSession?.isCustomGame ?? false,
          position: localPlayer.value?.assignedPosition
        })
        if (context.status !== 'ready') return
        const request: AutoApplyRequest = {
          generation: requestGeneration,
          key: requestKey,
          championId: confirmedChampionId,
          context
        }

        pendingApplyTimer = setTimeout(() => {
          pendingApplyTimer = null
          void enqueueAutoApply(request)
        }, AUTO_BUILD_DELAY_MS)
      },
      { immediate: true }
    )
  }

  function stopAutoBuildWatch(): void {
    requestGeneration += 1
    clearPendingApply()
    stopWatcher?.()
    stopWatcher = null
  }

  function reset(): void {
    requestGeneration += 1
    clearPendingApply()
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

class StaleAutoBuildRequest extends Error {}
