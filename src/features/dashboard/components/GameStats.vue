<template>
  <Card
    class="p-8 rounded-2xl shadow-xl bg-gradient-to-br from-white/80 to-muted/60 dark:from-background/80 dark:to-muted/40 border border-border"
  >
    <div class="space-y-6">
      <div class="flex items-center justify-between">
        <div>
          <h3 class="text-lg font-semibold flex items-center">
            <BarChart class="h-5 w-5 mr-2 text-muted-foreground" />
            游戏统计
          </h3>
          <p class="text-sm text-muted-foreground">近期游戏数据概览</p>
        </div>
        <div class="flex items-center gap-2">
          <!-- 队列选择器 -->
          <Select :model-value="selectedQueueId?.toString() || 'all'" @update:model-value="handleQueueSelect">
            <SelectTrigger class="w-[140px] h-9">
              <SelectValue placeholder="选择队列" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">全部模式</SelectItem>
              <SelectItem value="420">单双排位</SelectItem>
              <SelectItem value="440">灵活组排</SelectItem>
              <SelectItem value="450">极地大乱斗</SelectItem>
              <SelectItem value="1700">斗魂竞技场</SelectItem>
              <SelectItem value="900">无限火力</SelectItem>
            </SelectContent>
          </Select>

          <Button
            :disabled="!isConnected || matchHistoryLoading"
            variant="outline"
            size="sm"
            @click="$emit('fetch-match-history')"
          >
            <RefreshCw :class="['h-4 w-4 mr-2', { 'animate-spin': matchHistoryLoading }]" />
            {{ matchHistoryLoading ? '加载中...' : '刷新数据' }}
          </Button>
        </div>
      </div>

      <!-- 加载状态 -->
      <div v-if="matchHistoryLoading" class="flex items-center justify-center py-16">
        <div class="text-center">
          <Loader2 class="h-12 w-12 animate-spin text-blue-500 mx-auto mb-4" />
          <p class="text-lg font-medium text-muted-foreground">正在分析对局数据...</p>
          <p class="text-sm text-muted-foreground">请稍候，这可能需要几秒钟</p>
        </div>
      </div>

      <!-- 未连接状态 -->
      <div v-else-if="!isConnected" class="flex items-center justify-center py-16">
        <div class="text-center">
          <Wifi class="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <p class="text-lg font-medium text-muted-foreground">需要连接到League客户端</p>
          <p class="text-sm text-muted-foreground">连接后即可查看详细的游戏统计</p>
        </div>
      </div>

      <!-- 无数据状态 -->
      <div v-else-if="!matchStatistics" class="flex items-center justify-center py-16">
        <div class="text-center">
          <BarChart class="h-12 w-12 text-muted-foreground mx-auto mb-4" />
          <p class="text-lg font-medium text-muted-foreground">暂无统计数据</p>
          <p class="text-sm text-muted-foreground">点击"刷新数据"获取最新的游戏统计</p>
        </div>
      </div>

      <!-- 统计数据展示 -->
      <div v-else class="space-y-6">
        <!-- 总体数据概览（紧凑水平布局） -->
        <div class="flex items-center justify-between gap-4 p-4 rounded-lg border bg-card/50">
          <!-- 总对局 -->
          <div class="flex items-center gap-3 flex-1">
            <Trophy class="h-5 w-5 text-yellow-500 shrink-0" />
            <div class="min-w-0">
              <p class="text-xl font-bold text-foreground tabular-nums">{{ matchStatistics?.totalGames || 0 }}</p>
              <p class="text-xs text-muted-foreground">总对局</p>
            </div>
          </div>
          <div class="w-px h-10 bg-border" />
          <!-- 胜场 -->
          <div class="flex items-center gap-3 flex-1">
            <Award class="h-5 w-5 text-green-500 shrink-0" />
            <div class="min-w-0">
              <p class="text-xl font-bold text-green-600 dark:text-green-400 tabular-nums">
                {{ matchStatistics?.wins || 0 }}
              </p>
              <p class="text-xs text-muted-foreground">胜场</p>
            </div>
          </div>
          <div class="w-px h-10 bg-border" />
          <!-- 负场 -->
          <div class="flex items-center gap-3 flex-1">
            <Target class="h-5 w-5 text-red-500 shrink-0" />
            <div class="min-w-0">
              <p class="text-xl font-bold text-red-600 dark:text-red-400 tabular-nums">
                {{ matchStatistics?.losses || 0 }}
              </p>
              <p class="text-xs text-muted-foreground">负场</p>
            </div>
          </div>
          <div class="w-px h-10 bg-border" />
          <!-- 胜率 -->
          <div class="flex items-center gap-3 flex-1">
            <TrendingUp class="h-5 w-5 text-blue-500 shrink-0" />
            <div class="min-w-0">
              <p class="text-xl font-bold text-blue-600 dark:text-blue-400 tabular-nums">
                {{ (matchStatistics?.winRate || 0).toFixed(1) }}%
              </p>
              <p class="text-xs text-muted-foreground">胜率</p>
            </div>
          </div>
          <div class="w-px h-10 bg-border" />
          <!-- KDA -->
          <div class="flex items-center gap-3 flex-1">
            <Flame class="h-5 w-5 text-orange-500 shrink-0" />
            <div class="min-w-0">
              <p class="text-xl font-bold text-orange-600 dark:text-orange-400 tabular-nums">
                {{ (matchStatistics?.avgKda || 0).toFixed(2) }}
              </p>
              <p class="text-xs text-muted-foreground">平均KDA</p>
            </div>
          </div>
        </div>

        <!-- 召唤师特征分析 + 常用英雄 -->
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div class="space-y-4">
            <SummonerTraits :match-statistics="matchStatistics" />
          </div>

          <!-- 常用英雄 -->
          <div class="space-y-4">
            <h4 class="font-semibold flex items-center">
              <Star class="h-5 w-5 mr-2 text-muted-foreground" />
              常用英雄
            </h4>
            <div class="grid grid-cols-3 sm:grid-cols-5 gap-2">
              <div
                v-for="champion in (matchStatistics?.favoriteChampions || []).slice(0, 10)"
                :key="champion.championId"
                class="flex flex-col items-center p-3 rounded-lg border hover:bg-muted/50 transition-colors cursor-pointer"
              >
                <img
                  v-if="champion.championId"
                  :src="getChampionIconUrl(champion.championId)"
                  loading="lazy"
                  alt=""
                  class="h-10 w-10 rounded-full border-2 border-primary/20"
                />
                <p class="text-xs font-medium text-center mt-1.5 truncate w-full">
                  {{ getChampionName(champion.championId) }}
                </p>
                <p
                  class="text-sm font-bold tabular-nums mt-0.5"
                  :class="[
                    champion.winRate >= 60
                      ? 'text-green-600 dark:text-green-400'
                      : champion.winRate >= 50
                        ? 'text-yellow-600 dark:text-yellow-400'
                        : 'text-red-600 dark:text-red-400'
                  ]"
                >
                  {{ champion.winRate.toFixed(0) }}%
                </p>
                <p class="text-[10px] text-muted-foreground tabular-nums">{{ champion.games }}场</p>
              </div>
            </div>
          </div>
        </div>

        <!-- 最近对局 -->
        <div class="space-y-4" v-if="matchStatistics?.recentPerformance?.length > 0">
          <h4 class="font-semibold flex items-center">
            <Calendar class="h-5 w-5 mr-2 text-muted-foreground" />
            最近对局
          </h4>
          <div class="grid gap-2" style="grid-template-columns: repeat(auto-fit, minmax(240px, 1fr))">
            <div
              v-for="game in (matchStatistics?.recentPerformance || []).slice(0, showCount)"
              :key="game.gameCreation"
              class="group relative flex bg-gradient-to-br from-card/80 to-muted/60 border rounded-lg cursor-pointer transition-all duration-150 hover:-translate-y-0.5 hover:shadow-md backdrop-blur-sm"
              @click="openGameDetail(game)"
            >
              <!-- 左侧状态条 -->
              <div :class="game.win ? 'bg-green-500' : 'bg-red-500'" class="w-1 rounded-l-lg"></div>
              <div class="flex-1 p-3">
                <!-- 顶部：英雄 + 结果 + 时长 -->
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-2">
                    <img
                      v-if="game.championId"
                      :src="getChampionIconUrl(game.championId)"
                      alt=""
                      class="h-9 w-9 rounded-full border-2 border-primary/20"
                    />
                    <span class="font-semibold text-base">{{ getChampionName(game.championId) }}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span class="text-xs text-muted-foreground tabular-nums">{{
                      formatGameTime(game.gameDuration ?? 0)
                    }}</span>
                    <Badge :variant="game.win ? 'default' : 'destructive'" class="text-xs px-1.5 py-0 h-5">
                      {{ game.win ? '胜' : '负' }}
                    </Badge>
                  </div>
                </div>
                <!-- 中部：KDA + 表现标签 -->
                <div class="flex items-center justify-between">
                  <span class="font-mono font-bold text-base tabular-nums">
                    <span class="text-red-500">{{ game.kills }}</span>
                    <span class="text-gray-500">/</span>
                    <span class="text-gray-500">{{ game.deaths }}</span>
                    <span class="text-gray-500">/</span>
                    <span class="text-blue-500">{{ game.assists }}</span>
                  </span>
                  <div
                    class="px-2 py-0.5 rounded text-xs font-medium"
                    :class="[
                      getPerformanceRating(game).includes('超神') || getPerformanceRating(game).includes('亮眼')
                        ? 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400'
                        : '',
                      getPerformanceRating(game).includes('不错')
                        ? 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400'
                        : '',
                      getPerformanceRating(game).includes('需要加油')
                        ? 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400'
                        : '',
                      getPerformanceRating(game).includes('五杀') || getPerformanceRating(game).includes('四杀')
                        ? 'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400'
                        : ''
                    ]"
                  >
                    {{ getPerformanceRating(game) }}
                  </div>
                </div>
                <!-- 底部：时间 + 模式（单行） -->
                <div class="flex items-center gap-3 mt-2 text-xs text-muted-foreground">
                  <span class="flex items-center gap-1">
                    <Clock class="w-3 h-3" />
                    {{ formatRelativeTime(game.gameCreation ?? 0) }}
                  </span>
                  <span>·</span>
                  <span>{{ getQueueName(game.queueId ?? 0) }}</span>
                </div>
              </div>
            </div>
          </div>
          <div v-if="(matchStatistics?.recentPerformance?.length || 0) > showCount" class="flex justify-center mt-4">
            <Button @click="loadMore" variant="outline" size="sm"> 加载更多 </Button>
          </div>
        </div>
        <div v-else class="text-center text-muted-foreground py-8">
          <Gamepad2 class="h-12 w-12 mx-auto mb-3 opacity-50" />
          <p>暂无对局记录</p>
        </div>
      </div>
    </div>
  </Card>

  <GameDetailDialog v-model:visible="dialogOpen" :selectedGame="selectedGame" />
