<template>
  <Card>
    <CardHeader>
      <CardTitle class="text-base">能力雷达图</CardTitle>
      <CardDescription>各维度能力评估（满分10分）</CardDescription>
    </CardHeader>
    <CardContent>
      <v-chart class="w-full h-[350px]" :option="chartOption" :autoresize="true" />
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { RadarChart } from 'echarts/charts'
import { TooltipComponent, LegendComponent } from 'echarts/components'
import VChart from 'vue-echarts'

// 注册必要的组件
use([CanvasRenderer, RadarChart, TooltipComponent, LegendComponent])

interface Props {
  positionData: PositionStats
  allPositions?: PositionStats[]
}

const props = defineProps<Props>()

// 计算能力维度数据
const calculateScore = (value: number, ideal: number, min: number = 0) => {
  if (value >= ideal) return 10
  if (value <= min) return 0
  return Math.min(10, Math.max(0, ((value - min) / (ideal - min)) * 10))
}

const chartOption = computed(() => {
  const stats = props.positionData.stats

  const radarData = [
    calculateScore(stats.avgKda, 4.0, 0), // KDA
    calculateScore(stats.cspm, 8.0, 2.0), // 补刀
    calculateScore(stats.vspm, 2.0, 0.3), // 视野
    calculateScore(stats.killParticipation, 0.7, 0.3), // 参团率
    calculateScore(stats.damageShare, 0.35, 0.15), // 输出占比
    calculateScore(props.positionData.winRate / 100, 0.6, 0.4) // 胜率
  ]

  // 固定颜色
  const primaryColor = '#3b82f6'
  const foregroundColor = '#0f172a'
  const borderColor = '#e2e8f0'
  const mutedColor = '#f1f5f9'
  const bgColor = '#ffffff'

  return {
    tooltip: {
      trigger: 'item',
      formatter: (params: any) => {
        const data = params.data
        const names = ['KDA', '补刀', '视野', '参团', '输出', '胜率']
        const values = data.value
        let html = `<div style="font-weight: bold; margin-bottom: 8px;">${data.name}</div>`
        names.forEach((name, idx) => {
          html += `<div>${name}: ${values[idx].toFixed(1)}/10</div>`
        })
        return html
      }
    },
    radar: {
      indicator: [
        { name: 'KDA', max: 10 },
        { name: '补刀', max: 10 },
        { name: '视野', max: 10 },
        { name: '参团', max: 10 },
        { name: '输出', max: 10 },
        { name: '胜率', max: 10 }
      ],
      radius: '70%',
      splitNumber: 4,
      shape: 'polygon',
      axisName: {
        color: foregroundColor,
        fontSize: 14,
        fontWeight: 500
      },
      splitLine: {
        lineStyle: {
          color: borderColor,
          width: 1
        }
      },
      splitArea: {
        show: true,
        areaStyle: {
          color: ['transparent', 'rgba(241, 245, 249, 0.3)', 'transparent', 'rgba(241, 245, 249, 0.3)']
        }
      },
      axisLine: {
        lineStyle: {
          color: borderColor,
          width: 1.5
        }
      }
    },
    series: [
      {
        type: 'radar',
        emphasis: {
          lineStyle: {
            width: 4
          }
        },
        data: [
          {
            value: radarData,
            name: props.positionData.position,
            areaStyle: {
              color: 'rgba(59, 130, 246, 0.3)'
            },
            lineStyle: {
              color: primaryColor,
              width: 3
            },
            itemStyle: {
              color: primaryColor,
              borderColor: bgColor,
              borderWidth: 2
            },
            symbol: 'circle',
            symbolSize: 8
          }
        ]
      }
    ]
  }
})
</script>
