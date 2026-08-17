<script setup lang="ts">
import { Gamepad2, Users } from 'lucide-vue-next'

import { Card, CardContent } from '@/components/ui/card'
import { Spinner } from '@/components/ui/spinner'
import { useMatchAnalysisStore } from '@/features/match-analysis/store'

import MatchmakingStatusCard from './MatchmakingStatusCard.vue'

const matchmakingStore = useMatchmakingStore()
const matchAnalysisStore = useMatchAnalysisStore()
const { state: matchmakingState } = storeToRefs(matchmakingStore)
const { currentPhase } = storeToRefs(matchAnalysisStore)
const { handleMatchmaking } = useMatchmaking()

const matchmakingStartTime = shallowRef<number | null>(null)
const now = shallowRef(Date.now())
const isCancelling = shallowRef(false)

watch(
  () => matchmakingState.value?.searchState,
  (searchState, previousState) => {
    if (searchState === 'Searching' && previousState !== 'Searching') {
      matchmakingStartTime.value = Date.now()
      now.value = Date.now()
    } else if (searchState !== 'Searching') {
      matchmakingStartTime.value = null
    }
  },
  { immediate: true }
)

watch(
  matchmakingStartTime,
  (startedAt, _, onCleanup) => {
    if (startedAt === null) return

    const timer = window.setInterval(() => {
      now.value = Date.now()
    }, 1000)
    onCleanup(() => window.clearInterval(timer))
  },
  { immediate: true }
)

const elapsedSeconds = computed(() => {
  if (matchmakingStartTime.value === null) return 0
  return Math.floor((now.value - matchmakingStartTime.value) / 1000)
})

const estimatedSeconds = computed(() => matchmakingState.value?.estimatedQueueTime ?? 0)

const waitProgress = computed(() => {
  if (estimatedSeconds.value <= 0) return 0
  return Math.min((elapsedSeconds.value / estimatedSeconds.value) * 100, 100)
})

const status = computed(() => {
  switch (currentPhase.value) {
    case 'Lobby':
      return { icon: Users, title: '房间中', description: '开始匹配后，这里会同步显示当前进度。' }
    case 'Reconnect':
      return { icon: Users, title: '等待重新连接', description: '请在游戏客户端中重新连接当前对局。' }
    case 'None':
      return { icon: Users, title: '正在大厅', description: '请选择游戏模式并开始匹配。' }
    case 'EndOfGame':
      return { icon: Gamepad2, title: '对局已结束', description: '返回大厅后即可开始下一场对局。' }
    default:
      return { icon: Gamepad2, title: '等待游戏客户端', description: '请启动并登录英雄联盟客户端。' }
  }
})

async function cancelMatchmaking() {
  if (isCancelling.value) return
  isCancelling.value = true
  try {
    await handleMatchmaking()
  } finally {
    isCancelling.value = false
  }
}
</script>

<template>
  <div class="flex min-h-[calc(100dvh-10.5rem)] w-full items-center justify-center px-4">
    <MatchmakingStatusCard
      v-if="currentPhase === 'Matchmaking'"
      :estimated-seconds="estimatedSeconds"
      :elapsed-seconds="elapsedSeconds"
      :progress="waitProgress"
      :cancelling="isCancelling"
      @cancel="cancelMatchmaking"
    />

    <Card
      v-else-if="['ChampSelect', 'ReadyCheck', 'Found'].includes(currentPhase)"
      class="surface-raised w-full max-w-lg py-0"
    >
      <CardContent class="flex flex-col items-center gap-3 px-6 py-10 text-center">
        <div class="surface-inset flex size-12 items-center justify-center rounded-xl">
          <Spinner class="size-5 text-primary" />
        </div>
        <h2 class="text-lg font-semibold text-foreground">正在准备对局分析</h2>
        <p class="max-w-sm text-sm leading-relaxed text-muted-foreground">
          正在同步双方玩家与英雄信息，数据就绪后会自动展示。
        </p>
      </CardContent>
    </Card>

    <Card v-else class="surface-raised w-full max-w-lg py-0">
      <CardContent class="flex flex-col items-center gap-3 px-6 py-10 text-center">
        <div class="surface-inset flex size-12 items-center justify-center rounded-xl">
          <component :is="status.icon" class="size-5 text-muted-foreground" />
        </div>
        <h2 class="text-lg font-semibold text-foreground">{{ status.title }}</h2>
        <p class="max-w-sm text-sm leading-relaxed text-muted-foreground">{{ status.description }}</p>
      </CardContent>
    </Card>
  </div>
</template>
