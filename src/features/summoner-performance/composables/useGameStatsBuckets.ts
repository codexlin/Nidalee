import { computed, ref, watch } from 'vue'
import { PERFORMANCE_SAMPLE_SIZE, performanceScopeLabel, type PerformanceScope } from '@/common/performanceScope'
import { inferModeAffinityTraits } from '../utils/inferModeAffinityTraits'

export type GameStatsBucketProps = {
  matchStatistics: PlayerMatchStats | null
  analysisTraits?: DeterministicTrait[] | null
  positionStats?: PositionStats[] | null
  mainPosition?: string | null
  scope: PerformanceScope
}

/** Derives the visible summary from one already-scoped analysis result. */
export function useGameStatsBuckets(props: GameStatsBucketProps) {
  const isRanked = computed(() => props.scope.category === 'ranked')
  const isFilterEmpty = computed(() => !!props.matchStatistics && (props.matchStatistics.totalGames || 0) === 0)
  const hasGames = computed(() => (props.matchStatistics?.totalGames || 0) > 0)
  const bucketStatistics = computed(() => props.matchStatistics)
  const bucketPositionStats = computed(() => (isRanked.value ? props.positionStats : null))
  const bucketMainPosition = computed(() => (isRanked.value ? props.mainPosition : null))

  const bucketTraits = computed(() => {
    if (isRanked.value) return props.analysisTraits || []
    const traits = (props.analysisTraits || []).filter(
      (trait) => trait.key.startsWith('mode_affinity') || trait.key.startsWith('fun_')
    )
    const supportedAffinity = traits.some(
      (trait) =>
        trait.supportsConclusion && trait.key.startsWith('mode_affinity') && trait.key !== 'mode_affinity_ranked'
    )
    return supportedAffinity ? traits : inferModeAffinityTraits(props.matchStatistics?.recentPerformance)
  })

  const sampleShortfallTip = computed(() => {
    const games = props.matchStatistics?.totalGames || 0
    if (games <= 0 || games >= PERFORMANCE_SAMPLE_SIZE) return ''
    return `${performanceScopeLabel(props.scope)}近期仅有 ${games} 场有效样本`
  })

  const listGames = computed(() => props.matchStatistics?.recentPerformance || [])
  const showCount = ref(10)
  const loadMore = () => {
    showCount.value += 10
  }

  watch(listGames, () => {
    showCount.value = 10
  })

  const recentResultDots = computed(() =>
    [...listGames.value]
      .reverse()
      .slice(-20)
      .map((game) => !!game.win)
  )
  const winRateToneClass = computed(() => {
    const rate = props.matchStatistics?.winRate ?? 0
    if (rate > 50) return 'text-emerald-600 dark:text-emerald-400'
    if (rate < 50) return 'text-rose-600 dark:text-rose-400'
    return 'text-foreground'
  })

  const emptyTitle = computed(() => `最近没有「${performanceScopeLabel(props.scope)}」对局`)
  const emptyDetail = computed(() => '可以切换分析范围，或完成新对局后刷新。')
  const favoriteChampions = computed(() => (props.matchStatistics?.favoriteChampions || []).slice(0, 5))
  const hasFavoriteChampions = computed(() => favoriteChampions.value.length > 0)
  const hasPositionStats = computed(() =>
    (bucketPositionStats.value || []).some((position) => position.position !== 'UNKNOWN' && position.games > 0)
  )
  const hasIdentitySection = computed(() => {
    if (isRanked.value) return hasPositionStats.value
    return bucketTraits.value.some(
      (trait) =>
        trait.supportsConclusion && trait.key.startsWith('mode_affinity') && trait.key !== 'mode_affinity_ranked'
    )
  })

  return {
    isRanked,
    isFilterEmpty,
    hasGames,
    bucketStatistics,
    bucketPositionStats,
    bucketMainPosition,
    bucketTraits,
    sampleShortfallTip,
    listGames,
    showCount,
    loadMore,
    recentResultDots,
    winRateToneClass,
    emptyTitle,
    emptyDetail,
    favoriteChampions,
    hasFavoriteChampions,
    hasIdentitySection
  }
}
