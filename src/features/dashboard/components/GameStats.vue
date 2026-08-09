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
        <div class="surface-inset px-4 py-3.5">
          <div class="flex flex-wrap items-end gap-x-6 gap-y-3">
            <div>
              <p class="text-xs text-muted-foreground mb-1">胜率</p>
              <p class="text-2xl font-semibold tabular-nums leading-none" :class="winRateToneClass">
                {{ (matchStatistics?.winRate || 0).toFixed(0) }}%
              </p>
            </div>
            <div class="h-9 w-px bg-border/70 hidden sm:block mb-0.5" />
            <div>
              <p class="text-xs text-muted-foreground mb-1">战绩</p>
              <p class="text-lg font-semibold tabular-nums leading-none">
                <span class="text-green-600 dark:text-green-400">{{ matchStatistics?.wins || 0 }}</span>
                <span class="text-muted-foreground/70 font-normal mx-1">-</span>
                <span class="text-red-600 dark:text-red-400">{{ matchStatistics?.losses || 0 }}</span>
                <span class="ml-1.5 text-xs font-normal text-muted-foreground tabular-nums">
                  {{ matchStatistics?.totalGames || 0 }} 场
                </span>
              </p>
            </div>
            <div class="h-9 w-px bg-border/70 hidden sm:block mb-0.5" />
            <div>
              <p class="text-xs text-muted-foreground mb-1">平均 KDA</p>
              <p class="text-lg font-semibold tabular-nums leading-none text-foreground">
                {{ (matchStatistics?.avgKda || 0).toFixed(2) }}
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
              :analysis-traits="analysisTraits"
              :match-statistics="matchStatistics"
              :position-stats="positionStats"
              :main-position="mainPosition"
              :filter-mode="selectedMatchMode"
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
                  {{ getChampionName(champion.championId) }}
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

        <div v-if="matchStatistics?.recentPerformance?.length" class="space-y-4">
          <div class="space-y-1">
            <h4 class="text-base font-semibold flex items-center">
              <Calendar class="h-5 w-5 mr-2 text-muted-foreground" />
              最近对局
            </h4>
            <p class="text-xs text-muted-foreground">右下角为自研评级（S+～D）。点击查看详情。</p>
          </div>
          <div class="grid gap-2" style="grid-template-columns: repeat(auto-fit, minmax(240px, 1fr))">
            <div
              v-for="game in (matchStatistics?.recentPerformance || []).slice(0, showCount)"
              :key="game.gameCreation"
              class="surface-inset-interactive group relative flex cursor-pointer overflow-hidden"
              @click="openGameDetail(game)"
            >
              <div :class="game.win ? 'bg-emerald-600' : 'bg-rose-600'" class="w-1 shrink-0"></div>
              <!-- 大号衬底评级字 -->
              <span
                class="pointer-events-none absolute -right-1 -bottom-2 z-0 select-none font-black leading-none tabular-nums -rotate-12 origin-bottom-right"
                :class="[gradeWatermarkSizeClass(displayGrade(game)), gradeWatermarkClass(displayGrade(game))]"
                aria-hidden="true"
              >
                {{ displayGrade(game) }}
              </span>
              <div class="relative z-10 flex-1 p-3">
                <div class="flex items-center justify-between mb-2">
                  <div class="flex items-center gap-2 min-w-0">
                    <img
                      v-if="game.championId"
                      :src="getChampionIconUrl(game.championId)"
                      alt=""
                      class="h-9 w-9 shrink-0 rounded-full border-2 border-primary/20"
                    />
                    <span class="font-semibold text-sm truncate">{{ getChampionName(game.championId) }}</span>
                  </div>
                  <div class="flex items-center gap-2 shrink-0">
                    <span class="flex items-center gap-1 text-xs text-muted-foreground tabular-nums">
                      <Timer class="w-3 h-3 shrink-0" />
                      {{ formatGameTime(game.gameDuration ?? 0) }}
                    </span>
                    <span
                      class="h-5 px-1.5 inline-flex items-center rounded-md text-xs font-medium text-white"
                      :class="game.win ? 'bg-emerald-600' : 'bg-rose-600'"
                    >
                      {{ game.win ? '胜' : '负' }}
                    </span>
                  </div>
                </div>
                <div class="pr-6">
                  <span class="font-mono font-bold text-base tabular-nums leading-none">
                    <span class="text-red-500">{{ game.kills }}</span>
                    <span class="text-muted-foreground/50">/</span>
                    <span class="text-muted-foreground">{{ game.deaths }}</span>
                    <span class="text-muted-foreground/50">/</span>
                    <span class="text-blue-500">{{ game.assists }}</span>
                  </span>
                  <div class="flex items-center gap-2 mt-2 text-xs text-muted-foreground">
                    <span class="flex items-center gap-1">
                      <Clock class="w-3 h-3 shrink-0" />
                      {{ formatRelativeTime(game.gameCreation ?? 0) }}
                    </span>
                    <span class="text-border">·</span>
                    <span class="truncate">{{ getQueueName(game.queueId ?? 0) }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div v-if="(matchStatistics?.recentPerformance?.length || 0) > showCount" class="flex justify-center mt-4">
            <FloatIconButton variant="pill" title="加载更多" @click="loadMore"> 加载更多 </FloatIconButton>
          </div>
        </div>
      </div>
    </div>
  </Card>

  <GameDetailDialog v-model:visible="dialogOpen" :selectedGame="selectedGame" />
</template>

<script setup lang="ts">
import { getChampionIconUrl, getChampionName, getQueueName } from '@/lib'
import { BarChart, Calendar, Clock, Gamepad2, ImageDown, Loader2, RefreshCw, Star, Timer, Wifi } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Checkbox } from '@/components/ui/checkbox'
import type { AcceptableValue } from 'reka-ui'
import { MATCH_MODE_OPTIONS, getMatchModeLabel, isMatchModeKey, type MatchModeKey } from '@/common/queueCatalog'
import { displayGrade, gradeWatermarkClass, gradeWatermarkSizeClass } from '../utils/matchGrade'

