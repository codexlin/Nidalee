export const BUILD_POSITIONS = ['TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'] as const

export type BuildPosition = (typeof BUILD_POSITIONS)[number]
export type BuildPresetScope = 'champion-position' | 'champion-all' | 'position-all'
export type BuildPresetSourceKind = 'custom' | 'opgg' | 'client' | 'import'

export interface RuneSelection {
  primaryStyleId: number
  subStyleId: number
  selectedPerkIds: number[]
}

export interface BuildApplicability {
  scope: BuildPresetScope
  championId: number | null
  championName: string | null
  position: BuildPosition | null
}

export interface BuildPresetSource {
  kind: BuildPresetSourceKind
  provider?: 'opgg'
  region?: string
  mode?: string
  tier?: string
  capturedAt?: number
}

/**
 * 用户拥有的构建方案。
 *
 * `components` 是后续装备、召唤师技能和加点方案的唯一扩展点；当前只支持符文，
 * 因而保存时必须包含完整的 `runes`，不创建不可应用的半成品方案。
 */
export interface BuildPreset {
  id: string
  name: string
  applicability: BuildApplicability
  components: {
    runes: RuneSelection
  }
  source: BuildPresetSource
  isDefault: boolean
  createdAt: number
  updatedAt: number
  usageCount: number
}

export interface RecommendedRuneSnapshot {
  championId: number
  championName: string
  position: BuildPosition | null
  selection: RuneSelection
  source: Required<Pick<BuildPresetSource, 'kind' | 'provider' | 'region' | 'mode' | 'tier' | 'capturedAt'>>
}

export function isBuildPosition(value: unknown): value is BuildPosition {
  return typeof value === 'string' && BUILD_POSITIONS.includes(value as BuildPosition)
}

export function normalizeBuildPosition(value: unknown): BuildPosition | null {
  if (typeof value !== 'string') return null
  const normalized = value.trim().toUpperCase()
  const aliases: Record<string, BuildPosition> = {
    TOP: 'TOP',
    JUNGLE: 'JUNGLE',
    MID: 'MID',
    MIDDLE: 'MID',
    ADC: 'ADC',
    BOTTOM: 'ADC',
    SUPPORT: 'SUPPORT',
    UTILITY: 'SUPPORT'
  }
  return aliases[normalized] ?? null
}

export function isBuildPresetSourceKind(value: unknown): value is BuildPresetSourceKind {
  return ['custom', 'opgg', 'client', 'import'].includes(value as BuildPresetSourceKind)
}

export function validateBuildApplicability(applicability: BuildApplicability): string | null {
  const hasChampion = Number.isInteger(applicability.championId) && (applicability.championId ?? 0) > 0
  const hasPosition = isBuildPosition(applicability.position)

  if (applicability.scope === 'champion-position') {
    return hasChampion && hasPosition ? null : '英雄 + 位置方案必须指定英雄和位置'
  }
  if (applicability.scope === 'champion-all') {
    return hasChampion && applicability.position === null ? null : '英雄通用方案只能指定英雄'
  }
  if (applicability.scope === 'position-all') {
    return applicability.championId === null && hasPosition ? null : '位置通用方案只能指定位置'
  }
  return '方案适用范围无效'
}

export function validateRuneSelection(selection: RuneSelection): string | null {
  if (!Number.isInteger(selection.primaryStyleId) || selection.primaryStyleId <= 0) return '主系无效'
  if (!Number.isInteger(selection.subStyleId) || selection.subStyleId <= 0) return '副系无效'
  if (selection.primaryStyleId === selection.subStyleId) return '主系与副系不能相同'
  if (selection.selectedPerkIds.length !== 9) return `符文数量应为 9，当前为 ${selection.selectedPerkIds.length}`
  if (new Set(selection.selectedPerkIds).size !== 9) return '符文中存在重复项'
  if (selection.selectedPerkIds.some((id) => !Number.isInteger(id) || id <= 0)) return '符文 ID 无效'
  return null
}

export function createPresetFromRecommendation(snapshot: RecommendedRuneSnapshot, name?: string): BuildPreset {
  const now = Date.now()
  return {
    id: crypto.randomUUID(),
    name: name?.trim() || `${snapshot.championName}${positionLabel(snapshot.position)}推荐`,
    applicability: {
      scope: snapshot.position ? 'champion-position' : 'champion-all',
      championId: snapshot.championId,
      championName: snapshot.championName,
      position: snapshot.position
    },
    components: { runes: cloneRuneSelection(snapshot.selection) },
    source: { ...snapshot.source },
    isDefault: false,
    createdAt: now,
    updatedAt: now,
    usageCount: 0
  }
}

export function selectMatchingPreset(
  presets: readonly BuildPreset[],
  championId: number,
  position?: string
): BuildPreset | null {
  const normalizedPosition = normalizeBuildPosition(position)
  const ranked = presets
    .map((preset) => ({ preset, score: matchScore(preset, championId, normalizedPosition) }))
    .filter((candidate) => candidate.score >= 0)
    .sort((left, right) => {
      if (left.score !== right.score) return right.score - left.score
      if (left.preset.isDefault !== right.preset.isDefault) return left.preset.isDefault ? -1 : 1
      if (left.preset.updatedAt !== right.preset.updatedAt) return right.preset.updatedAt - left.preset.updatedAt
      return left.preset.id.localeCompare(right.preset.id)
    })

  return ranked[0]?.preset ?? null
}

function matchScore(preset: BuildPreset, championId: number, position: BuildPosition | null): number {
  const target = preset.applicability
  if (target.scope === 'champion-position') {
    return target.championId === championId && target.position === position ? 300 : -1
  }
  if (target.scope === 'champion-all') return target.championId === championId ? 200 : -1
  return position !== null && target.position === position ? 100 : -1
}

export function cloneRuneSelection(selection: RuneSelection): RuneSelection {
  return {
    primaryStyleId: selection.primaryStyleId,
    subStyleId: selection.subStyleId,
    selectedPerkIds: [...selection.selectedPerkIds]
  }
}

export function positionLabel(position: BuildPosition | null): string {
  const labels: Record<BuildPosition, string> = {
    TOP: '上路',
    JUNGLE: '打野',
    MID: '中路',
    ADC: '下路',
    SUPPORT: '辅助'
  }
  return position ? labels[position] : '通用'
}
