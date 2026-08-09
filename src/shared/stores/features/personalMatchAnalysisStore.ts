import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

/**
 * 个人战绩统一分析结果（Dashboard 单次 analyze_matches 的唯一状态源）
 */
export const usePersonalMatchAnalysisStore = defineStore('personalMatchAnalysis', () => {
  const result = ref<MatchAnalysisResult | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const lastPuuid = ref<string | null>(null)

  const overallStats = computed(() => result.value?.overallStats ?? null)
  const positionStats = computed(() => result.value?.positionStats ?? [])
  const mainPosition = computed(() => result.value?.mainPosition ?? 'UNKNOWN')
  const matches = computed(() => result.value?.matches ?? [])
  const traits = computed(() => result.value?.traits ?? [])
  const advice = computed(() => result.value?.advice ?? [])
  const capabilities = computed(() => result.value?.capabilities ?? null)
  const diagnostics = computed(() => result.value?.diagnostics ?? [])
  const policy = computed(() => result.value?.policy ?? null)
  const aiInsight = computed(() => result.value?.aiInsight ?? null)
  const evidence = computed(() => result.value?.evidence ?? null)

  const getMatchEvidence = (gameId: number): MatchEvidence | null => {
    const matches = result.value?.evidence?.matches
    if (!matches?.length) return null
    return matches.find((m) => m.gameId === gameId) ?? null
  }

  const multiPositionView = computed<MultiPositionAnalysis | null>(() => {
    if (!result.value) return null
    return {
      positionStats: result.value.positionStats,
      mainPosition: result.value.mainPosition,
      overallStats: result.value.overallStats
    }
  })

  const setResult = (next: MatchAnalysisResult | null, puuid?: string) => {
    result.value = next
    if (puuid !== undefined) lastPuuid.value = puuid
    error.value = null
  }

  const setLoading = (value: boolean) => {
    loading.value = value
  }

  const setError = (message: string | null) => {
    error.value = message
  }

  const setAiInsight = (insight: AiInsight | null) => {
    if (!result.value) return
    result.value = { ...result.value, aiInsight: insight ?? undefined }
  }

  const clear = () => {
    result.value = null
    error.value = null
    lastPuuid.value = null
  }

  return {
    result,
    loading,
    error,
    lastPuuid,
    overallStats,
    positionStats,
    mainPosition,
    matches,
    traits,
    advice,
    capabilities,
    diagnostics,
    policy,
    aiInsight,
    evidence,
    multiPositionView,
    getMatchEvidence,
    setResult,
    setLoading,
    setError,
    setAiInsight,
    clear
  }
})
