import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

/** 非敏感 AI 配置（Key 只存系统凭据库，永不进入本 store） */
export interface AiSettingsPublic {
  enabled: boolean
  provider: string
  baseUrl: string
  model: string
  hasApiKey: boolean
}

const defaults: Omit<AiSettingsPublic, 'hasApiKey'> = {
  enabled: false,
  provider: 'openai-compatible',
  baseUrl: 'https://api.openai.com/v1',
  model: 'gpt-4o-mini'
}

export const useAiSettingsStore = defineStore(
  'aiSettings',
  () => {
    const enabled = ref(defaults.enabled)
    const provider = ref(defaults.provider)
    const baseUrl = ref(defaults.baseUrl)
    const model = ref(defaults.model)
    const hasApiKey = ref(false)
    const hydrated = ref(false)

    const publicView = computed<AiSettingsPublic>(() => ({
      enabled: enabled.value,
      provider: provider.value,
      baseUrl: baseUrl.value,
      model: model.value,
      hasApiKey: hasApiKey.value
    }))

    const applyPublic = (settings: AiSettingsPublic) => {
      enabled.value = settings.enabled
      provider.value = settings.provider
      baseUrl.value = settings.baseUrl
      model.value = settings.model
      hasApiKey.value = settings.hasApiKey
    }

    /** 从后端读取（含 hasApiKey）；合并本地持久化的非敏感字段 */
    const hydrateFromBackend = async () => {
      try {
        const remote = await invoke<AiSettingsPublic>('get_ai_settings')
        // 后端进程内存可能已重置；用本地持久化的非敏感配置覆盖并回写
        const merged: AiSettingsPublic = {
          enabled: enabled.value,
          provider: provider.value || remote.provider,
          baseUrl: baseUrl.value || remote.baseUrl,
          model: model.value || remote.model,
          hasApiKey: remote.hasApiKey
        }
        applyPublic(merged)
        await invoke<AiSettingsPublic>('set_ai_settings', {
          enabled: merged.enabled,
          baseUrl: merged.baseUrl,
          model: merged.model
        })
        hasApiKey.value = (await invoke<AiSettingsPublic>('get_ai_settings')).hasApiKey
      } catch (e) {
        console.warn('[aiSettings] hydrate failed', e)
      } finally {
        hydrated.value = true
      }
    }

    const syncToBackend = async () => {
      const updated = await invoke<AiSettingsPublic>('set_ai_settings', {
        enabled: enabled.value,
        baseUrl: baseUrl.value,
        model: model.value
      })
      hasApiKey.value = updated.hasApiKey
      return updated
    }

    const setEnabled = async (value: boolean) => {
      enabled.value = value
      await syncToBackend()
    }

    const setEndpoint = async (nextBaseUrl: string, nextModel: string) => {
      const normalized = nextBaseUrl.trim().replace(/\/+$/, '')
      if (!/^https?:\/\/[^/\s]+/i.test(normalized)) {
        throw new Error('Base URL 须以 http:// 或 https:// 开头，并包含有效主机名')
      }
      baseUrl.value = normalized
      model.value = nextModel.trim()
      await syncToBackend()
    }

    /** Key 只发往后端 keyring，不在本 store 保留明文 */
    const saveApiKey = async (apiKey: string) => {
      await invoke<boolean>('set_ai_api_key', { apiKey })
      hasApiKey.value = true
      await syncToBackend()
    }

    const clearApiKey = async () => {
      await invoke<boolean>('clear_ai_api_key')
      hasApiKey.value = false
    }

    const testConnection = async () => {
      await syncToBackend()
      return invoke<string>('test_ai_connection')
    }

    return {
      enabled,
      provider,
      baseUrl,
      model,
      hasApiKey,
      hydrated,
      publicView,
      hydrateFromBackend,
      syncToBackend,
      setEnabled,
      setEndpoint,
      saveApiKey,
      clearApiKey,
      testConnection
    }
  },
  {
    persist: {
      // 绝不持久化 Key；hasApiKey 每次从后端刷新
      pick: ['enabled', 'provider', 'baseUrl', 'model']
    }
  }
)
