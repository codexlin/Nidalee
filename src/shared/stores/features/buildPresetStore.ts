import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import { load } from '@tauri-apps/plugin-store'
import { isOpggTier, type OpggTier } from '@/shared/utils/opggTier'
import {
  cloneRuneSelection,
  isBuildPresetSourceKind,
  selectMatchingPreset,
  validateBuildApplicability,
  validateRuneSelection,
  type BuildPreset,
  type RecommendedRuneSnapshot
} from '@/shared/models/buildPreset'
import { createPresetFromRecommendation } from '@/shared/models/buildPreset'

export interface AutoBuildPolicy {
  enabled: boolean
  strategy: 'smart' | 'recommended-only' | 'saved-only'
  opggTier: OpggTier
  showToast: boolean
}

interface PersistedBuildPresetState {
  version: 1
  presets: BuildPreset[]
  autoBuild: AutoBuildPolicy
}

interface BuildPresetExport {
  version: 1
  presets: BuildPreset[]
}

const STORE_FILE = 'build-presets.json'
const STATE_KEY = 'state'

function defaultAutoBuildPolicy(): AutoBuildPolicy {
  return {
    enabled: false,
    strategy: 'smart',
    opggTier: 'diamond_plus',
    showToast: true
  }
}

function normalizePolicy(value: unknown): AutoBuildPolicy {
  if (!value || typeof value !== 'object') return defaultAutoBuildPolicy()
  const candidate = value as Partial<AutoBuildPolicy>
  const strategy = ['smart', 'recommended-only', 'saved-only'].includes(candidate.strategy ?? '')
    ? (candidate.strategy as AutoBuildPolicy['strategy'])
    : 'smart'
  return {
    enabled: candidate.enabled === true,
    strategy,
    opggTier: isOpggTier(candidate.opggTier) ? candidate.opggTier : 'diamond_plus',
    showToast: candidate.showToast !== false
  }
}

function parsePersistedPolicy(value: unknown): AutoBuildPolicy | null {
  if (!value || typeof value !== 'object') return null
  const candidate = value as Partial<AutoBuildPolicy>
  if (typeof candidate.enabled !== 'boolean' || typeof candidate.showToast !== 'boolean') return null
  if (!['smart', 'recommended-only', 'saved-only'].includes(candidate.strategy ?? '')) return null
  if (!isOpggTier(candidate.opggTier)) return null
  return {
    enabled: candidate.enabled,
    strategy: candidate.strategy as AutoBuildPolicy['strategy'],
    opggTier: candidate.opggTier,
    showToast: candidate.showToast
  }
}

function normalizePreset(value: unknown): BuildPreset | null {
  if (!value || typeof value !== 'object') return null
  const preset = value as BuildPreset
  const runes = preset.components?.runes
  if (!preset.id || !preset.name?.trim() || !preset.applicability || !runes) return null
  if (validateBuildApplicability(preset.applicability)) return null
  if (!isBuildPresetSourceKind(preset.source?.kind)) return null
  if (validateRuneSelection(runes)) return null
  if (typeof preset.isDefault !== 'boolean') return null
  if (!Number.isFinite(preset.createdAt) || !Number.isFinite(preset.updatedAt)) return null
  if (!Number.isInteger(preset.usageCount) || preset.usageCount < 0) return null
  return {
    ...preset,
    name: preset.name.trim(),
    applicability: { ...preset.applicability },
    components: { runes: cloneRuneSelection(runes) },
    source: { ...preset.source }
  }
}

