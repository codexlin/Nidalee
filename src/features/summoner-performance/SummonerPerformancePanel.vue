<template>
  <Card class="p-8">
    <div class="space-y-6">
      <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h3 class="text-lg font-semibold flex flex-wrap items-center gap-2">
            <span class="inline-flex items-center">
              <BarChart class="h-5 w-5 mr-2 text-muted-foreground" />
              近期表现
            </span>
            <Badge v-if="showAiBadge" variant="outline" class="h-5 px-1.5 text-xs font-normal"> AI 已就绪 </Badge>
          </h3>
          <p class="text-sm text-muted-foreground">统一按最近 20 场有效样本分析</p>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <Button
            :disabled="!isConnected || matchHistoryLoading"
            variant="outline"
            size="sm"
            class="h-9"
            @click="$emit('fetch-match-history')"
          >
            <RefreshCw :class="['h-4 w-4 mr-2', { 'animate-spin': matchHistoryLoading }]" />
            {{ matchHistoryLoading ? '加载中...' : '刷新' }}
          </Button>

          <FloatIconButton
            v-if="canExportPoster"
            :title="posterExporting ? '生成中…' : '导出海报'"
            :disabled="posterExporting || !isConnected || matchHistoryLoading"
            @click="$emit('export-poster')"
          >
            <Loader2 v-if="posterExporting" class="h-4 w-4 animate-spin" />
            <ImageDown v-else class="h-4 w-4" />
          </FloatIconButton>
        </div>
      </div>

      <PerformanceScopeTabs :model-value="scope" @update:model-value="emit('scope-change', $event)" />

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

      <div v-else-if="error" class="flex items-center justify-center py-16">
        <div class="max-w-md text-center">
          <CircleAlert class="mx-auto mb-4 h-12 w-12 text-destructive/80" />
          <p class="text-lg font-medium text-foreground">分析暂时不可用</p>
          <p class="mt-1 text-sm leading-relaxed text-muted-foreground">{{ error }}</p>
          <Button class="mt-5" size="sm" variant="outline" @click="$emit('fetch-match-history')">重试</Button>
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

      <!-- 当前筛选下无对局：产品空态，不渲染全 0 指标 -->
      <div v-else-if="isFilterEmpty" class="rounded-xl border border-dashed bg-muted/20 px-6 py-12 text-center">
        <Gamepad2 class="h-10 w-10 mx-auto mb-3 text-muted-foreground/70" />
        <p class="text-base font-medium text-foreground">{{ emptyTitle }}</p>
        <p v-if="emptyDetail" class="mt-1.5 text-sm text-muted-foreground max-w-md mx-auto leading-relaxed">
          {{ emptyDetail }}
        </p>
        <div class="mt-5 flex flex-wrap items-center justify-center gap-2">
          <Button
            size="sm"
            variant="outline"
            :disabled="!isConnected || matchHistoryLoading"
            @click="$emit('fetch-match-history')"
          >
            刷新
          </Button>
        </div>
      </div>

      <!-- 有数据时展示 -->
      <div v-else class="space-y-6">
        <div class="surface-inset px-4 py-3.5">
          <div class="flex flex-wrap items-end gap-x-6 gap-y-3">
            <div>
              <p class="text-xs text-muted-foreground mb-1">胜率</p>
              <p class="text-2xl font-semibold tabular-nums leading-none" :class="winRateToneClass">
                {{ (bucketStatistics?.winRate || 0).toFixed(0) }}%
              </p>
            </div>
            <div class="h-9 w-px bg-border/70 hidden sm:block mb-0.5" />
            <div>
              <p class="text-xs text-muted-foreground mb-1">战绩</p>
              <p class="text-lg font-semibold tabular-nums leading-none">
                <span class="text-green-600 dark:text-green-400">{{ bucketStatistics?.wins || 0 }}</span>
                <span class="text-muted-foreground/70 font-normal mx-1">-</span>
                <span class="text-red-600 dark:text-red-400">{{ bucketStatistics?.losses || 0 }}</span>
                <span class="ml-1.5 text-xs font-normal text-muted-foreground tabular-nums">
                  {{ bucketStatistics?.totalGames || 0 }} 场
                </span>
              </p>
            </div>
            <div class="h-9 w-px bg-border/70 hidden sm:block mb-0.5" />
            <div>
              <p class="text-xs text-muted-foreground mb-1">平均 KDA</p>
              <p class="text-lg font-semibold tabular-nums leading-none text-foreground">
                {{ (bucketStatistics?.avgKda || 0).toFixed(2) }}
              </p>
            </div>
            <div
              v-if="recentResultDots.length"
              class="sm:ml-auto"
              :title="`最近 ${recentResultDots.length} 场胜负（左旧右新）`"
            >
              <p class="text-xs text-muted-foreground mb-1.5">胜负走势</p>
              <div class="flex items-center gap-1">
                <span
                  v-for="(won, idx) in recentResultDots"
                  :key="idx"
                  class="h-2.5 w-2.5 rounded-full"
                  :class="won ? 'bg-emerald-500' : 'bg-rose-500/85'"
                />
              </div>
            </div>
          </div>
          <p v-if="sampleShortfallTip" class="mt-2.5 text-xs text-muted-foreground leading-relaxed">
            {{ sampleShortfallTip }}
          </p>
        </div>

        <div
          v-if="hasIdentitySection || hasFavoriteChampions"
          class="grid grid-cols-1 gap-6"
          :class="hasIdentitySection && hasFavoriteChampions ? 'md:grid-cols-2' : ''"
        >
          <div v-if="hasIdentitySection" class="space-y-4">
            <SummonerTraits
              :analysis-traits="bucketTraits"
              :match-statistics="bucketStatistics"
              :position-stats="bucketPositionStats"
              :main-position="bucketMainPosition"
              :performance-category="scope.category"
            />
          </div>

          <div v-if="hasFavoriteChampions" class="space-y-3">
            <div class="space-y-1">
              <h4 class="text-base font-semibold flex items-center">
                <Star class="h-5 w-5 mr-2 text-muted-foreground" />
                常用英雄
              </h4>
              <p class="text-xs text-muted-foreground">按最近游玩场次排序</p>
            </div>
            <div class="grid grid-cols-3 sm:grid-cols-5 gap-2">
              <div
                v-for="champion in favoriteChampions"
                :key="champion.championId"
                class="surface-inset flex flex-col items-center px-2 py-2.5 transition-colors hover:bg-muted/40"
              >
                <img
                  v-if="champion.championId"
                  :src="getChampionIconUrl(champion.championId)"
                  loading="lazy"
                  alt=""
                  class="h-9 w-9 rounded-full border border-primary/20"
                />
                <p class="text-xs font-medium text-center mt-1 truncate w-full">
                  {{ resolveChampionName(champion.championId, champion.championName) }}
                </p>
                <p
                  class="text-sm font-bold tabular-nums"
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
                <p class="text-xs text-muted-foreground tabular-nums">{{ champion.games }}场</p>
              </div>
            </div>
          </div>
        </div>

        <RecentMatchList
          :games="listGames"
          :show-count="showCount"
          @load-more="loadMore"
          @open-game-detail="emit('open-game-detail', $event)"
        />
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { getChampionIconUrl, resolveChampionName } from '@/lib'
import { BarChart, CircleAlert, Gamepad2, ImageDown, Loader2, RefreshCw, Star, Wifi } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'
import type { PerformanceScope } from '@/common/performanceScope'
import PerformanceScopeTabs from './components/PerformanceScopeTabs.vue'
import { useGameStatsBuckets } from './composables/useGameStatsBuckets'
import RecentMatchList from './components/RecentMatchList.vue'
import SummonerTraits from './components/SummonerTraits.vue'

