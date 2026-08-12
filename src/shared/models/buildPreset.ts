export const BUILD_POSITIONS = ['TOP', 'JUNGLE', 'MID', 'ADC', 'SUPPORT'] as const

export const BUILD_SCENARIOS = [
  'ranked-top',
  'ranked-jungle',
  'ranked-mid',
  'ranked-adc',
  'ranked-support',
  'normal-sr',
  'aram'
] as const

export type BuildPosition = (typeof BUILD_POSITIONS)[number]
export type BuildScenario = (typeof BUILD_SCENARIOS)[number]
export type BuildPresetSourceKind = 'custom' | 'opgg' | 'client' | 'import'

export interface RuneSelection {
  primaryStyleId: number
  subStyleId: number
  selectedPerkIds: number[]
}

export interface BuildTarget {
  championId: number
  championName: string
  scenario: BuildScenario
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
 * `target` 使用“英雄 + 场景”的精确键。只有 `autoUse` 方案参与自动匹配；
 * 同一个 target 最多有一套自动方案，其余方案只供手动使用。
 *
 * `components` 是后续装备、召唤师技能和加点方案的唯一扩展点；当前只支持符文，
 * 因而保存时必须包含完整的 `runes`，不创建不可应用的半成品方案。
 */
export interface BuildPreset {
  id: string
  name: string
  target: BuildTarget
  components: {
    runes: RuneSelection
  }
  source: BuildPresetSource
  autoUse: boolean
  createdAt: number
  updatedAt: number
  usageCount: number
}

export interface RecommendedRuneSnapshot {
  target: BuildTarget
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

export function isBuildScenario(value: unknown): value is BuildScenario {
  return typeof value === 'string' && BUILD_SCENARIOS.includes(value as BuildScenario)
}

export function rankedScenarioFromPosition(position: unknown): BuildScenario | null {
  const normalized = normalizeBuildPosition(position)
  if (!normalized) return null
  const scenarios: Record<BuildPosition, BuildScenario> = {
    TOP: 'ranked-top',
    JUNGLE: 'ranked-jungle',
    MID: 'ranked-mid',
    ADC: 'ranked-adc',
    SUPPORT: 'ranked-support'
  }
  return scenarios[normalized]
}

export function rankedPositionFromScenario(scenario: BuildScenario): BuildPosition | null {
  const positions: Partial<Record<BuildScenario, BuildPosition>> = {
    'ranked-top': 'TOP',
    'ranked-jungle': 'JUNGLE',
    'ranked-mid': 'MID',
    'ranked-adc': 'ADC',
    'ranked-support': 'SUPPORT'
  }
  return positions[scenario] ?? null
}

export function isBuildPresetSourceKind(value: unknown): value is BuildPresetSourceKind {
  return ['custom', 'opgg', 'client', 'import'].includes(value as BuildPresetSourceKind)
}

export function validateBuildTarget(target: BuildTarget): string | null {
  if (!Number.isInteger(target.championId) || target.championId <= 0) return '方案必须指定有效英雄'
  if (!target.championName.trim()) return '方案必须包含英雄名称'
  if (!isBuildScenario(target.scenario)) return '方案场景无效'
  return null
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

export function buildTargetKey(target: Pick<BuildTarget, 'championId' | 'scenario'>): string {
  return `${target.championId}:${target.scenario}`
}

export function sameBuildTarget(left: BuildTarget, right: BuildTarget): boolean {
  return left.championId === right.championId && left.scenario === right.scenario
}

export function createPresetFromRecommendation(snapshot: RecommendedRuneSnapshot, name?: string): BuildPreset {
  const now = Date.now()
  return {
    id: crypto.randomUUID(),
    name: name?.trim() || `${snapshot.target.championName}${scenarioLabel(snapshot.target.scenario)}推荐`,
    target: { ...snapshot.target },
    components: { runes: cloneRuneSelection(snapshot.selection) },
    source: { ...snapshot.source },
    autoUse: false,
    createdAt: now,
    updatedAt: now,
    usageCount: 0
  }
}

export function selectAutoPreset(
  presets: readonly BuildPreset[],
  championId: number,
  scenario: BuildScenario
): BuildPreset | null {
  return (
    presets.find(
      (preset) => preset.autoUse && preset.target.championId === championId && preset.target.scenario === scenario
    ) ?? null
  )
}

export function cloneRuneSelection(selection: RuneSelection): RuneSelection {
  return {
    primaryStyleId: selection.primaryStyleId,
    subStyleId: selection.subStyleId,
    selectedPerkIds: [...selection.selectedPerkIds]
  }
}

export function sameRuneSelection(left: RuneSelection, right: RuneSelection): boolean {
  return (
    left.primaryStyleId === right.primaryStyleId &&
    left.subStyleId === right.subStyleId &&
    left.selectedPerkIds.length === right.selectedPerkIds.length &&
    left.selectedPerkIds.every((perkId, index) => perkId === right.selectedPerkIds[index])
  )
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

export function scenarioLabel(scenario: BuildScenario): string {
  const position = rankedPositionFromScenario(scenario)
  if (position) return `排位${positionLabel(position)}`
  return scenario === 'normal-sr' ? '普通峡谷' : '极地大乱斗'
}
