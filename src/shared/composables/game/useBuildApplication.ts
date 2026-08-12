import { invoke } from '@tauri-apps/api/core'
import { getChampionName } from '@/lib'
import { useBuildPresetStore } from '@/shared/stores/features/buildPresetStore'
import {
  cloneRuneSelection,
  normalizeBuildPosition,
  validateRuneSelection,
  type BuildPreset,
  type RecommendedRuneSnapshot,
  type RuneSelection
} from '@/shared/models/buildPreset'

export interface RecommendationContext {
  championId: number
  position?: string | null
  region: string
  mode: string
  tier: string
}

export function runeSnapshotFromOpgg(perk: OpggPerk, context: RecommendationContext): RecommendedRuneSnapshot {
  const selection: RuneSelection = {
    primaryStyleId: perk.primaryId,
    subStyleId: perk.secondaryId,
    selectedPerkIds: [...perk.perks]
  }
  const validationError = validateRuneSelection(selection)
  if (validationError) throw new Error(`推荐符文不完整：${validationError}`)

  return {
    championId: context.championId,
    championName: getChampionName(context.championId),
    position: normalizeBuildPosition(context.position),
    selection,
    source: {
      kind: 'opgg',
      provider: 'opgg',
      region: context.region,
      mode: context.mode,
      tier: context.tier,
      capturedAt: Date.now()
    }
  }
}

export function useBuildApplication() {
  const presetStore = useBuildPresetStore()
  const applying = shallowRef(false)
  const lastError = shallowRef<string | null>(null)
  const lastAppliedLabel = shallowRef<string | null>(null)

  async function applyRuneSelection(pageLabel: string, selection: RuneSelection): Promise<string> {
    const validationError = validateRuneSelection(selection)
    if (validationError) throw new Error(validationError)

    applying.value = true
    lastError.value = null
    try {
      const result = await invoke<string>('apply_rune_selection', {
        pageLabel: pageLabel.trim(),
        selection: cloneRuneSelection(selection)
      })
      lastAppliedLabel.value = pageLabel.trim()
      return result
    } catch (error) {
      lastError.value = error instanceof Error ? error.message : String(error)
      throw error
    } finally {
      applying.value = false
    }
  }

  async function applyPreset(preset: BuildPreset): Promise<string> {
    const result = await applyRuneSelection(preset.name, preset.components.runes)
    try {
      await presetStore.recordUsage(preset.id)
    } catch (error) {
      console.warn('[BuildApplication] 符文已应用，但使用次数保存失败:', error)
    }
    return result
  }

  async function applyRecommendation(snapshot: RecommendedRuneSnapshot): Promise<string> {
    return applyRuneSelection(snapshot.championName, snapshot.selection)
  }

  return {
    applying: readonly(applying),
    lastError: readonly(lastError),
    lastAppliedLabel: readonly(lastAppliedLabel),
    applyRuneSelection,
    applyPreset,
    applyRecommendation
  }
}