const props = defineProps<{
  isConnected: boolean
  matchHistoryLoading: boolean
  error?: string | null
  matchStatistics: PlayerMatchStats | null
  /** 统一契约特征（Dashboard 优先传入；仅用于少量过程异常信号） */
  analysisTraits?: DeterministicTrait[] | null
  /** 排位分路：作为召唤师身份特征展示 */
  positionStats?: PositionStats[] | null
  mainPosition?: string | null
  scope: PerformanceScope
  /** 本地 AI 已配置（开启 + Key）且策略允许 */
  aiReady?: boolean
  canExportPoster?: boolean
  posterExporting?: boolean
}>()

const {
  isFilterEmpty,
  hasGames,
  bucketStatistics,
  bucketPositionStats,
  bucketMainPosition,
  bucketTraits,
  sampleShortfallTip,
  listGames,
  showCount,
  loadMore,
  recentResultDots,
  winRateToneClass,
  emptyTitle,
  emptyDetail,
  favoriteChampions,
  hasFavoriteChampions,
  hasIdentitySection
} = useGameStatsBuckets(props)

const showAiBadge = computed(() => hasGames.value && !!props.aiReady)

const emit = defineEmits<{
  (e: 'fetch-match-history'): void
  (e: 'scope-change', scope: PerformanceScope): void
  (e: 'export-poster'): void
  (e: 'open-game-detail', game: MatchPerformance): void
}>()
</script>
