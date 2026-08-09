<template>
  <div class="chart-container">
    <!-- 加载状态 -->
    <div v-if="loading" class="flex items-center justify-center h-full">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
    </div>

    <!-- 错误状态 -->
    <div v-else-if="error" class="flex items-center justify-center h-full text-red-500">
      <div class="text-center">
        <div class="text-lg font-semibold mb-2">图表加载失败</div>
        <div class="text-sm">{{ error }}</div>
        <Button @click="retry" variant="outline" size="sm" class="mt-2"> 重试 </Button>
      </div>
    </div>

    <!-- 图表内容 -->
    <canvas v-else ref="chartContainer" class="w-full h-full"></canvas>
  </div>
</template>

<script setup lang="ts">
import { shallowRef, ref, onMounted, onUnmounted, watch, nextTick, computed } from 'vue'
import { Button } from '@/components/ui/button'
import { useSettingsStore } from '@/shared/stores/ui/settingsStore'
import { themeColors } from '@/lib/themeColor'
import type { Chart, ChartConfiguration, ChartData, ChartOptions } from 'chart.js'

type SupportedChartType = 'radar' | 'line' | 'bar' | 'pie'

interface Props {
  type: SupportedChartType
  data: ChartData<SupportedChartType>
  options?: ChartOptions<SupportedChartType>
  height?: string
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  height: '300px',
  loading: false
})

const settingsStore = useSettingsStore()
const chartContainer = ref<HTMLCanvasElement>()
const error = ref<string | null>(null)
const chartInstance = shallowRef<Chart | null>(null)

const resolvedThemeColors = computed(() => themeColors())

// 动态导入图表库
const loadChartLibrary = async () => {
  try {
    const { Chart: ChartJS, registerables } = await import('chart.js')
    ChartJS.register(...registerables)
    return ChartJS
  } catch (err: unknown) {
    console.error('Failed to load chart library:', err)
    throw new Error('图表库加载失败')
  }
}

// 创建图表
const createChart = async () => {
  if (!chartContainer.value) return

  try {
    error.value = null
    const ChartCtor = await loadChartLibrary()

    // 销毁旧图表
    if (chartInstance.value) {
      chartInstance.value.destroy()
    }

    const colors = resolvedThemeColors.value

    // 创建新图表
    chartInstance.value = new ChartCtor(chartContainer.value, {
      type: props.type,
      data: props.data,
      options: {
        responsive: true,
        maintainAspectRatio: false,
        plugins: {
          legend: {
            position: 'top' as const,
            labels: {
              color: colors.foreground,
              font: {
                family: '"HarmonyOS Sans SC", "Microsoft YaHei", sans-serif'
              }
            }
          },
          tooltip: {
            enabled: true,
            backgroundColor: colors.card,
            titleColor: colors.cardForeground,
            bodyColor: colors.cardForeground,
            borderColor: colors.border,
            borderWidth: 1,
            cornerRadius: 6,
            displayColors: true,
            titleFont: {
              family: '"HarmonyOS Sans SC", "Microsoft YaHei", sans-serif',
              weight: 'bold'
            },
            bodyFont: {
              family: '"HarmonyOS Sans SC", "Microsoft YaHei", sans-serif'
            }
          }
        },
        scales:
          props.type !== 'radar' && props.type !== 'pie'
            ? {
                x: {
                  grid: {
                    color: colors.border,
                    drawBorder: false
                  },
                  ticks: {
                    color: colors.mutedForeground,
                    font: {
                      family: '"HarmonyOS Sans SC", "Microsoft YaHei", sans-serif'
                    }
                  }
                },
                y: {
                  grid: {
                    color: colors.border,
                    drawBorder: false
                  },
                  ticks: {
                    color: colors.mutedForeground,
                    font: {
                      family: '"HarmonyOS Sans SC", "Microsoft YaHei", sans-serif'
                    }
                  }
                }
              }
            : undefined,
        ...(props.options ?? {})
      }
    } as ChartConfiguration)
  } catch (err: unknown) {
    error.value = err instanceof Error ? err.message : '图表创建失败'
    console.error('Chart creation failed:', err)
  }
}

// 重试功能
const retry = () => {
  createChart()
}

// 监听数据变化
watch(
  () => props.data,
  () => {
    if (chartInstance.value) {
      chartInstance.value.data = props.data
      chartInstance.value.update()
    }
  },
  { deep: true }
)

// 监听类型变化
watch(
  () => props.type,
  () => {
    createChart()
  }
)

// 监听主题变化
watch(
  () => [settingsStore.isDark, settingsStore.selectedColor],
  () => {
    // 主题变化时重新创建图表以应用新颜色
    nextTick(() => {
      createChart()
    })
  },
  { deep: true }
)

onMounted(async () => {
  await nextTick()
  createChart()
})

onUnmounted(() => {
  if (chartInstance.value) {
    chartInstance.value.destroy()
  }
})
</script>

<style scoped>
.chart-container {
  width: 100%;
  height: v-bind(height);
  position: relative;
}
</style>
