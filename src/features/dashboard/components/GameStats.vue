<template>
  <Card class="p-8">
    <div class="space-y-6">
      <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <h3 class="text-lg font-semibold flex flex-wrap items-center gap-2">
            <span class="inline-flex items-center">
              <BarChart class="h-5 w-5 mr-2 text-muted-foreground" />
              游戏统计
            </span>
            <Badge v-if="showAiBadge" variant="outline" class="h-5 px-1.5 text-xs font-normal"> AI 已就绪 </Badge>
          </h3>
          <p class="text-sm text-muted-foreground">近期游戏数据概览</p>
        </div>
        <div class="flex flex-wrap items-center gap-2">
          <Select v-if="selectedMatchMode" :model-value="selectedMatchMode" @update:model-value="handleModeSelect">
            <SelectTrigger class="w-[180px] h-9">
              <SelectValue placeholder="选择模式" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="option in matchModeOptions" :key="option.key" :value="option.key">
                {{ getMatchModeLabel(option.key) }}
              </SelectItem>
            </SelectContent>
          </Select>

          <Select
            v-if="matchCount !== null && matchCount !== undefined"
            :model-value="String(matchCount)"
            @update:model-value="handleCountSelect"
          >
            <SelectTrigger class="w-[100px] h-9">
              <SelectValue placeholder="场数" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="count in matchCountOptions" :key="count" :value="String(count)">
                {{ count }} 场
              </SelectItem>
            </SelectContent>
          </Select>

          <label
            v-if="showRememberOption"
            for="remember-match-preferences"
            class="inline-flex h-9 items-center gap-2 rounded-md border border-border bg-background px-2.5 text-sm cursor-pointer select-none"
            title="下次启动自动恢复模式与场数"
          >
            <Checkbox
              id="remember-match-preferences"
              :checked="rememberPreferences"
              @update:checked="(checked) => emit('remember-change', checked)"
            />
            <span class="text-muted-foreground whitespace-nowrap">记住选择</span>
          </label>

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

      <!-- 当前筛选下无对局：产品空态，不渲染全 0 指标 -->
      <div v-else-if="isFilterEmpty" class="rounded-xl border border-dashed bg-muted/20 px-6 py-12 text-center">
        <Gamepad2 class="h-10 w-10 mx-auto mb-3 text-muted-foreground/70" />
        <p class="text-base font-medium text-foreground">{{ emptyTitle }}</p>
        <p v-if="emptyDetail" class="mt-1.5 text-sm text-muted-foreground max-w-md mx-auto leading-relaxed">
          {{ emptyDetail }}
        </p>
        <div class="mt-5 flex flex-wrap items-center justify-center gap-2">
          <Button v-if="selectedMatchMode && selectedMatchMode !== 'all'" size="sm" @click="emit('mode-change', 'all')">
            切换到全部模式
          </Button>
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
        <div v-if="showBucketTabs" class="surface-chip inline-flex items-center gap-1 p-1">
          <button
            v-for="tab in bucketTabOptions"
            :key="tab.key"
            type="button"
            class="rounded-lg px-3 py-1.5 text-sm font-medium outline-none transition-colors focus-visible:ring-ring/50 focus-visible:ring-[3px]"
            :class="
              activeBucket === tab.key
                ? 'bg-primary/15 text-primary'
                : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
            "
            @click="activeBucket = tab.key"
          >
            {{ tab.label }}
            <span class="ml-1 tabular-nums text-xs opacity-80">{{ tab.games }}</span>
          </button>
        </div>

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
              :filter-mode="bucketFilterMode"
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
          @open-game-detail="openGameDetail"
        />
      </div>
    </div>
  </Card>

  <!-- 放在加载/断线/空态分支外，避免刷新或断线卸载列表时关掉详情 -->
  <GameDetailDialog v-model:visible="dialogOpen" :selectedGame="selectedGame" />
</template>

<script setup lang="ts">
import { getChampionIconUrl, resolveChampionName } from '@/lib'
import { BarChart, Gamepad2, ImageDown, Loader2, RefreshCw, Star, Wifi } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import type { AcceptableValue } from 'reka-ui'
import { MATCH_MODE_OPTIONS, getMatchModeLabel, isMatchModeKey, type MatchModeKey } from '@/common/queueCatalog'
import { useGameStatsBuckets } from '../composables/useGameStatsBuckets'
import GameDetailDialog from './detail/GameDetailDialog.vue'
import RecentMatchList from './RecentMatchList.vue'

const matchModeOptions = MATCH_MODE_OPTIONS
const matchCountOptions = [20, 25, 30] as const

const props = defineProps<{
  isConnected: boolean
  matchHistoryLoading: boolean
  matchStatistics: PlayerMatchStats | null
  /** 排位桶（420/440）；全部模式优先用此驱动 KPI */
  rankedStats?: PlayerMatchStats | null
  /** 非排位桶 */
  otherStats?: PlayerMatchStats | null
  /** 统一契约特征（Dashboard 优先传入；仅用于少量过程异常信号） */
  analysisTraits?: DeterministicTrait[] | null
  /** 排位分路：作为召唤师身份特征展示 */
  positionStats?: PositionStats[] | null
  mainPosition?: string | null
  /** 仪表盘传入时显示模式切换；搜索页可省略 */
  selectedMatchMode?: MatchModeKey
  /** 仪表盘传入时显示场数切换 */
  matchCount?: number
  /** 是否记住模式/场数 */
  rememberPreferences?: boolean
  /** 扫描过的历史场数（空态说明用） */
  scannedGames?: number | null
  /** 本地 AI 已配置（开启 + Key）且策略允许 */
  aiReady?: boolean
  /** 近况/展示场数（本页统计覆盖，含分路与常用英雄） */
  displayGames?: number | null
  canExportPoster?: boolean
  posterExporting?: boolean
}>()

const showRememberOption = computed(() => props.selectedMatchMode !== undefined && props.matchCount !== undefined)

const {
  isFilterEmpty,
  hasGames,
  showBucketTabs,
  bucketTabOptions,
  activeBucket,
  bucketStatistics,
  bucketFilterMode,
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

const dialogOpen = ref(false)
const selectedGame = ref<MatchPerformance | null>(null)

function openGameDetail(game: MatchPerformance) {
  selectedGame.value = game
  dialogOpen.value = true
}

const emit = defineEmits<{
  (e: 'fetch-match-history'): void
  (e: 'mode-change', mode: MatchModeKey): void
  (e: 'count-change', count: number): void
  (e: 'remember-change', enabled: boolean): void
  (e: 'export-poster'): void
}>()

const handleModeSelect = (value: AcceptableValue) => {
  const key = String(value ?? 'all')
  if (isMatchModeKey(key)) {
    emit('mode-change', key)
  }
}

const handleCountSelect = (value: AcceptableValue) => {
  const count = Number(value)
  if (Number.isFinite(count)) {
    emit('count-change', count)
  }
}
</script>
