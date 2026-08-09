<template>
  <Card>
    <CardHeader>
      <CardTitle class="text-base">胜率趋势</CardTitle>
      <CardDescription>近期表现变化（累计胜率 / 移动平均）</CardDescription>
    </CardHeader>
    <CardContent>
      <ThemedChart type="line" :data="chartData" :options="chartOptions" height="220px" :loading="isLoading" />
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import ThemedChart from '@/shared/components/charts/ThemedChart.vue'
import type { PositionStatsWithTrend, WinRateTrendPoint } from '@/shared/utils/chartValidation'
import { themeColor, themeColors } from '@/lib/themeColor'
import type { TooltipItem } from 'chart.js'

interface Props {
  positionData: PositionStatsWithTrend
}

const props = defineProps<Props>()
const isLoading = ref(false)

// Chart.js 折线图数据
const chartData = computed(() => {
  const trendData = props.positionData.winRateTrend || []
  const c = themeColors()

  if (trendData.length === 0) {
    return {
      labels: ['暂无数据'],
      datasets: [
        {
          label: '胜率',
          data: [0],
          borderColor: c.primary,
          backgroundColor: themeColor('--primary', 0.1),
          borderWidth: 3,
          fill: true,
          tension: 0.4,
          pointBackgroundColor: c.primary,
          pointBorderColor: c.background,
          pointBorderWidth: 2,
          pointRadius: 6,
          pointHoverRadius: 8,
          pointHoverBackgroundColor: c.background,
          pointHoverBorderColor: c.primary,
          pointHoverBorderWidth: 3
        }
      ]
    }
  }

  const labels = trendData.map((_: WinRateTrendPoint, index: number) => `第${index + 1}场`)
  const cumulativeData = trendData.map((point: WinRateTrendPoint) => point.cumulativeWinRate)
  const movingAvgData = trendData.map((point: WinRateTrendPoint) => point.movingAvgWinRate)

  return {
    labels,
    datasets: [
      {
        label: '累计胜率',
        data: cumulativeData,
        borderColor: c.primary,
        backgroundColor: themeColor('--primary', 0.1),
        borderWidth: 3,
        fill: true,
        tension: 0.4,
        pointBackgroundColor: c.primary,
        pointBorderColor: c.background,
        pointBorderWidth: 2,
        pointRadius: 6,
        pointHoverRadius: 8,
        pointHoverBackgroundColor: c.background,
        pointHoverBorderColor: c.primary,
        pointHoverBorderWidth: 3
      },
      {
        label: '移动平均',
        data: movingAvgData,
        borderColor: c.secondary,
        backgroundColor: themeColor('--secondary', 0.1),
        borderWidth: 2,
        fill: false,
        tension: 0.4,
        pointBackgroundColor: c.secondary,
        pointBorderColor: c.background,
        pointBorderWidth: 2,
        pointRadius: 4,
        pointHoverRadius: 6,
        pointHoverBackgroundColor: c.background,
        pointHoverBorderColor: c.secondary,
        pointHoverBorderWidth: 3,
        borderDash: [5, 5]
      }
    ]
  }
})

const chartOptions = computed(() => {
  const c = themeColors()
  return {
    responsive: true,
    maintainAspectRatio: false,
    plugins: {
      legend: {
        display: true,
        position: 'top' as const,
        labels: {
          usePointStyle: true,
          padding: 20,
          font: {
            size: 12,
            family: '"HarmonyOS Sans SC", "Microsoft YaHei", sans-serif'
          }
        }
      },
      tooltip: {
        callbacks: {
          label: (context: TooltipItem<'line'>) => {
            const y = context.parsed.y
            return `${context.dataset.label}: ${y === null || y === undefined ? '-' : y.toFixed(1)}%`
          }
        }
      }
    },
    scales: {
      x: {
        grid: {
          display: false
        },
        ticks: {
          maxTicksLimit: 6,
          font: {
            size: 11,
            family: '"HarmonyOS Sans SC", "Microsoft YaHei", sans-serif'
          },
          color: c.mutedForeground
        }
      },
      y: {
        min: 0,
        max: 100,
        ticks: {
          callback: (value: string | number) => `${value}%`,
          font: {
            size: 11,
            family: '"HarmonyOS Sans SC", "Microsoft YaHei", sans-serif'
          },
          color: c.mutedForeground
        },
        grid: {
          color: c.border,
          drawBorder: false
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
    },
    interaction: {
      intersect: false,
      mode: 'index' as const
    }
  }
})
</script>
