import { computed, ref, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { load } from '@tauri-apps/plugin-store'
import { isOpggTier, type OpggTier } from '@/shared/utils/opggTier'
import {
  cloneRuneSelection,
  createPresetFromRecommendation,
  isBuildPresetSourceKind,
  isBuildScenario,
  sameRuneSelection,
  sameBuildTarget,
  selectAutoPreset,
  validateBuildTarget,
  validateRuneSelection,
  type BuildPreset,
  type BuildScenario,
  type RecommendedRuneSnapshot
} from '@/shared/models/buildPreset'

export interface AutoBuildPolicy {
  enabled: boolean
  opggTier: OpggTier
  showToast: boolean
}

interface PersistedBuildPresetState {
  version: 2
  presets: BuildPreset[]
  autoBuild: AutoBuildPolicy
}

interface BuildPresetExport {
  version: 2
  presets: BuildPreset[]
}

const STORE_FILE = 'build-presets-v2.json'
const STATE_KEY = 'state'

function defaultAutoBuildPolicy(): AutoBuildPolicy {
  return {
    enabled: false,
    opggTier: 'diamond_plus',
    showToast: true
  }
}

function normalizePolicy(value: unknown): AutoBuildPolicy {
  if (!value || typeof value !== 'object') return defaultAutoBuildPolicy()
  const candidate = value as Partial<AutoBuildPolicy>
  return {
    enabled: candidate.enabled === true,
    opggTier: isOpggTier(candidate.opggTier) ? candidate.opggTier : 'diamond_plus',
    showToast: candidate.showToast !== false
  }
}

function parsePersistedPolicy(value: unknown): AutoBuildPolicy | null {
  if (!value || typeof value !== 'object') return null
  const candidate = value as Partial<AutoBuildPolicy>
  if (typeof candidate.enabled !== 'boolean' || typeof candidate.showToast !== 'boolean') return null
  if (!isOpggTier(candidate.opggTier)) return null
  return {
    enabled: candidate.enabled,
    opggTier: candidate.opggTier,
    showToast: candidate.showToast
  }
}

function normalizePreset(value: unknown): BuildPreset | null {
  if (!value || typeof value !== 'object') return null
  const preset = value as BuildPreset
  const runes = preset.components?.runes
  if (!preset.id || !preset.name?.trim() || !preset.target || !runes) return null
  if (validateBuildTarget(preset.target)) return null
  if (!isBuildPresetSourceKind(preset.source?.kind)) return null
  if (validateRuneSelection(runes)) return null
  if (typeof preset.autoUse !== 'boolean') return null
  if (!Number.isFinite(preset.createdAt) || !Number.isFinite(preset.updatedAt)) return null
  if (!Number.isInteger(preset.usageCount) || preset.usageCount < 0) return null
  return {
    ...preset,
    name: preset.name.trim(),
    target: { ...preset.target, championName: preset.target.championName.trim() },
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
  const isLoaded = shallowRef(false)
  const presetCount = computed(() => presets.value.length)

  async function store() {
    tauriStore ??= await load(STORE_FILE)
    return tauriStore
  }

  async function writeState(nextPresets: readonly BuildPreset[], nextAutoBuild: AutoBuildPolicy): Promise<void> {
    const target = await store()
    const previousState = await target.get<unknown>(STATE_KEY)
    const state: PersistedBuildPresetState = {
      version: 2,
      presets: [...nextPresets],
      autoBuild: nextAutoBuild
    }
    try {
      await target.set(STATE_KEY, state)
      await target.save()
    } catch (error) {
      if (previousState === null || previousState === undefined) {
        await target.delete(STATE_KEY)
      } else {
        await target.set(STATE_KEY, previousState)
      }
      throw error
    }
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
          if (state.version !== 2 || !Array.isArray(state.presets)) throw new Error('不支持的构建方案存储版本')
          const loadedPresets = state.presets.map(normalizePreset).filter((item): item is BuildPreset => item !== null)
          const loadedPolicy = parsePersistedPolicy(state.autoBuild)
          if (loadedPresets.length !== state.presets.length || !loadedPolicy) {
            throw new Error('构建方案存储包含无效或不完整的数据')
          }
          assertUniqueAutoUse(loadedPresets)
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
      if (findEquivalentPreset(currentPresets, normalized)) throw new Error('相同英雄、场景和符文的方案已存在')
      return {
        presets: enforceUniqueAutoUse([...currentPresets, normalized], normalized),
        policy: currentPolicy
      }
    })
  }

  async function saveRecommendation(snapshot: RecommendedRuneSnapshot, name?: string): Promise<BuildPreset> {
    const preset = createPresetFromRecommendation(snapshot, name)
    let savedPreset = preset
    await mutate((currentPresets, currentPolicy) => {
      const existing = findEquivalentPreset(currentPresets, preset)
      if (existing) {
        savedPreset = existing
        return { presets: [...currentPresets], policy: currentPolicy }
      }
      return { presets: [...currentPresets, preset], policy: currentPolicy }
    })
    return savedPreset
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
      if (findEquivalentPreset(currentPresets, normalized, id)) {
        throw new Error('相同英雄、场景和符文的方案已存在')
      }
      const updated = currentPresets.map((preset, presetIndex) => (presetIndex === index ? normalized : preset))
      return { presets: enforceUniqueAutoUse(updated, normalized), policy: currentPolicy }
    })
  }

  async function deletePreset(id: string): Promise<void> {
    await mutate((currentPresets, currentPolicy) => {
      if (!currentPresets.some((preset) => preset.id === id)) throw new Error('方案不存在')
      return { presets: currentPresets.filter((preset) => preset.id !== id), policy: currentPolicy }
    })
  }

  async function setAutoUse(id: string, enabled: boolean): Promise<void> {
    await mutate((currentPresets, currentPolicy) => {
      const selected = currentPresets.find((preset) => preset.id === id)
      if (!selected) throw new Error('方案不存在')
      const updatedAt = Date.now()
      const selectedPreset = { ...selected, autoUse: enabled, updatedAt }
      const updated = currentPresets.map((preset) => (preset.id === id ? selectedPreset : preset))
      return {
        presets: enabled ? enforceUniqueAutoUse(updated, selectedPreset) : updated,
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

  function findAutoPreset(championId: number, scenario: BuildScenario): BuildPreset | null {
    return selectAutoPreset(presets.value, championId, scenario)
  }

  async function updateAutoBuild(updates: Partial<AutoBuildPolicy>): Promise<void> {
    await mutate((currentPresets, currentPolicy) => ({
      presets: [...currentPresets],
      policy: normalizePolicy({ ...currentPolicy, ...updates })
    }))
  }

  function exportPresets(): string {
    const exported: BuildPresetExport = { version: 2, presets: presets.value }
    return JSON.stringify(exported, null, 2)
  }

  async function importPresets(json: string): Promise<number> {
    const parsed = JSON.parse(json) as Partial<BuildPresetExport>
    if (parsed.version !== 2 || !Array.isArray(parsed.presets)) throw new Error('不支持的方案文件')
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
      autoUse: false
    }))
    let importedCount = 0
    await mutate((currentPresets, currentPolicy) => {
      const nextPresets = [...currentPresets]
      for (const preset of importedCopies) {
        if (findEquivalentPreset(nextPresets, preset)) continue
        nextPresets.push(preset)
        importedCount += 1
      }
      return { presets: nextPresets, policy: currentPolicy }
    })
    return importedCount
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
      assertUniqueAutoUse(next.presets)
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
    setAutoUse,
    recordUsage,
    findAutoPreset,
    updateAutoBuild,
    exportPresets,
    importPresets
  }
})

function enforceUniqueAutoUse(presets: readonly BuildPreset[], selected: BuildPreset): BuildPreset[] {
  if (!selected.autoUse) return [...presets]
  return presets.map((preset) =>
    preset.id !== selected.id && preset.autoUse && sameBuildTarget(preset.target, selected.target)
      ? { ...preset, autoUse: false }
      : preset
  )
}

function findEquivalentPreset(
  presets: readonly BuildPreset[],
  candidate: BuildPreset,
  excludedId?: string
): BuildPreset | null {
  return (
    presets.find(
      (preset) =>
        preset.id !== excludedId &&
        sameBuildTarget(preset.target, candidate.target) &&
        sameRuneSelection(preset.components.runes, candidate.components.runes)
    ) ?? null
  )
}

function assertUniqueAutoUse(presets: readonly BuildPreset[]): void {
  const targets = new Set<string>()
  for (const preset of presets) {
    if (!preset.autoUse) continue
    if (!isBuildScenario(preset.target.scenario)) throw new Error('方案场景无效')
    const key = `${preset.target.championId}:${preset.target.scenario}`
    if (targets.has(key)) throw new Error('同一英雄和场景只能有一套自动方案')
    targets.add(key)
  }
}
