<template>
  <Card>
    <CardHeader>
      <CardTitle class="text-base">胜率趋势</CardTitle>
      <CardDescription>近期表现变化（模拟数据）</CardDescription>
    </CardHeader>
    <CardContent>
      <v-chart class="w-full h-[220px]" :option="chartOption" autoresize />
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, MarkLineComponent } from 'echarts/components'
import VChart from 'vue-echarts'

use([CanvasRenderer, LineChart, GridComponent, TooltipComponent, MarkLineComponent])

interface Props {
  positionData: PositionStats
}

const props = defineProps<Props>()

const chartOption = computed(() => {
  const games = props.positionData.games
  const winRate = props.positionData.winRate

  // 生成模拟的趋势数据
  const trendData: number[] = []
  const categories: string[] = []

  if (games >= 5) {
    // 生成模拟趋势（实际应该从后端获取每场对局的时间和结果）
    const dataPoints = Math.min(games, 15)
    const variance = 20

    for (let i = 0; i < dataPoints; i++) {
      const randomFactor = (Math.random() - 0.5) * variance
      const trend = winRate + randomFactor + (i - dataPoints / 2) * 2 // 添加趋势
      trendData.push(Math.max(0, Math.min(100, trend)))
      categories.push(`第${i + 1}场`)
    }
  } else {
    // 数据不足
    for (let i = 0; i < games; i++) {
      trendData.push(winRate + (Math.random() - 0.5) * 10)
      categories.push(`第${i + 1}场`)
    }
  }

  // 写死颜色
  const primaryColor = '#3b82f6'
  const mutedForegroundColor = '#64748b'
  const borderColor = '#e2e8f0'
  const bgColor = '#ffffff'

  return {
    tooltip: {
      trigger: 'axis',
      formatter: (params: any) => {
        const data = params[0]
        return `${data.name}<br/>胜率: ${data.value.toFixed(1)}%`
      }
    },
    grid: {
      left: '5%',
      right: '5%',
      bottom: '10%',
      top: '15%',
      containLabel: true
    },
    xAxis: {
      type: 'category',
      data: categories,
      boundaryGap: false,
      axisLabel: {
        color: mutedForegroundColor,
        fontSize: 11,
        interval: Math.floor(categories.length / 6) // 自适应显示间隔
      },
      axisLine: {
        lineStyle: {
          color: borderColor
        }
      },
      axisTick: {
        show: false
      }
    },
    yAxis: {
      type: 'value',
      min: 0,
      max: 100,
      axisLabel: {
        color: mutedForegroundColor,
        fontSize: 11,
        formatter: '{value}%'
      },
      splitLine: {
        lineStyle: {
          color: borderColor,
          type: 'dashed'
        }
      }
    },
    series: [
      {
        data: trendData,
        type: 'line',
        smooth: true,
        showSymbol: true,
        symbol: 'circle',
        symbolSize: 6,
        lineStyle: {
          color: primaryColor,
          width: 3
        },
        itemStyle: {
          color: primaryColor,
          borderColor: bgColor,
          borderWidth: 2
        },
        areaStyle: {
          color: {
            type: 'linear',
            x: 0,
            y: 0,
            x2: 0,
            y2: 1,
            colorStops: [
              {
                offset: 0,
                color: `${primaryColor}66`
              },
              {
                offset: 1,
                color: `${primaryColor}0D`
              }
            ]
          }
        },
        markLine: {
          silent: true,
          symbol: 'none',
          lineStyle: {
            type: 'dashed',
            color: `${mutedForegroundColor}80`,
            width: 1
          },
          label: {
            color: mutedForegroundColor,
            fontSize: 10
          },
          data: [{ yAxis: 50, label: { formatter: '50%均线' } }]
        }
      }
    ]
  }
})
</script>
