<template>
  <!-- 勿用 h-screen：外层已有 TitleBar + Nav + padding，100vh 会把「正在大厅」顶偏下 -->
  <div class="flex min-h-[calc(100dvh-10.5rem)] w-full items-center justify-center bg-background">
    <div class="w-full max-w-2xl mx-auto px-2">
      <!-- Matchmaking View -->
      <div v-if="['Matchmaking'].includes(currentPhase)">
        <Card class="p-6 rounded-lg shadow-sm">
          <div class="space-y-6">
            <div class="flex items-center justify-between">
              <div class="flex items-center space-x-4">
                <div class="relative flex items-center justify-center h-12 w-12 rounded-full bg-muted">
                  <Search class="h-6 w-6 text-primary" />
                  <div class="absolute top-0 right-0 w-3 h-3 rounded-full bg-primary animate-ping"></div>
                </div>
                <div>
                  <h3 class="text-lg font-semibold text-foreground">正在匹配中</h3>
                  <p class="text-sm text-muted-foreground">正在为您寻找实力相当的对手</p>
                </div>
              </div>
              <Badge variant="outline" class="text-sm font-medium">匹配中</Badge>
            </div>
            <div class="space-y-6">
              <div class="grid grid-cols-2 gap-4 text-center">
                <div>
                  <p class="text-sm text-muted-foreground">预估时间</p>
                  <p class="text-2xl font-bold text-primary">
                    {{ formatTime(matchmakingState?.estimatedQueueTime ?? 0) }}
                  </p>
                </div>
                <div>
                  <p class="text-sm text-muted-foreground">已等待</p>
                  <p class="text-2xl font-bold text-foreground">{{ formatTime(actualWaitTime) }}</p>
                </div>
              </div>
              <Progress :model-value="waitProgress" />
            </div>
            <div class="pt-4 border-t border-border/50">
              <Button class="w-full h-12 text-base font-semibold" variant="destructive" @click="handleMatchmaking">
                <Loader2 class="h-4 w-4 mr-2 animate-spin" />
                取消匹配
              </Button>
            </div>
          </div>
        </Card>
      </div>

      <!-- ChampSelect Loading View -->
      <div v-else-if="['ChampSelect', 'ReadyCheck', 'Found'].includes(currentPhase)">
        <div class="flex flex-col items-center justify-center">
          <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary mb-6"></div>
          <h2 class="text-2xl font-semibold text-foreground">正在分析对局...</h2>
          <p class="text-muted-foreground mt-2 max-w-sm text-center">正在获取双方玩家的战绩与英雄数据，请稍候。</p>
        </div>
      </div>

      <!-- Generic Status View (for Lobby, ReadyCheck, None, etc.) -->
      <div v-else>
        <div class="text-center space-y-4">
          <div class="w-20 h-20 mx-auto rounded-full flex items-center justify-center bg-muted">
            <component :is="statusIcon" class="h-10 w-10 text-muted-foreground" />
          </div>
          <h2 class="text-2xl font-bold text-foreground">{{ statusTitle }}</h2>
          <p class="text-muted-foreground leading-relaxed max-w-md mx-auto">{{ statusDescription }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Search, Loader2, Gamepad2, Users, Hourglass } from 'lucide-vue-next'
import { useMatchAnalysisStore } from '@/features/match-analysis/store'

const matchmakingStore = useMatchmakingStore()
const matchAnalysisStore = useMatchAnalysisStore()

const { state: matchmakingState } = storeToRefs(matchmakingStore)
const { currentPhase } = storeToRefs(matchAnalysisStore)

const { handleMatchmaking } = useMatchmaking()

// --- Matchmaking Timer Logic ---
const matchmakingStartTime = ref<Date | null>(null)
const now = ref(new Date())
let timer: ReturnType<typeof setInterval> | null = null

watch(
  () => matchmakingState.value?.searchState,
  (newState, oldState) => {
    if (newState === 'Searching' && oldState !== 'Searching') {
      matchmakingStartTime.value = new Date()
    } else if (newState !== 'Searching') {
      matchmakingStartTime.value = null
    }
  },
  { immediate: true }
)

watch(
  matchmakingStartTime,
  (newStartTime) => {
    if (newStartTime && !timer) {
      timer = setInterval(() => {
        now.value = new Date()
      }, 1000)
    } else if (!newStartTime && timer) {
      clearInterval(timer)
      timer = null
    }
  },
  { immediate: true }
)

const actualWaitTime = computed(() => {
  if (!matchmakingStartTime.value) {
    return 0
  }
  return Math.floor((now.value.getTime() - matchmakingStartTime.value.getTime()) / 1000)
})

const waitProgress = computed(() => {
  if (!matchmakingState.value?.estimatedQueueTime || actualWaitTime.value === 0) {
    return 0
  }
  const progress = (actualWaitTime.value / matchmakingState.value.estimatedQueueTime) * 100
  return Math.min(progress, 100)
})

const formatTime = (timeValue: number) => {
  const seconds = Math.max(0, Math.floor(timeValue))
  if (seconds < 60) {
    return `${seconds}s`
  }
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  return `${minutes}m ${remainingSeconds}s`
}

// --- Generic Status Logic ---
const statusIcon = shallowRef<unknown>(Gamepad2)
const statusTitle = ref('')
const statusDescription = ref('')

watch(
  currentPhase,
  (phase) => {
    console.log('[GameStatusHub]👍 当前阶段:', phase)
    switch (phase) {
      case 'Lobby':
        statusIcon.value = Users
        statusTitle.value = '房间中'
        statusDescription.value = '请开始匹配，进入选人后将自动显示队伍信息。'
        break
      case 'Reconnect':
        statusIcon.value = Users
        statusTitle.value = '重新连接'
        statusDescription.value = '已断开，请重新连接游戏。'
        break
      case 'None':
        statusIcon.value = Users
        statusTitle.value = '正在大厅'
        statusDescription.value = '请选择游戏模式并开始匹配，进入选人后将自动显示队伍信息。'
        break
      case 'ReadyCheck':
        statusIcon.value = Hourglass
        statusTitle.value = '等待所有玩家确认'
        statusDescription.value = '对局已找到，请在客户端中接受对局。'
        break

      case 'EndOfGame':
        statusIcon.value = Gamepad2
        statusTitle.value = '对局结束'
        statusDescription.value = '返回大厅后将恢复。'
        break
      default:
        statusIcon.value = Gamepad2
        statusTitle.value = '等待游戏客户端'
        statusDescription.value = '请启动并登录英雄联盟客户端。'
        break
    }
  },
  { immediate: true }
)
</script>
