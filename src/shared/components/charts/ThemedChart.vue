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
import { ref, onMounted, onUnmounted, watch, nextTick, computed } from 'vue'
import Button from '@/shared/components/ui/button/index.vue'
import { useSettingsStore } from '@/shared/stores/ui/settingsStore'

interface Props {
  type: 'radar' | 'line' | 'bar' | 'pie'
  data: any
  options?: any
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
const chartInstance = ref<any>(null)

// 获取当前主题颜色
const themeColors = computed(() => {
  const root = document.documentElement
  const computedStyle = getComputedStyle(root)

  return {
    primary: computedStyle.getPropertyValue('--primary').trim() || '#3b82f6',
    primaryForeground: computedStyle.getPropertyValue('--primary-foreground').trim() || '#ffffff',
    background: computedStyle.getPropertyValue('--background').trim() || '#ffffff',
    foreground: computedStyle.getPropertyValue('--foreground').trim() || '#0f172a',
    muted: computedStyle.getPropertyValue('--muted').trim() || '#f1f5f9',
    mutedForeground: computedStyle.getPropertyValue('--muted-foreground').trim() || '#64748b',
    border: computedStyle.getPropertyValue('--border').trim() || '#e2e8f0',
    card: computedStyle.getPropertyValue('--card').trim() || '#ffffff',
    cardForeground: computedStyle.getPropertyValue('--card-foreground').trim() || '#0f172a',
    destructive: computedStyle.getPropertyValue('--destructive').trim() || '#ef4444',
    destructiveForeground: computedStyle.getPropertyValue('--destructive-foreground').trim() || '#ffffff'
  }
})

// 动态导入图表库
const loadChartLibrary = async () => {
  try {
    const { Chart, registerables } = await import('chart.js')
    Chart.register(...registerables)
    return Chart
  } catch (err) {
    console.error('Failed to load chart library:', err)
    throw new Error('图表库加载失败')
  }
}

// 创建图表
const createChart = async () => {
  if (!chartContainer.value) return

  try {
    error.value = null
    const Chart = await loadChartLibrary()

    // 销毁旧图表
    if (chartInstance.value) {
      chartInstance.value.destroy()
    }

    // 获取主题颜色
    const colors = themeColors.value

    // 创建新图表
    chartInstance.value = new Chart(chartContainer.value, {
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
                family: 'Satoshi, "Noto Sans SC", "Microsoft YaHei", sans-serif'
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
              family: 'Satoshi, "Noto Sans SC", "Microsoft YaHei", sans-serif',
              weight: 'bold'
            },
            bodyFont: {
              family: 'Satoshi, "Noto Sans SC", "Microsoft YaHei", sans-serif'
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
                      family: 'Satoshi, "Noto Sans SC", "Microsoft YaHei", sans-serif'
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
                      family: 'Satoshi, "Noto Sans SC", "Microsoft YaHei", sans-serif'
                    }
                  }
                }
              }
            : undefined,
        ...props.options
      }
    })
  } catch (err: any) {
    error.value = err.message || '图表创建失败'
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