</template>

<script setup lang="ts">
import { getChampionIconUrl, getChampionName, getQueueName } from '@/lib'
import {
  Award,
  BarChart,
  Calendar,
  Clock,
  Flame,
  Gamepad2,
  Loader2,
  RefreshCw,
  Star,
  Target,
  TrendingUp,
  Trophy,
  Wifi
} from 'lucide-vue-next'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import type { AcceptableValue } from 'reka-ui'

const dialogOpen = ref(false)
const selectedGame = ref<MatchPerformance | null>(null)

const props = defineProps<{
  isConnected: boolean
  matchHistoryLoading: boolean
  matchStatistics: PlayerMatchStats | null
  selectedQueueId?: number | null
}>()

/** MatchPerformance 运行时可能附带表现评级字段 */
const getPerformanceRating = (game: MatchPerformance): string => {
  if ('performanceRating' in game && typeof game.performanceRating === 'string') {
    return game.performanceRating
  }
  return ''
}

const openGameDetail = (game: MatchPerformance) => {
  selectedGame.value = game
  console.log(game)
  dialogOpen.value = true
}
const emit = defineEmits<{
  (e: 'fetch-match-history'): void
  (e: 'open-game-detail', game: MatchPerformance): void
  (e: 'queue-change', queueId: number | null): void
}>()

// 处理队列选择
const handleQueueSelect = (value: AcceptableValue) => {
  if (!value || value === 'all') {
    emit('queue-change', null)
  } else {
    const queueId = typeof value === 'string' ? parseInt(value) : Number(value)
    emit('queue-change', queueId)
  }
}

const { formatGameTime, formatRelativeTime } = useFormatters()

const initialShowCount = 10
const showCount = ref(initialShowCount)
const loadMore = () => {
  showCount.value += 10
}
</script>
