<template>
  <Dialog :open="open" @update:open="(val) => emit('close')">
    <DialogContent class="max-w-5xl max-h-[85vh] overflow-y-auto" aria-describedby="position-details-description">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <div
            class="h-10 w-10 rounded-md bg-primary text-primary-foreground flex items-center justify-center text-lg font-bold"
          >
            {{ getPositionIcon(positionData.position) }}
          </div>
          <div>
            <div>{{ positionData.position }} 位置详情</div>
            <div id="position-details-description" class="text-sm text-muted-foreground font-normal">
              {{ positionData.games }} 场对局 · 胜率 {{ positionData.winRate.toFixed(1) }}%
            </div>
          </div>
        </DialogTitle>
      </DialogHeader>

      <div class="space-y-4 mt-4">
        <!-- Tabs切换 -->
        <Tabs default-value="stats" class="w-full">
          <TabsList class="grid w-full grid-cols-3">
            <TabsTrigger value="stats">数据统计</TabsTrigger>
            <TabsTrigger value="analysis">深度分析</TabsTrigger>
            <TabsTrigger value="suggestions">改进建议</TabsTrigger>
          </TabsList>

          <!-- 数据统计Tab -->
          <TabsContent value="stats" class="space-y-4">
            <!-- 胜率统计 -->
            <Card>
              <CardHeader class="pb-3">
                <CardTitle class="text-base">胜率统计</CardTitle>
              </CardHeader>
              <CardContent>
                <div class="grid grid-cols-3 gap-4">
                  <div class="text-center">
                    <div class="text-2xl font-bold text-green-600 dark:text-green-400">{{ positionData.wins }}</div>
                    <div class="text-xs text-muted-foreground">胜场</div>
                  </div>
                  <div class="text-center">
                    <div class="text-2xl font-bold text-red-600 dark:text-red-400">
                      {{ positionData.games - positionData.wins }}
                    </div>
                    <div class="text-xs text-muted-foreground">负场</div>
                  </div>
                  <div class="text-center">
                    <div
                      :class="[
                        'text-2xl font-bold',
                        positionData.winRate >= 55
                          ? 'text-green-600 dark:text-green-400'
                          : positionData.winRate >= 50
                            ? 'text-blue-600 dark:text-blue-400'
                            : 'text-red-600 dark:text-red-400'
                      ]"
                    >
                      {{ positionData.winRate.toFixed(1) }}%
                    </div>
                    <div class="text-xs text-muted-foreground">胜率</div>
                  </div>
                </div>
              </CardContent>
            </Card>

            <!-- KDA统计 -->
            <Card>
              <CardHeader class="pb-3">
                <CardTitle class="text-base">KDA 数据</CardTitle>
              </CardHeader>
              <CardContent>
                <div class="grid grid-cols-4 gap-3">
                  <div class="bg-muted/50 rounded-lg p-3">
                    <div class="text-xs text-muted-foreground mb-1">KDA</div>
                    <div class="text-lg font-bold">{{ positionData.stats.avgKda.toFixed(2) }}</div>
                  </div>
                  <div class="bg-muted/50 rounded-lg p-3">
                    <div class="text-xs text-muted-foreground mb-1">击杀</div>
                    <div class="text-lg font-bold text-green-600 dark:text-green-400">
                      {{ positionData.stats.avgKills.toFixed(1) }}
                    </div>
                  </div>
                  <div class="bg-muted/50 rounded-lg p-3">
                    <div class="text-xs text-muted-foreground mb-1">死亡</div>
                    <div class="text-lg font-bold text-red-600 dark:text-red-400">
                      {{ positionData.stats.avgDeaths.toFixed(1) }}
                    </div>
                  </div>
                  <div class="bg-muted/50 rounded-lg p-3">
                    <div class="text-xs text-muted-foreground mb-1">助攻</div>
                    <div class="text-lg font-bold text-blue-600 dark:text-blue-400">
                      {{ positionData.stats.avgAssists.toFixed(1) }}
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>

            <!-- 游戏表现 -->
            <Card>
              <CardHeader class="pb-3">
                <CardTitle class="text-base">游戏表现</CardTitle>
              </CardHeader>
              <CardContent>
                <div class="grid grid-cols-2 gap-3">
                  <div class="flex justify-between items-center py-2 px-3 bg-muted/30 rounded">
                    <span class="text-sm text-muted-foreground">补刀/分钟</span>
                    <span class="font-semibold">{{ positionData.stats.cspm.toFixed(1) }}</span>
                  </div>
                  <div class="flex justify-between items-center py-2 px-3 bg-muted/30 rounded">
                    <span class="text-sm text-muted-foreground">视野得分/分钟</span>
                    <span class="font-semibold">{{ positionData.stats.vspm.toFixed(2) }}</span>
                  </div>
                  <div class="flex justify-between items-center py-2 px-3 bg-muted/30 rounded">
                    <span class="text-sm text-muted-foreground">伤害/分钟</span>
                    <span class="font-semibold">{{ positionData.stats.dpm.toFixed(0) }}</span>
                  </div>
                  <div class="flex justify-between items-center py-2 px-3 bg-muted/30 rounded">
                    <span class="text-sm text-muted-foreground">参团率</span>
                    <span class="font-semibold"
                      >{{
                        (
                          (positionData.stats.avgAssists /
                            (positionData.stats.avgKills +
                              positionData.stats.avgDeaths +
                              positionData.stats.avgAssists)) *
                            100 || 0
                        ).toFixed(1)
                      }}%</span
                    >
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          <!-- 深度分析Tab -->
          <TabsContent value="analysis" class="space-y-4">
            <!-- 能力雷达图 -->
            <PositionComparisonChart :position-data="positionData" />

            <!-- 英雄池分析 -->
            <PositionChampionPool :position-data="positionData" />

            <!-- 胜率趋势 -->
            <PositionTrendChart :position-data="positionData" />
          </TabsContent>

          <!-- 改进建议Tab -->
          <TabsContent value="suggestions" class="space-y-4">
            <Card v-if="positionData.stats.advice && positionData.stats.advice.length > 0">
              <CardHeader class="pb-3">
                <CardTitle class="text-base">位置建议</CardTitle>
                <CardDescription>基于该位置的表现生成的针对性建议</CardDescription>
              </CardHeader>
              <CardContent class="space-y-2">
                <div
                  v-for="(advice, idx) in positionData.stats.advice.slice(0, 10)"
                  :key="idx"
                  class="p-3 border rounded-lg"
                >
                  <div class="flex items-start justify-between mb-1">
                    <h4 class="font-semibold text-sm">{{ advice.title }}</h4>
                    <Badge :variant="advice.priority >= 4 ? 'destructive' : 'secondary'" class="text-xs">
                      优先级 {{ advice.priority }}
                    </Badge>
                  </div>
                  <p class="text-xs text-muted-foreground mb-2">{{ advice.problem }}</p>
                  <div v-if="advice.suggestions && advice.suggestions.length > 0" class="space-y-1">
                    <div
                      v-for="(suggestion, sidx) in advice.suggestions"
                      :key="sidx"
                      class="text-xs flex items-start gap-1"
                    >
                      <span class="text-primary">•</span>
                      <span>{{ suggestion }}</span>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
            <Card v-else>
              <CardContent class="py-8 text-center text-muted-foreground">
                <p>暂无建议数据</p>
                <p class="text-xs mt-1">非排位模式不生成详细建议</p>
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import PositionComparisonChart from './PositionComparisonChart.vue'
import PositionTrendChart from './PositionTrendChart.vue'
import PositionChampionPool from './PositionChampionPool.vue'

interface Props {
  open: boolean
  positionData: PositionStats
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
}>()

const getPositionIcon = (position: string): string => {
  const icons: Record<string, string> = {
    上单: '上',
    打野: '野',
    中单: '中',
    ADC: '下',
    辅助: '辅',
    灵活: '灵'
  }
  return icons[position] || '?'
}
</script>
