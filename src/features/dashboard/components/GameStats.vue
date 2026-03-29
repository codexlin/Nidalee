<template>
  <Card class="p-6">
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
              <p class="text-xs text-muted-foreground tabular-nums">
                <span class="text-orange-600/70">{{ (matchStatistics?.avgKills || 0).toFixed(1) }}</span>
                <span class="mx-0.5">/</span>
                <span>{{ (matchStatistics?.avgDeaths || 0).toFixed(1) }}</span>
                <span class="mx-0.5">/</span>
                <span class="text-blue-600/70">{{ (matchStatistics?.avgAssists || 0).toFixed(1) }}</span>
                <span class="ml-1 text-muted-foreground">KDA</span>
              </p>
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
                <p class="text-xs font-medium text-center mt-1.5 truncate w-full">{{ getChampionName(champion.championId) }}</p>
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
          <div class="grid gap-3" style="grid-template-columns: repeat(auto-fit, minmax(260px, 1fr))">
            <div
              v-for="game in (matchStatistics?.recentPerformance || []).slice(0, showCount)"
              :key="game.gameCreation"
              class="group relative flex bg-gradient-to-br from-card/80 to-muted/60 rounded-xl shadow-sm cursor-pointer transition-transform duration-150 will-change-transform hover:-translate-y-1 hover:shadow-lg backdrop-blur-sm"
              @click="openGameDetail(game)"
            >
              <!-- 左侧彩色竖条 -->
              <div :class="game.win ? 'bg-green-400' : 'bg-red-400'" class="w-1 rounded-l-xl"></div>
              <div class="flex-1 p-4 flex flex-col">
                <!-- 标题区 -->
                <div class="flex items-center justify-between mb-3">
                  <div class="flex items-center space-x-2">
                    <img
                      v-if="game.championId"
                      :src="getChampionIconUrl(game.championId)"
                      alt=""
                      class="h-8 w-8 rounded-full border-2 border-primary ring-1 ring-primary/20"
                    />
                    <span class="text-base font-semibold text-foreground">{{ getChampionName(game.championId) }}</span>
                  </div>
                  <Badge :variant="game.win ? 'default' : 'destructive'" class="text-xs px-2 py-0.5">
                    {{ game.win ? '胜利' : '失败' }}
                  </Badge>
                </div>
                <!-- KDA区 -->
                <div class="flex items-center justify-between text-sm mb-3">
                  <span class="font-mono font-bold text-lg tabular-nums">
                    <span class="text-red-500">{{ game.kills }}</span>
                    <span class="text-gray-400">/</span>
                    <span class="text-gray-400">{{ game.deaths }}</span>
                    <span class="text-gray-400">/</span>
                    <span class="text-blue-500">{{ game.assists }}</span>
                  </span>
                  <span class="text-muted-foreground tabular-nums">{{ formatGameTime(game.gameDuration) }}</span>
                </div>
                <!-- 只保留一条淡色分割线 -->
                <div class="border-t border-blacl/10 dark:border-white/10 my-2"></div>
                <!-- 底部信息和标签 -->
                <div class="flex items-end justify-between mt-1">
                  <div class="flex flex-col text-xs text-muted-foreground">
                    <div class="flex items-center">
                      <Clock class="w-3 h-3 mr-1" />
                      <span>{{ formatRelativeTime(game.gameCreation) }}</span>
                    </div>
                    <span>{{ getQueueName(game.queueId) }}</span>
                  </div>
                  <div
                    class="ml-2 px-2 py-0.5 rounded-full shadow text-xs font-bold select-none flex items-center gap-1 transition-transform duration-150 group-hover:scale-105 group-hover:shadow-lg"
                    :class="[
                      'bg-gradient-to-r',
                      (game.performanceRating || '').includes('超神') || (game.performanceRating || '').includes('亮眼')
                        ? 'from-green-400 to-green-600 text-white'
                        : '',
                      (game.performanceRating || '').includes('不错') ? 'from-yellow-400 to-yellow-500 text-white' : '',
                      (game.performanceRating || '').includes('需要加油') ? 'from-red-500 to-red-700 text-white' : '',
                      (game.performanceRating || '').includes('五杀') || (game.performanceRating || '').includes('四杀')
                        ? 'from-purple-500 to-purple-700 text-white'
                        : ''
                    ]"
                  >
                    <Award v-if="(game.performanceRating || '').includes('超神')" class="w-3 h-3" />
                    <Star v-else-if="(game.performanceRating || '').includes('亮眼')" class="w-3 h-3" />
                    <Flame
                      v-else-if="
                        (game.performanceRating || '').includes('五杀') ||
                        (game.performanceRating || '').includes('四杀')
                      "
                      class="w-3 h-3"
                    />
                    <Smile v-else-if="(game.performanceRating || '').includes('不错')" class="w-3 h-3" />
                    <Meh v-else-if="(game.performanceRating || '').includes('一般')" class="w-3 h-3" />
                    <AlertCircle v-else-if="(game.performanceRating || '').includes('需要加油')" class="w-3 h-3" />
                    <span>{{ game.performanceRating }}</span>
                  </div>
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
  AlertCircle,
  Award,
  BarChart,
  Calendar,
  Clock,
  Flame,
  Gamepad2,
  Loader2,
  Meh,
  RefreshCw,
  Smile,
  Star,
  Target,
  TrendingUp,
  Trophy,
  Wifi
} from 'lucide-vue-next'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'

const dialogOpen = ref(false)
const selectedGame = ref(null)

const props = defineProps<{
  isConnected: boolean
  matchHistoryLoading: boolean
  matchStatistics: any
  selectedQueueId?: number | null
}>()

const openGameDetail = (game: any) => {
  selectedGame.value = game
  console.log(game)
  dialogOpen.value = true
}
const emit = defineEmits<{
  (e: 'fetch-match-history'): void
  (e: 'open-game-detail', game: any): void
  (e: 'queue-change', queueId: number | null): void
}>()

// 处理队列选择
const handleQueueSelect = (value: any) => {
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