export const useBuildPresetStore = defineStore('buildPreset', () => {
  let tauriStore: Awaited<ReturnType<typeof load>> | null = null
  let loadPromise: Promise<void> | null = null
  let mutationQueue: Promise<void> = Promise.resolve()

  const presets = ref<BuildPreset[]>([])
  const autoBuild = ref<AutoBuildPolicy>(defaultAutoBuildPolicy())
  const isLoaded = ref(false)
  const presetCount = computed(() => presets.value.length)

  async function store() {
    tauriStore ??= await load(STORE_FILE)
    return tauriStore
  }

  async function writeState(nextPresets: readonly BuildPreset[], nextAutoBuild: AutoBuildPolicy): Promise<void> {
    const target = await store()
    const state: PersistedBuildPresetState = {
      version: 1,
      presets: [...nextPresets],
      autoBuild: nextAutoBuild
    }
    await target.set(STATE_KEY, state)
    await target.save()
  }

  function loadFromStore(): Promise<void> {
    if (isLoaded.value) return Promise.resolve()
    if (loadPromise) return loadPromise

    loadPromise = (async () => {
      try {
        const target = await store()
        const stored = await target.get<unknown>(STATE_KEY)
        if (stored !== null && stored !== undefined) {
          if (!stored || typeof stored !== 'object') throw new Error('构建方案存储格式无效')
          const state = stored as Partial<PersistedBuildPresetState>
          if (state.version !== 1 || !Array.isArray(state.presets)) throw new Error('不支持的构建方案存储版本')
          const loadedPresets = state.presets.map(normalizePreset).filter((item): item is BuildPreset => item !== null)
          const loadedPolicy = parsePersistedPolicy(state.autoBuild)
          if (loadedPresets.length !== state.presets.length || !loadedPolicy) {
            throw new Error('构建方案存储包含无效或不完整的数据')
          }
          presets.value = loadedPresets
          autoBuild.value = loadedPolicy
        }
        isLoaded.value = true
      } catch (error) {
        console.error('[BuildPresetStore] 加载方案失败:', error)
        throw error
      } finally {
        loadPromise = null
      }
    })()
    return loadPromise
  }

  async function addPreset(preset: BuildPreset): Promise<void> {
    const normalized = normalizePreset(preset)
    if (!normalized) throw new Error('方案数据不完整')
    await mutate((currentPresets, currentPolicy) => {
      if (currentPresets.some((item) => item.id === normalized.id)) throw new Error('方案 ID 已存在')
      const existing = normalized.isDefault ? clearCompetingDefaults(currentPresets, normalized) : [...currentPresets]
      return { presets: [...existing, normalized], policy: currentPolicy }
    })
  }

  async function saveRecommendation(snapshot: RecommendedRuneSnapshot, name?: string): Promise<BuildPreset> {
    const preset = createPresetFromRecommendation(snapshot, name)
    await addPreset(preset)
    return preset
  }

  async function updatePreset(id: string, next: BuildPreset): Promise<void> {
    await mutate((currentPresets, currentPolicy) => {
      const index = currentPresets.findIndex((preset) => preset.id === id)
      if (index < 0) throw new Error('方案不存在')
      const normalized = normalizePreset({
        ...next,
        id,
        createdAt: currentPresets[index].createdAt,
        updatedAt: Date.now()
      })
      if (!normalized) throw new Error('方案数据不完整')
      const updated = normalized.isDefault ? clearCompetingDefaults(currentPresets, normalized) : [...currentPresets]
      updated[index] = normalized
      return { presets: updated, policy: currentPolicy }
    })
  }

  async function deletePreset(id: string): Promise<void> {
    await mutate((currentPresets, currentPolicy) => {
      if (!currentPresets.some((preset) => preset.id === id)) throw new Error('方案不存在')
      return { presets: currentPresets.filter((preset) => preset.id !== id), policy: currentPolicy }
    })
  }

  async function setDefault(id: string): Promise<void> {
    await mutate((currentPresets, currentPolicy) => {
      const selected = currentPresets.find((preset) => preset.id === id)
      if (!selected) throw new Error('方案不存在')
      const now = Date.now()
      return {
        presets: currentPresets.map((preset) =>
          sameApplicability(preset, selected)
            ? { ...preset, isDefault: preset.id === id, updatedAt: preset.id === id ? now : preset.updatedAt }
            : preset
        ),
        policy: currentPolicy
      }
    })
  }

  async function recordUsage(id: string): Promise<void> {
    await mutate((currentPresets, currentPolicy) => ({
      presets: currentPresets.map((preset) =>
        preset.id === id ? { ...preset, usageCount: preset.usageCount + 1 } : preset
      ),
      policy: currentPolicy
    }))
  }

  function findMatchingPreset(championId: number, position?: string): BuildPreset | null {
    return selectMatchingPreset(presets.value, championId, position)
  }

  async function updateAutoBuild(updates: Partial<AutoBuildPolicy>): Promise<void> {
    await mutate((currentPresets, currentPolicy) => ({
      presets: [...currentPresets],
      policy: normalizePolicy({ ...currentPolicy, ...updates })
    }))
  }

  function exportPresets(): string {
    const exported: BuildPresetExport = { version: 1, presets: presets.value }
    return JSON.stringify(exported, null, 2)
  }

  async function importPresets(json: string): Promise<number> {
    const parsed = JSON.parse(json) as Partial<BuildPresetExport>
    if (parsed.version !== 1 || !Array.isArray(parsed.presets)) throw new Error('不支持的方案文件')
    const imported = parsed.presets.map(normalizePreset).filter((item): item is BuildPreset => item !== null)
    if (imported.length !== parsed.presets.length) throw new Error('方案文件包含无效或不完整的数据')
    const now = Date.now()
    const importedCopies = imported.map((preset) => ({
      ...preset,
      id: crypto.randomUUID(),
      source: { ...preset.source, kind: 'import' as const },
      createdAt: now,
      updatedAt: now,
      usageCount: 0,
      isDefault: false
    }))
    await mutate((currentPresets, currentPolicy) => ({
      presets: [...currentPresets, ...importedCopies],
      policy: currentPolicy
    }))
    return imported.length
  }

  function mutate(
    transform: (
      currentPresets: readonly BuildPreset[],
      currentPolicy: AutoBuildPolicy
    ) => { presets: BuildPreset[]; policy: AutoBuildPolicy }
  ): Promise<void> {
    const operation = mutationQueue.then(async () => {
      await loadFromStore()
      const next = transform(presets.value, autoBuild.value)
      await writeState(next.presets, next.policy)
      presets.value = next.presets
      autoBuild.value = next.policy
    })
    mutationQueue = operation.catch(() => {})
    return operation
  }

  return {
    presets,
    autoBuild,
    isLoaded,
    presetCount,
    loadFromStore,
    addPreset,
    saveRecommendation,
    updatePreset,
    deletePreset,
    setDefault,
    recordUsage,
    findMatchingPreset,
    updateAutoBuild,
    exportPresets,
    importPresets
  }
})

function sameApplicability(left: BuildPreset, right: BuildPreset): boolean {
  return (
    left.applicability.scope === right.applicability.scope &&
    left.applicability.championId === right.applicability.championId &&
    left.applicability.position === right.applicability.position
  )
}

function clearCompetingDefaults(presets: readonly BuildPreset[], selected: BuildPreset): BuildPreset[] {
  return presets.map((preset) =>
    preset.id !== selected.id && preset.isDefault && sameApplicability(preset, selected)
      ? { ...preset, isDefault: false }
      : preset
  )
}
