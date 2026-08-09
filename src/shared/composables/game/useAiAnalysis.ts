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
    try {
      const bundle = await invoke<unknown>('preview_ai_prompt', { result })
      preview.value = bundle
      return bundle
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : String(e)
      error.value = message
      return null
    } finally {
      loading.value = false
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
    try {
      await aiSettings.syncToBackend()
      const insight = await invoke<AiInsight>('analyze_with_ai', { result })
      analysisStore.setAiInsight(insight)
      activityStore.addActivity('success', 'AI 解读完成', 'data')
      return insight
    } catch (e: unknown) {
      const message = e instanceof Error ? e.message : String(e)
      error.value = message
      activityStore.addActivity('error', `AI 解读失败: ${message}`, 'error')
      return null
    } finally {
      loading.value = false
    }
  }

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
