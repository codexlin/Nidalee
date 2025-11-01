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

          <!-- 默认分析模式 -->
          <div class="space-y-3">
            <div class="font-medium text-foreground">默认分析模式</div>
            <Select :value="analysisSettings.config.defaultMode" @update:value="analysisSettings.setDefaultMode">
              <SelectTrigger class="w-full">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="mode in analysisModes" :key="mode.value" :value="mode.value">
                  <div>
                    <div class="font-medium">{{ mode.label }}</div>
                    <div class="text-xs text-muted-foreground">{{ mode.description }}</div>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
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

            <!-- 启用缓存 -->
            <div class="flex items-center justify-between p-4 bg-muted/30 rounded-lg border border-border">
              <div class="flex-1">
                <div class="font-medium text-foreground">启用分析缓存</div>
                <div class="text-sm text-muted-foreground">缓存分析结果以提高性能</div>
              </div>
              <Switch
                :checked="analysisSettings.config.enableCaching"
                @update:checked="
                  (enabled: boolean) => analysisSettings.setPerformanceSettings({ enableCaching: enabled })
                "
              />
            </div>

            <!-- 缓存过期时间 -->
            <div v-if="analysisSettings.config.enableCaching" class="space-y-3">
              <div class="font-medium text-foreground">缓存过期时间</div>
              <div class="flex items-center gap-4">
                <Slider
                  :value="[analysisSettings.config.cacheExpirationHours]"
                  @update:value="
                    (value: number[]) => analysisSettings.setPerformanceSettings({ cacheExpirationHours: value[0] })
                  "
                  :min="1"
                  :max="168"
                  :step="1"
                  class="flex-1"
                />
                <div class="w-20 text-center font-medium text-foreground">
                  {{ analysisSettings.config.cacheExpirationHours }}小时
                </div>
              </div>
            </div>
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
import { computed } from 'vue'
import { useAnalysisSettingsStore, AnalysisDepth, AnalysisMode } from '@/shared/stores/features/analysisSettingsStore'
import { RotateCcw, Download, Upload } from 'lucide-vue-next'

// 组件导入
import Card from '@/shared/components/ui/card/Card.vue'
import Switch from '@/shared/components/ui/switch/Switch.vue'
import Select from '@/shared/components/ui/select/Select.vue'
import SelectContent from '@/shared/components/ui/select/SelectContent.vue'
import SelectItem from '@/shared/components/ui/select/SelectItem.vue'
import SelectTrigger from '@/shared/components/ui/select/SelectTrigger.vue'
import SelectValue from '@/shared/components/ui/select/SelectValue.vue'
import Slider from '@/shared/components/ui/slider/Slider.vue'
import Button from '@/shared/components/ui/button/Button.vue'

const analysisSettings = useAnalysisSettingsStore()

// 分析模式选项
const analysisModes = computed(() => [
  {
    value: AnalysisMode.SoloRanked,
    label: '单排分析',
    description: '只分析单排对局 (420)'
  },
  {
    value: AnalysisMode.FlexRanked,
    label: '灵活组排分析',
    description: '只分析灵活组排对局 (440)'
  },
  {
    value: AnalysisMode.MixedRanked,
    label: '混合排位分析',
    description: '分析单排+灵活组排对局 (420+440)'
  },
  {
    value: AnalysisMode.Aram,
    label: '大乱斗分析',
    description: '只分析大乱斗对局 (450)'
  },
  {
    value: AnalysisMode.AllModes,
    label: '全部模式分析',
    description: '分析所有对局'
  }
])

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
    description: '分析队友协同配合'
  },
  {
    key: 'selfImprovement',
    label: '自我提升',
    description: '提供个人改进建议'
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
