import { invoke } from '@tauri-apps/api/core'
import { usePersonalMatchAnalysisStore } from '@/shared/stores/features/personalMatchAnalysisStore'
import { useAiSettingsStore } from '@/shared/stores/features/aiSettingsStore'

/**
 * 本地 BYOK AI 解读：显式触发，失败不影响本地分析结果
 */
export function useAiAnalysis() {
  const aiSettings = useAiSettingsStore()
  const analysisStore = usePersonalMatchAnalysisStore()
  const activityStore = useActivityStore()

  const loading = ref(false)
  const error = ref<string | null>(null)
  const preview = ref<unknown | null>(null)
  let requestSequence = 0

  const beginRequest = () => ({
    sequence: ++requestSequence,
    resultRevision: analysisStore.resultRevision
  })

  const isCurrentRequest = (request: { sequence: number; resultRevision: number }) =>
    request.sequence === requestSequence && request.resultRevision === analysisStore.resultRevision

  const ensureSynced = async () => {
    await aiSettings.hydrateFromBackend()
  }

  const previewPrompt = async () => {
    const result = analysisStore.result
    if (!result) {
      error.value = '请先完成战绩分析'
      return null
    }
    loading.value = true
    error.value = null
    const request = beginRequest()
    try {
      const bundle = await invoke<unknown>('preview_ai_prompt', { result })
      if (!isCurrentRequest(request)) return null
      preview.value = bundle
      return bundle
    } catch (e: unknown) {
      if (!isCurrentRequest(request)) return null
      const message = e instanceof Error ? e.message : String(e)
      error.value = message
      return null
    } finally {
      if (isCurrentRequest(request)) loading.value = false
    }
  }

  const analyzeWithAi = async (): Promise<AiInsight | null> => {
    const result = analysisStore.result
    if (!result) {
      error.value = '请先完成战绩分析'
      return null
    }
    if (!aiSettings.enabled) {
      error.value = '请先在设置中启用本地 AI'
      return null
    }
    if (!aiSettings.hasApiKey) {
      error.value = '尚未配置 API Key'
      return null
    }

    loading.value = true
    error.value = null
    const request = beginRequest()
    try {
      await aiSettings.syncToBackend()
      if (!isCurrentRequest(request)) return null
      const insight = await invoke<AiInsight>('analyze_with_ai', { result })
      if (!isCurrentRequest(request)) return null
      analysisStore.setAiInsight(insight)
      activityStore.addActivity('success', 'AI 解读完成', 'data')
      return insight
    } catch (e: unknown) {
      if (!isCurrentRequest(request)) return null
      const message = e instanceof Error ? e.message : String(e)
      error.value = message
      activityStore.addActivity('error', `AI 解读失败: ${message}`, 'error')
      return null
    } finally {
      if (isCurrentRequest(request)) loading.value = false
    }
  }

  watch(
    () => analysisStore.resultRevision,
    () => {
      requestSequence += 1
      loading.value = false
      error.value = null
      preview.value = null
    }
  )

  return {
    loading,
    error,
    preview,
    ensureSynced,
    previewPrompt,
    analyzeWithAi,
    aiSettings,
    aiInsight: computed(() => analysisStore.aiInsight)
  }
}
