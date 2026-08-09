<template>
  <Card>
    <CardHeader>
      <CardTitle class="text-base">能力雷达图</CardTitle>
      <CardDescription>各维度能力评估（满分10分）</CardDescription>
    </CardHeader>
    <CardContent>
      <div v-if="!dataValidation.isValid" class="flex items-center justify-center h-[350px] text-red-500">
        <div class="text-center">
          <div class="text-lg font-semibold mb-2">数据验证失败</div>
          <div class="text-sm whitespace-pre-line">{{ errorMessage }}</div>
        </div>
      </div>
      <ThemedChart v-else type="radar" :data="chartData" :options="chartOptions" height="350px" :loading="isLoading" />
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import ThemedChart from '@/shared/components/charts/ThemedChart.vue'
import { validateRadarData, generateChartErrorMessage } from '@/shared/utils/chartValidation'
import { getPositionLabel } from '@/common/positionLabels'
import type { TooltipItem } from 'chart.js'

interface Props {
  positionData: PositionStats
}

const props = defineProps<Props>()
const isLoading = ref(false)

const RADAR_LABELS = ['KDA', '补刀', '视野', 'KP', 'DPM', '胜率'] as const

const dataValidation = computed(() => {
  return validateRadarData(props.positionData.stats)
})

const errorMessage = computed(() => {
  return generateChartErrorMessage(dataValidation.value)
})

const calculateScore = (value: number, ideal: number, min: number = 0) => {
  if (value >= ideal) return 10
  if (value <= min) return 0
  return Math.min(10, Math.max(0, ((value - min) / (ideal - min)) * 10))
}

/**
 * KP（参团率代理）：(K+A)/(K+D+A)
 * 战绩列表不含全队击杀，无法算真实 team KP；用战斗参与度作为可核对的 KP 维度。
 * DPM：每分钟对英雄伤害，不再伪装成「伤害占比」。
 */
const chartData = computed(() => {
  const stats = props.positionData.stats
  const involvement =
    stats.avgKills + stats.avgDeaths + stats.avgAssists > 0
      ? (stats.avgKills + stats.avgAssists) / (stats.avgKills + stats.avgDeaths + stats.avgAssists)
      : 0

  const radarData = [
    calculateScore(stats.avgKda, 4.0, 0),
    calculateScore(stats.cspm, 8.0, 2.0),
    calculateScore(stats.vspm, 2.0, 0.3),
    calculateScore(involvement, 0.7, 0.35),
    calculateScore(stats.dpm, 700, 200),
    calculateScore(props.positionData.winRate / 100, 0.6, 0.4)
  ]

  return {
    labels: [...RADAR_LABELS],
    datasets: [
      {
        label: getPositionLabel(props.positionData.position),
        data: radarData,
        backgroundColor: 'hsl(var(--primary) / 0.2)',
        borderColor: 'hsl(var(--primary))',
        borderWidth: 3,
        pointBackgroundColor: 'hsl(var(--primary))',
        pointBorderColor: 'hsl(var(--background))',
        pointHoverBackgroundColor: 'hsl(var(--background))',
        pointHoverBorderColor: 'hsl(var(--primary))',
        pointRadius: 6,
        pointHoverRadius: 8
      }
    ]
  }
})

const chartOptions = computed(() => ({
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      display: false
    },
    tooltip: {
      callbacks: {
        label: (context: TooltipItem<'radar'>) => {
          return `${RADAR_LABELS[context.dataIndex]}: ${context.parsed.r.toFixed(1)}/10`
        }
      }
    }
  },
  scales: {
    r: {
      beginAtZero: true,
      min: 0,
      max: 10,
      stepSize: 2,
      grid: {
        color: 'hsl(var(--border))'
      },
      pointLabels: {
        font: {
          size: 12,
          weight: 'bold' as const,
          family: 'Satoshi, "Noto Sans SC", "Microsoft YaHei", sans-serif'
        },
        color: 'hsl(var(--foreground))'
      },
      ticks: {
        font: {
          size: 10,
          family: 'Satoshi, "Noto Sans SC", "Microsoft YaHei", sans-serif'
        },
        color: 'hsl(var(--muted-foreground))'
      }
    }
  },
  elements: {
    line: {
      borderWidth: 3
    },
    point: {
      radius: 6,
      hoverRadius: 8
    }
  }
}))
</script>
