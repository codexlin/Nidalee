export type PerformanceCategory = 'ranked' | 'other'

export type RankedScope = 'mixed' | 'solo' | 'flex'

export interface PerformanceScope {
  category: PerformanceCategory
  rankedScope: RankedScope
}

export const DEFAULT_PERFORMANCE_SCOPE: PerformanceScope = {
  category: 'ranked',
  rankedScope: 'mixed'
}

export const PERFORMANCE_SAMPLE_SIZE = 20

export function performanceScopeKey(scope: PerformanceScope): string {
  return scope.category === 'ranked' ? `ranked:${scope.rankedScope}` : 'other'
}

export function performanceScopeLabel(scope: PerformanceScope): string {
  if (scope.category === 'other') return '其他模式'
  if (scope.rankedScope === 'solo') return '单双排'
  if (scope.rankedScope === 'flex') return '灵活组排'
  return '排位综合'
}

export function performanceScopeToAnalysis(scope: PerformanceScope): {
  mode: AnalysisMode
  queueId?: number
} {
  if (scope.category === 'other') return { mode: 'normals' }
  if (scope.rankedScope === 'solo') return { mode: 'soloRanked', queueId: 420 }
  if (scope.rankedScope === 'flex') return { mode: 'flexRanked', queueId: 440 }
  return { mode: 'mixedRanked' }
}

export function normalizePerformanceCategory(value: string): PerformanceCategory {
  return value === 'other' ? 'other' : 'ranked'
}

export function normalizeRankedScope(value: string): RankedScope {
  if (value === 'solo' || value === 'flex') return value
  return 'mixed'
}
