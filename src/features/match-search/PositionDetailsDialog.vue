<template>
  <Dialog :open="open" @update:open="() => emit('close')">
    <DialogContent class="max-w-5xl max-h-[85vh] overflow-y-auto" aria-describedby="position-details-description">
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2">
          <div
            class="h-10 w-10 rounded-md bg-primary/15 text-primary flex items-center justify-center text-lg font-bold overflow-hidden"
          >
            <img
              v-if="getRoleIconUrl(positionData.position)"
              :src="getRoleIconUrl(positionData.position)"
              :alt="getPositionLabel(positionData.position)"
              class="h-6 w-6 object-contain"
            />
            <template v-else>{{ getPositionLabel(positionData.position).slice(0, 1) }}</template>
          </div>
          <div>
            <div>{{ getPositionLabel(positionData.position) }} 位置详情</div>
            <div id="position-details-description" class="text-sm text-muted-foreground font-normal">
              {{ positionData.games }} 场对局 · 胜率 {{ positionData.winRate.toFixed(1) }}%
            </div>
          </div>
        </DialogTitle>
      </DialogHeader>

      <div class="space-y-4 mt-4">
        <Tabs default-value="analysis" class="w-full">
          <TabsList class="grid w-full grid-cols-3">
            <TabsTrigger value="analysis">深度分析</TabsTrigger>
            <TabsTrigger value="stats">数据统计</TabsTrigger>
            <TabsTrigger value="suggestions">改进建议</TabsTrigger>
          </TabsList>

          <!-- 过程复盘 -->
          <TabsContent value="analysis" class="space-y-4">
            <Card v-if="insight?.degradationMessage">
              <CardContent class="py-4 text-sm text-muted-foreground">
                {{ insight.degradationMessage }}
              </CardContent>
            </Card>

            <Card v-if="!insight">
              <CardContent class="py-8 text-center text-muted-foreground text-sm">
                暂无过程复盘数据。请使用深度分析并打开时间线后刷新。
              </CardContent>
            </Card>

            <template v-if="insight?.hasTimeline">
              <Card v-if="insight.deathBreakdown">
                <CardHeader class="pb-2">
                  <CardTitle class="text-base">阵亡复盘</CardTitle>
                  <CardDescription>看怎么死的，而不是死了几次</CardDescription>
                </CardHeader>
                <CardContent class="space-y-3">
                  <div class="grid grid-cols-3 gap-2 text-center">
                    <div class="rounded-lg bg-muted/40 px-2 py-2">
                      <div class="text-lg font-semibold">{{ insight.deathBreakdown.solo }}</div>
                      <div class="text-xs text-muted-foreground">被单杀</div>
                    </div>
                    <div class="rounded-lg bg-muted/40 px-2 py-2">
                      <div class="text-lg font-semibold">{{ insight.deathBreakdown.gankOrMulti }}</div>
                      <div class="text-xs text-muted-foreground">多人集火</div>
                    </div>
                    <div class="rounded-lg bg-muted/40 px-2 py-2">
                      <div class="text-lg font-semibold">{{ insight.deathBreakdown.towerOrMinion }}</div>
                      <div class="text-xs text-muted-foreground">塔刀/处决</div>
                    </div>
                  </div>
                  <p class="text-sm leading-relaxed">{{ insight.deathBreakdown.summary }}</p>
                </CardContent>
              </Card>

              <Card v-if="insight.laningProcess">
                <CardHeader class="pb-2">
                  <CardTitle class="text-base">对线过程</CardTitle>
                  <CardDescription>前 10 分钟相对对位</CardDescription>
                </CardHeader>
                <CardContent class="space-y-2">
                  <div class="flex flex-wrap gap-3 text-xs text-muted-foreground">
                    <span>补刀差 {{ formatSigned(insight.laningProcess.avgCsDiff) }}</span>
                    <span>经济差 {{ formatSigned(insight.laningProcess.avgGoldDiff) }}</span>
                    <span>综合 {{ formatSigned(insight.laningProcess.avgOverallAdvantagePct) }}%</span>
                  </div>
                  <p class="text-sm leading-relaxed">{{ insight.laningProcess.summary }}</p>
                </CardContent>
              </Card>

              <Card v-if="insight.objectiveProcess">
                <CardHeader class="pb-2">
                  <CardTitle class="text-base">资源过程</CardTitle>
                  <CardDescription>别人拿资源时你在干嘛</CardDescription>
                </CardHeader>
                <CardContent class="space-y-3">
                  <div class="grid grid-cols-3 gap-2 text-xs">
                    <div class="rounded-lg border px-2 py-2">
                      <div class="text-muted-foreground">小龙</div>
                      <div class="font-medium">
                        到场 {{ insight.objectiveProcess.dragonsTaken }}/{{ insight.objectiveProcess.dragonsSeen }}
                      </div>
                      <div class="text-muted-foreground">错过 {{ insight.objectiveProcess.dragonsMissed }}</div>
                    </div>
                    <div class="rounded-lg border px-2 py-2">
                      <div class="text-muted-foreground">先锋</div>
                      <div class="font-medium">
                        到场 {{ insight.objectiveProcess.heraldsTaken }}/{{ insight.objectiveProcess.heraldsSeen }}
                      </div>
                      <div class="text-muted-foreground">错过 {{ insight.objectiveProcess.heraldsMissed }}</div>
                    </div>
                    <div class="rounded-lg border px-2 py-2">
                      <div class="text-muted-foreground">大龙</div>
                      <div class="font-medium">
                        到场 {{ insight.objectiveProcess.baronsTaken }}/{{ insight.objectiveProcess.baronsSeen }}
                      </div>
                      <div class="text-muted-foreground">错过 {{ insight.objectiveProcess.baronsMissed }}</div>
                    </div>
                  </div>
                  <div v-if="insight.objectiveProcess.missedActivity?.length" class="flex flex-wrap gap-1.5">
                    <Badge
                      v-for="bucket in insight.objectiveProcess.missedActivity"
                      :key="bucket.activity"
                      variant="outline"
                      class="text-xs"
                    >
                      {{ bucket.label }} {{ bucket.count }}
                    </Badge>
                  </div>
                  <p class="text-sm leading-relaxed">{{ insight.objectiveProcess.summary }}</p>
                </CardContent>
              </Card>

              <Card v-if="insight.visionProcess">
                <CardHeader class="pb-2">
                  <CardTitle class="text-base">视野过程</CardTitle>
                </CardHeader>
                <CardContent>
                  <p class="text-sm leading-relaxed">{{ insight.visionProcess.summary }}</p>
                </CardContent>
              </Card>

              <Card
                v-if="
                  !insight.deathBreakdown &&
                  !insight.laningProcess &&
                  !insight.objectiveProcess &&
                  !insight.visionProcess
                "
              >
                <CardContent class="py-8 text-center text-muted-foreground text-sm">
                  有时间线，但还凑不出足够的过程事件。多打几场排位后再看。
                </CardContent>
              </Card>
            </template>
          </TabsContent>

          <!-- 结果附录 -->
          <TabsContent value="stats" class="space-y-4">
            <Card>
              <CardHeader class="pb-3">
                <CardTitle class="text-base">胜率统计</CardTitle>
                <CardDescription>结果附录，不是过程结论</CardDescription>
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

            <Card>
              <CardHeader class="pb-3">
                <CardTitle class="text-base">游戏表现</CardTitle>
              </CardHeader>
              <CardContent>
                <div class="grid grid-cols-2 gap-3">
                  <div class="flex justify-between items-center py-2 px-3 bg-muted/30 rounded">
                    <span class="text-sm text-muted-foreground">
                      {{ positionData.position === 'SUPPORT' ? '线刀+野刀/分钟' : '补刀/分钟（含野刀）' }}
                    </span>
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

            <PositionChampionPool v-if="analysisSettings.displayFeatures.championPool" :position-data="positionData" />
            <PositionTrendChart v-if="analysisSettings.displayFeatures.trendCharts" :position-data="positionData" />
            <PositionComparisonChart
              v-if="analysisSettings.displayFeatures.positionComparison"
              :position-data="positionData"
            />
          </TabsContent>

          <TabsContent value="suggestions" class="space-y-4">
            <Card v-if="processActions.length">
              <CardHeader class="pb-3">
                <CardTitle class="text-base">过程建议</CardTitle>
                <CardDescription>由阵亡 / 对线 / 资源过程推出来的练法</CardDescription>
              </CardHeader>
              <CardContent class="space-y-2">
                <div v-for="action in processActions" :key="action.key" class="p-3 border rounded-lg space-y-1">
                  <div class="flex items-start justify-between gap-2">
                    <h4 class="font-semibold text-sm">{{ action.title }}</h4>
                    <Badge :variant="action.priority >= 8 ? 'destructive' : 'secondary'" class="text-xs">
                      优先 {{ action.priority }}
                    </Badge>
                  </div>
                  <p class="text-xs text-muted-foreground leading-relaxed">{{ action.detail }}</p>
                </div>
              </CardContent>
            </Card>

            <Card
              v-if="
                analysisSettings.displayFeatures.detailedAdvice &&
                positionData.stats.advice &&
                positionData.stats.advice.length > 0
              "
            >
              <CardHeader class="pb-3">
                <CardTitle class="text-base">位置建议</CardTitle>
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

            <Card v-if="!processActions.length && !(positionData.stats.advice && positionData.stats.advice.length)">
              <CardContent class="py-8 text-center text-muted-foreground">
                <p v-if="!insight?.hasTimeline">深度样本不足或未拉到时间线，暂不做过程建议</p>
                <p v-else>暂无足够的负向过程信号</p>
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
import { getPositionLabel } from '@/common/positionLabels'
import { getRoleIconUrl } from '@/lib'
import { useAnalysisSettingsStore } from '@/shared/stores/features/analysisSettingsStore'

interface Props {
  open: boolean
  positionData: PositionStats
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
}>()

const analysisSettings = useAnalysisSettingsStore()

const insight = computed(() => props.positionData.processInsight ?? null)
const processActions = computed(() => insight.value?.actions ?? [])

const formatSigned = (value: number) => (value > 0 ? `+${value.toFixed(1)}` : value.toFixed(1))
</script>
