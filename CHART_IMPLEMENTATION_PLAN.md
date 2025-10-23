# 🎨 战绩分析可视化实现方案

## 📦 **第一步：安装图表库**

```bash
# 在项目根目录执行
pnpm add echarts vue-echarts
```

## 🏗️ **第二步：创建图表组件**

### 1. 创建雷达图组件 (`src/features/match-analysis/components/charts/SkillRadarChart.vue`)

```vue
<template>
  <div ref="chartRef" class="w-full h-[300px]"></div>
</template>

<script setup lang="ts">
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { RadarChart } from 'echarts/charts'
import { TitleComponent, TooltipComponent, LegendComponent } from 'echarts/components'
import VChart from 'vue-echarts'

// 注册必要的组件
use([CanvasRenderer, RadarChart, TitleComponent, TooltipComponent, LegendComponent])

const props = defineProps<{
  skillAssessment?: {
    laning_skill: number
    farming_skill: number
    teamfight_skill: number
    vision_skill: number
    positioning_skill: number
    macro_skill: number
    overall_skill: number
  }
}>()

const chartRef = ref()

const option = computed(() => ({
  tooltip: {
    trigger: 'item',
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    borderColor: '#666',
    textStyle: { color: '#fff' }
  },
  radar: {
    indicator: [
      { name: '对线', max: 10 },
      { name: '补刀', max: 10 },
      { name: '团战', max: 10 },
      { name: '视野', max: 10 },
      { name: '站位', max: 10 },
      { name: '宏观', max: 10 }
    ],
    shape: 'polygon',
    splitNumber: 5,
    name: {
      textStyle: {
        color: 'hsl(var(--foreground))',
        fontSize: 12,
        fontWeight: 'bold'
      }
    },
    splitLine: {
      lineStyle: {
        color: 'hsl(var(--border))'
      }
    },
    splitArea: {
      show: true,
      areaStyle: {
        color: ['rgba(24, 144, 255, 0.05)', 'transparent']
      }
    },
    axisLine: {
      lineStyle: {
        color: 'hsl(var(--border))'
      }
    }
  },
  series: [
    {
      type: 'radar',
      data: [
        {
          value: props.skillAssessment
            ? [
                props.skillAssessment.laning_skill,
                props.skillAssessment.farming_skill,
                props.skillAssessment.teamfight_skill,
                props.skillAssessment.vision_skill,
                props.skillAssessment.positioning_skill,
                props.skillAssessment.macro_skill
              ]
            : [0, 0, 0, 0, 0, 0],
          name: '当前能力',
          areaStyle: {
            color: 'rgba(24, 144, 255, 0.3)'
          },
          lineStyle: {
            color: 'rgb(24, 144, 255)',
            width: 2
          },
          itemStyle: {
            color: 'rgb(24, 144, 255)'
          }
        }
      ]
    }
  ]
}))

onMounted(() => {
  if (chartRef.value) {
    const chart = echarts.init(chartRef.value)
    chart.setOption(option.value)

    // 响应式调整
    window.addEventListener('resize', () => chart.resize())
    onUnmounted(() => {
      window.removeEventListener('resize', () => chart.resize())
      chart.dispose()
    })
  }
})

watch(() => props.skillAssessment, () => {
  if (chartRef.value) {
    const chart = echarts.getInstanceByDom(chartRef.value)
    chart?.setOption(option.value)
  }
}, { deep: true })
</script>
```

### 2. 创建时间线趋势图 (`src/features/match-analysis/components/charts/TimelineTrendChart.vue`)