const dialogOpen = ref(false)
const selectedGame = ref<MatchPerformance | null>(null)
const matchModeOptions = MATCH_MODE_OPTIONS

const matchCountOptions = [20, 25, 30] as const

const props = defineProps<{
  isConnected: boolean
  matchHistoryLoading: boolean
  matchStatistics: PlayerMatchStats | null
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

const isFilterEmpty = computed(() => !!props.matchStatistics && (props.matchStatistics.totalGames || 0) === 0)

const hasGames = computed(() => (props.matchStatistics?.totalGames || 0) > 0)
const resolvedDisplayGames = computed(() => {
  if (props.displayGames !== null && props.displayGames !== undefined && props.displayGames > 0) {
    return props.displayGames
  }
  return props.matchStatistics?.totalGames || 0
})
const showAiBadge = computed(() => hasGames.value && !!props.aiReady)

/** 样本不足说明放进概览卡底部，标题区不再堆长句 */
const sampleShortfallTip = computed(() => {
  if (!hasGames.value) return ''
  const n = resolvedDisplayGames.value
  const requested = props.matchCount
  if (requested === null || requested === undefined || n <= 0 || n >= requested) return ''
  const mode =
    props.selectedMatchMode && props.selectedMatchMode !== 'all'
      ? `「${getMatchModeLabel(props.selectedMatchMode)}」`
      : '当前模式'
  return `已选 ${requested} 场，${mode}近期仅有 ${n} 场`
})

/** 最近对局胜负点阵：列表通常新→旧，反转为左旧右新；最多 20 点 */
const recentResultDots = computed(() => {
  const list = props.matchStatistics?.recentPerformance || []
  if (!list.length) return [] as boolean[]
  const chronological = [...list].reverse()
  return chronological.slice(-20).map((g) => !!g.win)
})

/** 胜率以 50% 为界：正绿负红 */
const winRateToneClass = computed(() => {
  const rate = props.matchStatistics?.winRate ?? 0
  if (rate > 50) return 'text-emerald-600 dark:text-emerald-400'
  if (rate < 50) return 'text-rose-600 dark:text-rose-400'
  return 'text-foreground'
})

const emptyTitle = computed(() => {
  const mode = props.selectedMatchMode
  if (mode && mode !== 'all') {
    return `最近没有「${getMatchModeLabel(mode)}」对局`
  }
  return '最近没有可展示的对局'
})

const emptyDetail = computed(() => {
  const scanned = props.scannedGames
  if (scanned && scanned > 0) {
    return `已查看最近 ${scanned} 场历史。可以换到全部模式，或打几场后再刷新。`
  }
  return '可以换到全部模式，或打几场后再刷新。'
})

/** Dashboard 常用英雄：最多 Top 5（含 1 场） */
const favoriteChampions = computed(() => (props.matchStatistics?.favoriteChampions || []).slice(0, 5))
const hasFavoriteChampions = computed(() => favoriteChampions.value.length > 0)
const isRankedFilterMode = computed(() => {
  const mode = props.selectedMatchMode
  return mode === 'mixedRanked' || mode === '420' || mode === '440'
})

const hasIdentitySection = computed(() => {
  if (isRankedFilterMode.value) {
    return (props.positionStats || []).some((p) => p.position !== 'UNKNOWN')
  }
  return (
    props.analysisTraits?.some(
      (t) => t.supportsConclusion && t.key.startsWith('mode_affinity') && t.key !== 'mode_affinity_ranked'
    ) ?? false
  )
})

const openGameDetail = (game: MatchPerformance) => {
  selectedGame.value = game
  console.log(game)
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

const { formatGameTime, formatRelativeTime } = useFormatters()

const initialShowCount = 10
const showCount = ref(initialShowCount)
const loadMore = () => {
  showCount.value += 10
}
</script>
