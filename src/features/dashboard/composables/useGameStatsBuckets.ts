import { getMatchModeLabel, type MatchModeKey } from '@/common/queueCatalog'
import { computed, ref, watch } from 'vue'
import { inferModeAffinityTraits } from '../utils/inferModeAffinityTraits'

export type StatsBucket = 'ranked' | 'other'

export type GameStatsBucketProps = {
  matchStatistics: PlayerMatchStats | null
  rankedStats?: PlayerMatchStats | null
  otherStats?: PlayerMatchStats | null
  analysisTraits?: DeterministicTrait[] | null
  positionStats?: PositionStats[] | null
  mainPosition?: string | null
  selectedMatchMode?: MatchModeKey
  matchCount?: number
  scannedGames?: number | null
  displayGames?: number | null
}

/**
 * GameStats 双桶 / 空态 / 列表切片。
 * 调用方传入 `<script setup>` 的 reactive `props`（勿先解构），以保持追踪。
 */
export function useGameStatsBuckets(props: GameStatsBucketProps) {
  const isFilterEmpty = computed(() => !!props.matchStatistics && (props.matchStatistics.totalGames || 0) === 0)

  const hasGames = computed(() => (props.matchStatistics?.totalGames || 0) > 0)

  const resolvedDisplayGames = computed(() => {
    if (props.displayGames !== null && props.displayGames !== undefined && props.displayGames > 0) {
      return props.displayGames
    }
    return props.matchStatistics?.totalGames || 0
  })

  const rankedBucket = computed(() => {
    const stats = props.rankedStats
    return stats && (stats.totalGames || 0) > 0 ? stats : null
  })

  const otherBucket = computed(() => {
    const stats = props.otherStats
    return stats && (stats.totalGames || 0) > 0 ? stats : null
  })

  /** 全部模式 / 搜索页：有两侧样本时展示双桶切换 */
  const showBucketTabs = computed(() => {
    const mode = props.selectedMatchMode
    const dualContext = mode === undefined || mode === 'all'
    return dualContext && !!rankedBucket.value && !!otherBucket.value
  })

  const bucketTabOptions = computed(() => {
    const tabs: { key: StatsBucket; label: string; games: number }[] = []
    if (rankedBucket.value) {
      tabs.push({ key: 'ranked', label: '排位', games: rankedBucket.value.totalGames || 0 })
    }
    if (otherBucket.value) {
      tabs.push({ key: 'other', label: '其他', games: otherBucket.value.totalGames || 0 })
    }
    return tabs
  })

  const activeBucket = ref<StatsBucket>('ranked')

  watch(
    [rankedBucket, otherBucket, () => props.selectedMatchMode],
    () => {
      const mode = props.selectedMatchMode
      if (mode === 'normals') {
        activeBucket.value = 'other'
        return
      }
      if (mode === 'mixedRanked' || mode === '420' || mode === '440') {
        activeBucket.value = 'ranked'
        return
      }
      activeBucket.value = rankedBucket.value ? 'ranked' : 'other'
    },
    { immediate: true }
  )

  const isRankedBucketActive = computed(() => {
    const mode = props.selectedMatchMode
    if (mode === 'normals') return false
    if (mode === 'mixedRanked' || mode === '420' || mode === '440') return true
    if (showBucketTabs.value) return activeBucket.value === 'ranked'
    if (mode === undefined || mode === 'all') return !!rankedBucket.value
    return false
  })

  const bucketStatistics = computed(() => {
    if (isRankedBucketActive.value) {
      return rankedBucket.value ?? props.matchStatistics
    }
    return otherBucket.value ?? props.matchStatistics
  })

  const bucketFilterMode = computed<MatchModeKey | undefined>(() => {
    if (isRankedBucketActive.value) {
      const mode = props.selectedMatchMode
      if (mode === '420' || mode === '440' || mode === 'mixedRanked') return mode
      return 'mixedRanked'
    }
    return 'normals'
  })

  const bucketPositionStats = computed(() => (isRankedBucketActive.value ? props.positionStats : null))
  const bucketMainPosition = computed(() => (isRankedBucketActive.value ? props.mainPosition : null))

  const bucketTraits = computed(() => {
    const traits = props.analysisTraits || []
    if (isRankedBucketActive.value) {
      return traits.filter((t) => !t.key.startsWith('mode_affinity') || t.key === 'mode_affinity_ranked')
    }
    const fromProps = traits.filter((t) => t.key.startsWith('mode_affinity') || t.key.startsWith('fun_'))
    const affinityFromProps = fromProps.filter(
      (t) => t.supportsConclusion && t.key.startsWith('mode_affinity') && t.key !== 'mode_affinity_ranked'
    )
    if (affinityFromProps.length) return fromProps
    return inferModeAffinityTraits(bucketStatistics.value?.recentPerformance)
  })

  const sampleShortfallTip = computed(() => {
    if (!hasGames.value) return ''
    const n = bucketStatistics.value?.totalGames || resolvedDisplayGames.value
    const requested = props.matchCount
    if (requested === null || requested === undefined || n <= 0 || n >= requested) return ''
    if (showBucketTabs.value) {
      const label = isRankedBucketActive.value ? '排位' : '其他'
      return `已选 ${requested} 场，其中「${label}」近期仅有 ${n} 场`
    }
    const mode =
      props.selectedMatchMode && props.selectedMatchMode !== 'all'
        ? `「${getMatchModeLabel(props.selectedMatchMode)}」`
        : '当前模式'
    return `已选 ${requested} 场，${mode}近期仅有 ${n} 场`
  })

  const listGames = computed(() => bucketStatistics.value?.recentPerformance || [])

  const initialShowCount = 10
  const showCount = ref(initialShowCount)
  const loadMore = () => {
    showCount.value += 10
  }

  watch(listGames, () => {
    showCount.value = initialShowCount
  })

  const recentResultDots = computed(() => {
    const list = listGames.value
    if (!list.length) return [] as boolean[]
    const chronological = [...list].reverse()
    return chronological.slice(-20).map((g) => !!g.win)
  })

  const winRateToneClass = computed(() => {
    const rate = bucketStatistics.value?.winRate ?? 0
    if (rate > 50) return 'text-emerald-600 dark:text-emerald-400'
    if (rate < 50) return 'text-rose-600 dark:text-rose-400'
    return 'text-foreground'
  })

  const emptyTitle = computed(() => {
    const mode = props.selectedMatchMode
    if (mode && mode !== 'all') {
      return `最近没有「${getMatchModeLabel(mode)}」对局`
    }
    return '最近没有可展示的对局'
  })

  const emptyDetail = computed(() => {
    const scanned = props.scannedGames
    if (scanned && scanned > 0) {
      return `已查看最近 ${scanned} 场历史。可以换到全部模式，或打几场后再刷新。`
    }
    return '可以换到全部模式，或打几场后再刷新。'
  })

  const favoriteChampions = computed(() => (bucketStatistics.value?.favoriteChampions || []).slice(0, 5))
  const hasFavoriteChampions = computed(() => favoriteChampions.value.length > 0)

  const hasPositionStats = computed(() =>
    (bucketPositionStats.value || []).some((pos) => pos.position !== 'UNKNOWN' && pos.games > 0)
  )

  const hasIdentitySection = computed(() => {
    if (isRankedBucketActive.value) return hasPositionStats.value
    return (
      bucketTraits.value?.some(
        (t) => t.supportsConclusion && t.key.startsWith('mode_affinity') && t.key !== 'mode_affinity_ranked'
      ) ?? false
    )
  })

  return {
    isFilterEmpty,
    hasGames,
    showBucketTabs,
    bucketTabOptions,
    activeBucket,
    isRankedBucketActive,
    bucketStatistics,
    bucketFilterMode,
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