```vue
<template>
  <div ref="chartRef" class="w-full h-[250px]"></div>
</template>

<script setup lang="ts">
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart } from 'echarts/charts'
import {
  GridComponent,
  TooltipComponent,
  LegendComponent,
  ToolboxComponent
} from 'echarts/components'
import * as echarts from 'echarts/core'

use([CanvasRenderer, LineChart, GridComponent, TooltipComponent, LegendComponent, ToolboxComponent])

const props = defineProps<{
  timelineAnalysis?: {
    early_game: {
      cs_per_minute: number
      gold_per_minute: number
      xp_per_minute: number
    }
    mid_game: {
      cs_per_minute: number
      gold_per_minute: number
      xp_per_minute: number
    }
    late_game: {
      cs_per_minute: number
      gold_per_minute: number
      xp_per_minute: number
    }
  }
}>()

const chartRef = ref()

const option = computed(() => ({
  tooltip: {
    trigger: 'axis',
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    borderColor: '#666',
    textStyle: { color: '#fff' }
  },
  legend: {
    data: ['补刀/分钟', '金币/分钟', '经验/分钟'],
    textStyle: {
      color: 'hsl(var(--foreground))'
    },
    top: 10
  },
  grid: {
    left: '3%',
    right: '4%',
    bottom: '3%',
    top: '15%',
    containLabel: true
  },
  xAxis: {
    type: 'category',
    boundaryGap: false,
    data: ['对线期(0-10分)', '中期(10-20分)', '后期(20+分)'],
    axisLabel: {
      color: 'hsl(var(--muted-foreground))',
      fontSize: 11
    },
    axisLine: {
      lineStyle: {
        color: 'hsl(var(--border))'
      }
    }
  },
  yAxis: {
    type: 'value',
    axisLabel: {
      color: 'hsl(var(--muted-foreground))',
      fontSize: 11
    },
    splitLine: {
      lineStyle: {
        color: 'hsl(var(--border))',
        type: 'dashed'
      }
    }
  },
  series: [
    {
      name: '补刀/分钟',
      type: 'line',
      smooth: true,
      data: props.timelineAnalysis
        ? [
            props.timelineAnalysis.early_game.cs_per_minute,
            props.timelineAnalysis.mid_game.cs_per_minute,
            props.timelineAnalysis.late_game.cs_per_minute
          ]
        : [0, 0, 0],
      itemStyle: { color: '#10b981' },
      areaStyle: {
        color: {
          type: 'linear',
          x: 0,
          y: 0,
          x2: 0,
          y2: 1,
          colorStops: [
            { offset: 0, color: 'rgba(16, 185, 129, 0.3)' },
            { offset: 1, color: 'rgba(16, 185, 129, 0)' }
          ]
        }
      }
    },
    {
      name: '金币/分钟',
      type: 'line',
      smooth: true,
      data: props.timelineAnalysis
        ? [
            props.timelineAnalysis.early_game.gold_per_minute,
            props.timelineAnalysis.mid_game.gold_per_minute,
            props.timelineAnalysis.late_game.gold_per_minute
          ]
        : [0, 0, 0],
      itemStyle: { color: '#f59e0b' }
    },
    {
      name: '经验/分钟',
      type: 'line',
      smooth: true,
      data: props.timelineAnalysis
        ? [
            props.timelineAnalysis.early_game.xp_per_minute,
            props.timelineAnalysis.mid_game.xp_per_minute,
            props.timelineAnalysis.late_game.xp_per_minute
          ]
        : [0, 0, 0],
      itemStyle: { color: '#8b5cf6' }
    }
  ]
}))

// 同上，初始化和响应式逻辑...
</script>
```

### 3. 创建技能评分条组件 (`src/features/match-analysis/components/charts/SkillProgressBar.vue`)

```vue
<template>
  <div class="space-y-3">
    <div v-for="skill in skills" :key="skill.name" class="flex items-center gap-3">
      <span class="text-sm font-medium text-foreground w-16 flex-shrink-0">{{ skill.name }}</span>
      <div class="flex-1 h-6 bg-muted rounded-full overflow-hidden relative">
        <div
          class="h-full transition-all duration-500 ease-out rounded-full"
          :class="getSkillColor(skill.value)"
          :style="{ width: `${(skill.value / 10) * 100}%` }"
        >
          <div class="absolute inset-0 bg-gradient-to-r from-transparent to-white/20"></div>
        </div>
      </div>
      <span class="text-sm font-bold text-foreground w-12 text-right">
        {{ skill.value }}/10
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  skillAssessment?: {
    laning_skill: number
    farming_skill: number
    teamfight_skill: number
    vision_skill: number
    positioning_skill: number
    macro_skill: number
  }
}>()

const skills = computed(() => [
  { name: '对线', value: props.skillAssessment?.laning_skill || 0 },
  { name: '补刀', value: props.skillAssessment?.farming_skill || 0 },
  { name: '团战', value: props.skillAssessment?.teamfight_skill || 0 },
  { name: '视野', value: props.skillAssessment?.vision_skill || 0 },
  { name: '站位', value: props.skillAssessment?.positioning_skill || 0 },
  { name: '宏观', value: props.skillAssessment?.macro_skill || 0 }
])

const getSkillColor = (value: number) => {
  if (value >= 8) return 'bg-green-500'
  if (value >= 6) return 'bg-blue-500'
  if (value >= 4) return 'bg-yellow-500'
  return 'bg-red-500'
}
</script>
```

## 🎯 **第三步：集成到玩家详情弹窗**

修改 `SummonerDetailsDialog.vue`（需要找到这个文件）：

