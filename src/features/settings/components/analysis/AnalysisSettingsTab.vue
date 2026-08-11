<template>
  <div class="space-y-4">
    <Card class="gap-0 overflow-hidden py-0">
      <div class="space-y-0.5 border-b border-border/50 px-4 py-4 sm:px-5">
        <h2 class="text-lg font-medium leading-tight">智能分析</h2>
        <p class="text-xs text-muted-foreground">
          仪表盘固定做基础统计（不批量拉时间线）；排位过程复盘在对局详情内按需加载。此处只配置本地 AI。
        </p>
      </div>

      <section class="space-y-3 px-4 py-4 sm:px-5">
        <div class="space-y-0.5">
          <h3 class="text-sm font-medium">本地 AI 解读（BYOK）</h3>
          <p class="text-xs text-muted-foreground">
            使用你自己的 OpenAI-compatible API Key。密钥只存系统凭据库；默认关闭，需在仪表盘手动触发。AI
            解读还需当前样本具备排位深度证据。
          </p>
        </div>

        <div class="flex items-center justify-between gap-4 rounded-xl surface-inset px-3 py-3">
          <div class="min-w-0 space-y-0.5">
            <div class="text-sm font-medium">启用本地 AI</div>
            <div class="text-xs text-muted-foreground">关闭后仪表盘不展示 AI 入口</div>
          </div>
          <Switch
            :model-value="aiSettings.enabled"
            @update:model-value="(v: boolean) => aiSettings.setEnabled(v)"
          />
        </div>

        <div v-if="aiSettings.enabled" class="space-y-3">
          <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
            <div class="space-y-1.5">
              <Label class="text-sm font-medium">Base URL</Label>
              <Input v-model="draftBaseUrl" class="h-9 text-sm" placeholder="https://api.openai.com/v1" />
              <p class="text-xs text-muted-foreground">公网须 HTTPS；仅 localhost / 127.0.0.1 可用 HTTP。</p>
            </div>
            <div class="space-y-1.5">
              <Label class="text-sm font-medium">Model</Label>
              <Input v-model="draftModel" class="h-9 text-sm" placeholder="gpt-4o-mini" />
            </div>
          </div>

          <div class="space-y-1.5">
            <Label class="text-sm font-medium">API Key</Label>
            <div class="flex flex-col gap-2 sm:flex-row">
              <Input
                v-model="draftApiKey"
                type="password"
                autocomplete="off"
                class="h-9 flex-1 text-sm"
                :placeholder="aiSettings.hasApiKey ? '已配置（输入新 Key 可覆盖）' : 'sk-...'"
              />
              <Button
                variant="outline"
                size="sm"
                class="h-9"
                :disabled="!draftApiKey.trim() || aiBusy"
                @click="saveApiKey"
              >
                保存 Key
              </Button>
              <Button
                variant="ghost"
                size="sm"
                class="h-9"
                :disabled="!aiSettings.hasApiKey || aiBusy"
                @click="clearApiKey"
              >
                清除
              </Button>
            </div>
            <p class="text-xs text-muted-foreground">
              状态：{{ aiSettings.hasApiKey ? '已配置 Key' : '未配置 Key' }} · Provider：openai-compatible
            </p>
          </div>

          <div class="flex flex-wrap gap-2">
            <Button variant="outline" size="sm" class="h-8" :disabled="aiBusy" @click="saveEndpoint">
              保存端点
            </Button>
            <Button
              variant="outline"
              size="sm"
              class="h-8"
              :disabled="aiBusy || !aiSettings.hasApiKey"
              @click="testAi"
            >
              测试连接
            </Button>
          </div>
          <p v-if="aiStatus" class="text-xs text-muted-foreground">{{ aiStatus }}</p>
        </div>
      </section>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { useAiSettingsStore } from '@/shared/stores/features/aiSettingsStore'

const aiSettings = useAiSettingsStore()

const draftBaseUrl = ref(aiSettings.baseUrl)
const draftModel = ref(aiSettings.model)
const draftApiKey = ref('')
const aiBusy = ref(false)
const aiStatus = ref('')

onMounted(async () => {
  await aiSettings.hydrateFromBackend()
  draftBaseUrl.value = aiSettings.baseUrl
  draftModel.value = aiSettings.model
})

const saveEndpoint = async () => {
  aiBusy.value = true
  aiStatus.value = ''
  try {
    await aiSettings.setEndpoint(draftBaseUrl.value, draftModel.value)
    aiStatus.value = '端点已保存'
  } catch (e: unknown) {
    aiStatus.value = e instanceof Error ? e.message : String(e)
  } finally {
    aiBusy.value = false
  }
}

const saveApiKey = async () => {
  aiBusy.value = true
  aiStatus.value = ''
  try {
    await aiSettings.saveApiKey(draftApiKey.value)
    draftApiKey.value = ''
    aiStatus.value = 'API Key 已写入系统凭据库'
  } catch (e: unknown) {
    aiStatus.value = e instanceof Error ? e.message : String(e)
  } finally {
    aiBusy.value = false
  }
}

const clearApiKey = async () => {
  aiBusy.value = true
  aiStatus.value = ''
  try {
    await aiSettings.clearApiKey()
    draftApiKey.value = ''
    aiStatus.value = 'API Key 已清除'
  } catch (e: unknown) {
    aiStatus.value = e instanceof Error ? e.message : String(e)
  } finally {
    aiBusy.value = false
  }
}

const testAi = async () => {
  aiBusy.value = true
  aiStatus.value = ''
  try {
    await aiSettings.setEndpoint(draftBaseUrl.value, draftModel.value)
    aiStatus.value = await aiSettings.testConnection()
  } catch (e: unknown) {
    aiStatus.value = e instanceof Error ? e.message : String(e)
  } finally {
    aiBusy.value = false
  }
}
</script>
