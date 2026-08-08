<template>
  <div
    class="sticky top-0 z-10 flex items-center justify-between gap-2 px-3 py-2 border-b border-border/50 bg-background/80 backdrop-blur"
  >
    <!-- 左侧：队伍信息 -->
    <div class="flex items-center gap-2 min-w-0">
      <!-- 队伍标识 -->
      <div class="flex items-center gap-1.5 px-2 py-1 rounded-full text-xs font-medium" :class="teamTypeClass">
        <div class="w-1.5 h-1.5 rounded-full" :class="teamIndicatorClass" />
        <span>{{ teamTypeName }}</span>
      </div>

      <!-- 分隔线 -->
      <div class="h-3 w-px bg-border/60" />

      <!-- 阶段信息 -->
      <div class="flex items-center gap-1.5">
        <span class="text-xs text-muted-foreground">阶段</span>
        <div class="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-full text-xs font-medium" :class="phaseClass">
          <div class="w-1 h-1 rounded-full" :class="phaseIndicatorClass" />
          <span>{{ phaseDisplayName }}</span>
        </div>
      </div>

      <!-- 对局类型 (双方队伍都显示) -->
      <template v-if="queueTypeLabel">
        <!-- 分隔线 -->
        <div class="h-3 w-px bg-border/60" />

        <div class="flex items-center gap-2">
          <span class="text-sm text-muted-foreground">对局</span>
          <div class="inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs font-medium" :class="queueClass">
            <span>{{ queueTypeIcon }}</span>
            <span>{{ queueTypeLabel }}</span>
          </div>
        </div>
      </template>

      <!-- 分隔线 -->
      <div class="h-4 w-px bg-border/60" />

      <!-- 队伍人数 -->
      <div class="flex items-center gap-2">
        <span class="text-sm text-muted-foreground">队伍</span>
        <span class="text-sm font-mono font-medium text-foreground">{{ teamCount }}/5</span>
      </div>

      <!-- 数据状态 -->
      <!-- <div class="flex items-center gap-1 text-xs" :class="dataStatusClass">
        <div class="w-1.5 h-1.5 rounded-full" :class="dataIndicatorClass" />
        <span>{{ dataStatusText }}</span>
      </div> -->
      <Tooltip v-if="teamType === 'enemy'">
        <TooltipTrigger as-child>
          <span class="text-center text-xs text-muted-foreground"> <Info /></span>
        </TooltipTrigger>
        <TooltipContent>
          <div class="text-center text-xs text-muted-foreground">
            <p>💡 敌方完整信息将在游戏开始后获取</p>
          </div>
        </TooltipContent>
      </Tooltip>
    </div>

    <!-- 右侧：操作按钮 -->
    <!-- <div class="flex items-center gap-2">
      <div v-if="loading" class="flex items-center gap-2 text-xs text-muted-foreground">
        <div class="w-3 h-3 border border-primary/30 border-t-primary rounded-full animate-spin" />
        <span>加载中</span>
      </div>

      <button
        v-else
        type="button"
        class="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium text-muted-foreground hover:text-foreground transition-colors"
        @click="$emit('refresh')"
      >
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
          />
        </svg>
        <span>刷新</span>
      </button>
    </div> -->
  </div>
</template>

<script setup lang="ts">
import type { GamePhase } from '@/types/match-analysis'
import { useMatchAnalysisStore } from '@/features/match-analysis/store'
import { Info } from 'lucide-vue-next'
interface Props {
  teamType: 'ally' | 'enemy'
  phase: GamePhase
  teamCount: number
  hasData: boolean
  loading: boolean
}

const props = defineProps<Props>()

defineEmits<{
  refresh: []
}>()

const matchAnalysisStore = useMatchAnalysisStore()

// 对局类型信息
const queueTypeLabel = computed(() => matchAnalysisStore.queueTypeLabel)
const queueTypeIcon = computed(() => matchAnalysisStore.queueTypeIcon)
const isRankedGame = computed(() => matchAnalysisStore.isRankedGame)

// 队伍类型相关
const teamTypeName = computed(() => {
  return props.teamType === 'ally' ? '我方队伍' : '敌方队伍'
})

const teamTypeClass = computed(() => {
  return props.teamType === 'ally'
    ? 'bg-blue-50 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 ring-1 ring-blue-200 dark:ring-blue-800'
    : 'bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-300 ring-1 ring-red-200 dark:ring-red-800'
})

const teamIndicatorClass = computed(() => {
  return props.teamType === 'ally' ? 'bg-blue-500' : 'bg-red-500'
})

// 阶段相关
const phaseDisplayName = computed(() => {
  switch (props.phase) {
    case 'ChampSelect':
      return '选人阶段'
    case 'InProgress':
      return '游戏中'
    case 'Lobby':
      return '大厅'
    case 'Matchmaking':
      return '匹配中'
    case 'EndOfGame':
      return '游戏结束'
    default:
      return '未知'
  }
})

const phaseClass = computed(() => {
  switch (props.phase) {
    case 'ChampSelect':
      return 'bg-purple-50 dark:bg-purple-900/30 text-purple-700 dark:text-purple-300 ring-1 ring-purple-200 dark:ring-purple-800'
    case 'InProgress':
      return 'bg-green-50 dark:bg-green-900/30 text-green-700 dark:text-green-300 ring-1 ring-green-200 dark:ring-green-800'
    case 'Matchmaking':
      return 'bg-yellow-50 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-300 ring-1 ring-yellow-200 dark:ring-yellow-800'
    default:
      return 'bg-gray-50 dark:bg-gray-900/30 text-gray-700 dark:text-gray-300 ring-1 ring-gray-200 dark:ring-gray-800'
  }
})

const phaseIndicatorClass = computed(() => {
  switch (props.phase) {
    case 'ChampSelect':
      return 'bg-purple-500'
    case 'InProgress':
      return 'bg-green-500'
    case 'Matchmaking':
      return 'bg-yellow-500 animate-pulse'
    default:
      return 'bg-gray-500'
  }
})

// 对局类型样式
const queueClass = computed(() => {
  if (isRankedGame.value) {
    return 'bg-amber-50 dark:bg-amber-900/30 text-amber-700 dark:text-amber-300 ring-1 ring-amber-200 dark:ring-amber-800'
  }
  return 'bg-cyan-50 dark:bg-cyan-900/30 text-cyan-700 dark:text-cyan-300 ring-1 ring-cyan-200 dark:ring-cyan-800'
})
</script>