```vue
<template>
  <Dialog :open="open" @update:open="$emit('close')">
    <DialogContent class="max-w-4xl max-h-[90vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle>{{ summoner?.displayName }} - 详细分析</DialogTitle>
      </DialogHeader>

      <Tabs default-value="overview" class="w-full">
        <TabsList>
          <TabsTrigger value="overview">概览</TabsTrigger>
          <TabsTrigger value="skills">能力分析</TabsTrigger>
          <TabsTrigger value="timeline">发育曲线</TabsTrigger>
          <TabsTrigger value="suggestions">改进建议</TabsTrigger>
        </TabsList>

        <!-- 概览标签 -->
        <TabsContent value="overview">
          <div class="grid grid-cols-2 gap-4">
            <!-- 综合评分 -->
            <Card>
              <CardHeader>
                <CardTitle>综合评分</CardTitle>
              </CardHeader>
              <CardContent class="flex items-center justify-center">
                <div class="text-6xl font-bold text-primary">
                  {{ summonerResult?.skillAssessment?.overall_skill || 0 }}/10
                </div>
              </CardContent>
            </Card>

            <!-- 基础数据 -->
            <Card>
              <CardHeader>
                <CardTitle>基础数据</CardTitle>
              </CardHeader>
              <CardContent>
                <div class="space-y-2 text-sm">
                  <div class="flex justify-between">
                    <span>胜率:</span>
                    <span class="font-bold">{{ summonerResult?.winRate?.toFixed(1) }}%</span>
                  </div>
                  <div class="flex justify-between">
                    <span>平均KDA:</span>
                    <span class="font-bold">{{ summonerResult?.avgKda?.toFixed(2) }}</span>
                  </div>
                  <div class="flex justify-between">
                    <span>总场次:</span>
                    <span class="font-bold">{{ summonerResult?.totalGames }}</span>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <!-- 能力分析标签 -->
        <TabsContent value="skills">
          <div class="grid grid-cols-2 gap-4">
            <!-- 雷达图 -->
            <Card>
              <CardHeader>
                <CardTitle>能力雷达图</CardTitle>
              </CardHeader>
              <CardContent>
                <SkillRadarChart :skill-assessment="summonerResult?.skillAssessment" />
              </CardContent>
            </Card>

            <!-- 技能评分条 -->
            <Card>
              <CardHeader>
                <CardTitle>详细评分</CardTitle>
              </CardHeader>
              <CardContent>
                <SkillProgressBar :skill-assessment="summonerResult?.skillAssessment" />
              </CardContent>
            </Card>
          </div>
        </TabsContent>

        <!-- 发育曲线标签 -->
        <TabsContent value="timeline">
          <Card>
            <CardHeader>
              <CardTitle>游戏阶段发育曲线</CardTitle>
            </CardHeader>
            <CardContent>
              <TimelineTrendChart :timeline-analysis="summonerResult?.timelineAnalysis" />
            </CardContent>
          </Card>
        </TabsContent>

        <!-- 改进建议标签 -->
        <TabsContent value="suggestions">
          <div class="space-y-3">
            <Card
              v-for="(suggestion, index) in summonerResult?.improvementSuggestions"
              :key="index"
            >
              <CardHeader>
                <CardTitle class="text-base">
                  <Badge :variant="getPriorityVariant(suggestion.priority)">
                    优先级 {{ suggestion.priority }}
                  </Badge>
                  {{ suggestion.title }}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <p class="text-sm text-muted-foreground mb-2">{{ suggestion.description }}</p>
                <div class="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <span class="font-semibold">当前:</span>
                    {{ suggestion.current_performance }}
                  </div>
                  <div>
                    <span class="font-semibold">目标:</span>
                    {{ suggestion.target_performance }}
                  </div>
                </div>
                <div class="mt-3">
                  <p class="font-semibold text-sm mb-1">具体行动:</p>
                  <ul class="list-disc list-inside text-sm text-muted-foreground space-y-1">
                    <li v-for="(action, idx) in suggestion.specific_actions" :key="idx">
                      {{ action }}
                    </li>
                  </ul>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import SkillRadarChart from '../charts/SkillRadarChart.vue'
import TimelineTrendChart from '../charts/TimelineTrendChart.vue'
import SkillProgressBar from '../charts/SkillProgressBar.vue'

const props = defineProps<{
  open: boolean
  summoner: any
  summonerResult: any
  loading: boolean
}>()

const emit = defineEmits<{
  close: []
}>()

const getPriorityVariant = (priority: number) => {
  if (priority >= 4) return 'destructive'
  if (priority >= 3) return 'default'
  return 'secondary'
}
</script>
```

## 📝 **第四步：更新TODO**

已完成：
- [x] 分析前端代码结构
- [x] 选择合适的图表库（vue-echarts）
- [x] 设计雷达图组件
- [x] 设计时间线趋势图组件
- [x] 设计技能评分条组件
- [x] 设计集成方案

待执行：
- [ ] 安装 echarts 和 vue-echarts
- [ ] 创建图表组件文件
- [ ] 修改玩家详情弹窗
- [ ] 测试可视化效果

