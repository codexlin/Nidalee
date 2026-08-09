<template>
  <Card
    class="p-8 rounded-2xl shadow-xl bg-gradient-to-br from-white/80 to-muted/60 dark:from-background/80 dark:to-muted/40 border border-border"
  >
    <div class="space-y-6">
      <div>
        <h2 class="text-xl font-bold text-primary">智能分析设置</h2>
        <p class="text-sm text-muted-foreground">配置对局分析的深度和功能</p>
      </div>

      <div class="border-t border-dashed border-border pt-6 space-y-6">
        <!-- 基础设置 -->
        <div class="space-y-4">
          <h3 class="text-lg font-semibold text-foreground">基础设置</h3>

          <!-- 启用智能分析 -->
          <div class="flex items-center justify-between p-4 bg-muted/30 rounded-lg border border-border">
            <div class="flex-1">
              <div class="font-medium text-foreground">启用智能分析</div>
              <div class="text-sm text-muted-foreground">开启后将对对局进行智能分析</div>
            </div>
            <Switch :checked="analysisSettings.isEnabled" @update:checked="analysisSettings.toggleAnalysis" />
          </div>

          <!-- 分析深度 -->
          <div class="space-y-3">
            <div class="font-medium text-foreground">分析深度</div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <div
                :class="[
                  'p-4 rounded-lg border-2 cursor-pointer transition-all',
                  analysisSettings.isSimpleAnalysis
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'
                ]"
                @click="analysisSettings.setAnalysisDepth(AnalysisDepth.Simple)"
              >
                <div class="flex items-center gap-3">
                  <div
                    :class="[
                      'w-4 h-4 rounded-full border-2',
                      analysisSettings.isSimpleAnalysis ? 'border-primary bg-primary' : 'border-muted-foreground'
                    ]"
                  ></div>
                  <div>
                    <div class="font-medium text-foreground">简单分析</div>
                    <div class="text-xs text-muted-foreground">基础统计和简单建议</div>
                  </div>
                </div>
              </div>

              <div
                :class="[
                  'p-4 rounded-lg border-2 cursor-pointer transition-all',
                  analysisSettings.isDeepAnalysis
                    ? 'border-primary bg-primary/10'
                    : 'border-border hover:border-primary/50'
                ]"
                @click="analysisSettings.setAnalysisDepth(AnalysisDepth.Deep)"
              >
                <div class="flex items-center gap-3">
                  <div
                    :class="[
                      'w-4 h-4 rounded-full border-2',
                      analysisSettings.isDeepAnalysis ? 'border-primary bg-primary' : 'border-muted-foreground'
                    ]"
                  ></div>
                  <div>
                    <div class="font-medium text-foreground">深度分析</div>
                    <div class="text-xs text-muted-foreground">完整的多层分析和智能建议</div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <p class="text-sm text-muted-foreground">
            默认分析策略跟随「游戏设置 → 默认战绩模式」，无需在此单独配置。
          </p>
        </div>

        <!-- 高级功能设置 -->
        <div v-if="analysisSettings.isEnabled" class="space-y-4">
          <h3 class="text-lg font-semibold text-foreground">高级功能</h3>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div
              v-for="feature in analysisFeatures"
              :key="feature.key"
              class="flex items-center justify-between p-4 bg-muted/30 rounded-lg border border-border"
            >
              <div class="flex-1">
                <div class="font-medium text-foreground">{{ feature.label }}</div>
                <div class="text-sm text-muted-foreground">{{ feature.description }}</div>
              </div>
              <Switch
                :checked="
                  analysisSettings.analysisFeatures[feature.key as keyof typeof analysisSettings.analysisFeatures]
                "
                @update:checked="
                  (enabled: boolean) =>
                    analysisSettings.toggleAnalysisFeature(
                      feature.key as keyof typeof analysisSettings.analysisFeatures,
                      enabled
                    )
                "
              />
            </div>
          </div>
        </div>

        <!-- 显示设置 -->
        <div v-if="analysisSettings.isEnabled" class="space-y-4">
          <h3 class="text-lg font-semibold text-foreground">显示设置</h3>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div
              v-for="feature in displayFeatures"
              :key="feature.key"
              class="flex items-center justify-between p-4 bg-muted/30 rounded-lg border border-border"
            >
              <div class="flex-1">
                <div class="font-medium text-foreground">{{ feature.label }}</div>
                <div class="text-sm text-muted-foreground">{{ feature.description }}</div>
              </div>
              <Switch
                :checked="
                  analysisSettings.displayFeatures[feature.key as keyof typeof analysisSettings.displayFeatures]
                "
                @update:checked="
                  (enabled: boolean) =>
                    analysisSettings.toggleDisplayFeature(
                      feature.key as keyof typeof analysisSettings.displayFeatures,
                      enabled
                    )
                "
              />
            </div>
          </div>
        </div>

        <!-- 性能设置 -->
        <div v-if="analysisSettings.isEnabled" class="space-y-4">
          <h3 class="text-lg font-semibold text-foreground">性能设置</h3>

          <div class="space-y-4">
            <!-- 最大分析对局数 -->
            <div class="space-y-3">
              <div class="font-medium text-foreground">最大分析对局数</div>
              <div class="flex items-center gap-4">
                <Slider
                  :value="[analysisSettings.config.maxAnalysisGames]"
                  @update:value="
                    (value: number[]) => analysisSettings.setPerformanceSettings({ maxAnalysisGames: value[0] })
                  "
                  :min="5"
                  :max="50"
                  :step="5"
                  class="flex-1"
                />
                <div class="w-16 text-center font-medium text-foreground">
                  {{ analysisSettings.config.maxAnalysisGames }}
                </div>
              </div>
              <div class="text-xs text-muted-foreground">更多对局提供更准确的分析，但会增加处理时间</div>
            </div>

            <!-- 会话缓存说明（无独立「分析结果缓存」开关；时间线固定 TTL） -->
            <div class="p-4 bg-muted/30 rounded-lg border border-border space-y-1">
              <div class="font-medium text-foreground">时间线会话缓存</div>
              <div class="text-sm text-muted-foreground">
                深度分析会在内存中缓存时间线约 10 分钟（进程内 LRU），无需手动开关。完整「分析结果缓存」尚未接入后端。
              </div>
            </div>
          </div>
        </div>

        <!-- 本地 AI（BYOK，默认关闭，显式启用） -->
        <div class="space-y-4">
          <h3 class="text-lg font-semibold text-foreground">本地 AI 解读（BYOK）</h3>
          <p class="text-sm text-muted-foreground">
            使用你自己的 OpenAI-compatible API Key。密钥只存系统凭据库，不会进入前端缓存或日志。AI 默认关闭，需手动触发解读。
          </p>

          <div class="flex items-center justify-between p-4 bg-muted/30 rounded-lg border border-border">
            <div class="flex-1">
              <div class="font-medium text-foreground">启用本地 AI</div>
              <div class="text-sm text-muted-foreground">关闭时不会发起任何外部模型请求</div>
            </div>
            <Switch :checked="aiSettings.enabled" @update:checked="(v: boolean) => aiSettings.setEnabled(v)" />
          </div>

          <div v-if="aiSettings.enabled" class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <div class="space-y-2">
                <Label>Base URL</Label>
                <Input v-model="draftBaseUrl" placeholder="https://api.openai.com/v1" />
                <p class="text-xs text-muted-foreground">
                  公网端点必须使用 HTTPS；仅 localhost 或 127.0.0.1 本机开发端点可使用 HTTP。
                </p>
              </div>
              <div class="space-y-2">
                <Label>Model</Label>
                <Input v-model="draftModel" placeholder="gpt-4o-mini" />
              </div>
            </div>

            <div class="space-y-2">
              <Label>API Key</Label>
              <div class="flex flex-col sm:flex-row gap-2">
                <Input
                  v-model="draftApiKey"
                  type="password"
                  autocomplete="off"
                  :placeholder="aiSettings.hasApiKey ? '已配置（输入新 Key 可覆盖）' : 'sk-...'"
                  class="flex-1"
                />
                <Button variant="outline" :disabled="!draftApiKey.trim() || aiBusy" @click="saveApiKey">保存 Key</Button>
                <Button variant="ghost" :disabled="!aiSettings.hasApiKey || aiBusy" @click="clearApiKey">清除</Button>
              </div>
              <p class="text-xs text-muted-foreground">
                状态：{{ aiSettings.hasApiKey ? '已配置 Key' : '未配置 Key' }} · Provider：openai-compatible
              </p>
            </div>

            <div class="flex flex-wrap gap-2">
              <Button variant="outline" :disabled="aiBusy" @click="saveEndpoint">保存端点</Button>
              <Button variant="outline" :disabled="aiBusy || !aiSettings.hasApiKey" @click="testAi">测试连接</Button>
            </div>
            <p v-if="aiStatus" class="text-sm text-muted-foreground">{{ aiStatus }}</p>
          </div>
        </div>

        <!-- 操作按钮 -->
        <div class="flex items-center justify-between pt-4 border-t border-border">
          <Button variant="outline" @click="analysisSettings.resetToDefault" class="gap-2">
            <RotateCcw class="h-4 w-4" />
            重置为默认
          </Button>

          <div class="flex items-center gap-2">
            <Button variant="outline" @click="exportSettings" class="gap-2">
              <Download class="h-4 w-4" />
              导出配置
            </Button>
            <Button variant="outline" @click="importSettings" class="gap-2">
              <Upload class="h-4 w-4" />
              导入配置
            </Button>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useAnalysisSettingsStore, AnalysisDepth } from '@/shared/stores/features/analysisSettingsStore'
