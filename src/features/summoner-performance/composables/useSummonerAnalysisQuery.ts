import { invoke } from '@tauri-apps/api/core'
import { useQuery } from '@tanstack/vue-query'
import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import {
  PERFORMANCE_SAMPLE_SIZE,
  performanceScopeKey,
  performanceScopeToAnalysis,
  type PerformanceScope
} from '@/common/performanceScope'

const OVERVIEW_FEATURES: AnalysisFeatureFlags = {
  enabled: true,
  timeline: false,
  opponent: false,
  teammate: false,
  selfImprovement: false
}

export function buildSummonerAnalysisRequest(scope: PerformanceScope): MatchAnalysisRequest {
  const { mode, queueId } = performanceScopeToAnalysis(scope)
  const ranked = scope.category === 'ranked'
  return {
    count: PERFORMANCE_SAMPLE_SIZE,
    mode,
    // 排位保留位置画像与确定性深度结论；其他模式只做轻量统计。
    // 时间线等昂贵子能力仍关闭，因此切换范围不会触发逐场时间线请求。
    depth: ranked ? 'deep' : 'simple',
    queueId,
    features: { ...OVERVIEW_FEATURES },
    maxAnalysisGames: ranked ? PERFORMANCE_SAMPLE_SIZE : undefined
  }
}

function snapshotPerformanceScope(scope: PerformanceScope): PerformanceScope {
  return scope.category === 'other'
    ? { category: 'other', rankedScope: 'mixed' }
    : { category: 'ranked', rankedScope: scope.rankedScope }
}

export function summonerAnalysisQueryKey(puuid: string, scope: PerformanceScope) {
  return ['summoner-analysis', puuid, performanceScopeKey(scope), snapshotPerformanceScope(scope)] as const
}

type SummonerAnalysisQueryKey = ReturnType<typeof summonerAnalysisQueryKey>

export async function fetchSummonerAnalysis(puuid: string, scope: PerformanceScope): Promise<MatchAnalysisResult> {
  return invoke<MatchAnalysisResult>('analyze_matches', {
    puuid,
    request: buildSummonerAnalysisRequest(scope)
  })
}

export function fetchSummonerAnalysisForQuery({
  queryKey: [, puuid, , scope]
}: {
  queryKey: SummonerAnalysisQueryKey
}): Promise<MatchAnalysisResult> {
  return fetchSummonerAnalysis(puuid, scope)
}

interface UseSummonerAnalysisQueryOptions {
  puuid: MaybeRefOrGetter<string | null | undefined>
  scope: MaybeRefOrGetter<PerformanceScope>
  enabled?: MaybeRefOrGetter<boolean>
}

export function useSummonerAnalysisQuery(options: UseSummonerAnalysisQueryOptions) {
  const puuid = computed(() => toValue(options.puuid)?.trim() ?? '')
  const scope = computed(() => toValue(options.scope))
  const enabled = computed(() => (options.enabled === undefined ? true : toValue(options.enabled)) && !!puuid.value)

  return useQuery<MatchAnalysisResult, Error, MatchAnalysisResult, SummonerAnalysisQueryKey>({
    queryKey: computed(() => summonerAnalysisQueryKey(puuid.value, scope.value)),
    queryFn: fetchSummonerAnalysisForQuery,
    enabled,
    staleTime: 5 * 60 * 1000
  })
}