import { useAiSettingsStore } from '@/shared/stores/features/aiSettingsStore'
import { RotateCcw, Download, Upload } from 'lucide-vue-next'

// 组件导入
import { Card } from '@/components/ui/card'
import { Switch } from '@/components/ui/switch'
import { Slider } from '@/components/ui/slider'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'

const analysisSettings = useAnalysisSettingsStore()
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

// 分析功能选项
const analysisFeatures = computed(() => [
  {
    key: 'timeline',
    label: '时间线分析',
    description: '分析对线期和发育曲线'
  },
  {
    key: 'opponent',
    label: '对手分析',
    description: '分析对线对手的表现'
  },
  {
    key: 'teammate',
    label: '队友分析',
    description: '预留开关（Evidence 尚未接入该维度，当前不会声明可用）'
  },
  {
    key: 'selfImprovement',
    label: '自我提升',
    description: '预留开关（Evidence 尚未接入该维度，当前不会声明可用）'
  }
])

// 显示功能选项
const displayFeatures = computed(() => [
  {
    key: 'detailedAdvice',
    label: '详细建议',
    description: '显示详细的分析建议'
  },
  {
    key: 'positionComparison',
    label: '位置对比',
    description: '显示不同位置的对比分析'
  },
  {
    key: 'championPool',
    label: '英雄池分析',
    description: '显示英雄使用情况分析'
  },
  {
    key: 'trendCharts',
    label: '趋势图表',
    description: '显示胜率和表现趋势'
  }
])

// 导出设置
const exportSettings = () => {
  try {
    const config = analysisSettings.exportConfig()
    const blob = new Blob([config], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'analysis-settings.json'
    a.click()
    URL.revokeObjectURL(url)
  } catch (error) {
    console.error('导出设置失败:', error)
  }
}

// 导入设置
const importSettings = () => {
  const input = document.createElement('input')
  input.type = 'file'
  input.accept = '.json'
  input.onchange = (e) => {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (file) {
      const reader = new FileReader()
      reader.onload = (e) => {
        try {
          const content = e.target?.result as string
          analysisSettings.importConfig(content)
        } catch (error) {
          console.error('导入设置失败:', error)
        }
      }
      reader.readAsText(file)
    }
  }
  input.click()
}
</script>
