<template>
  <div class="surface-chip flex items-center gap-3 px-4 py-2" :class="isConnected ? '' : 'border-destructive/40'">
    <!-- 连接状态指示器 -->
    <div class="flex items-center gap-2">
      <div
        :class="['h-2 w-2 shrink-0 rounded-full', isConnected ? 'bg-emerald-500 animate-pulse' : 'bg-destructive']"
      />
      <span class="text-sm font-medium text-foreground">
        {{ isConnected ? summonerInfo?.displayName || '未知召唤师' : '未连接' }}
      </span>
      <GameLauncher v-if="!isConnected" class="ml-2" />
    </div>

    <!-- 等级信息 -->
    <div v-if="isConnected && summonerInfo" class="flex items-center gap-2">
      <div class="h-4 w-px bg-border" />
      <span class="text-xs text-muted-foreground">等级</span>
      <span class="text-sm font-semibold text-foreground tabular-nums">{{ summonerInfo.summonerLevel }}</span>
    </div>

    <!-- 会话时长 + 自动功能 -->
    <div v-if="isConnected" class="flex items-center gap-2">
      <div class="h-4 w-px bg-border" />
      <div class="flex items-center gap-1">
        <Clock class="h-3.5 w-3.5 text-muted-foreground" />
        <span class="text-xs text-muted-foreground">会话</span>
        <span class="text-sm font-medium text-foreground tabular-nums">{{ sessionDuration }}</span>
      </div>
      <div class="h-4 w-px bg-border" />
      <div class="flex items-center gap-1">
        <span class="text-xs text-muted-foreground">自动启用</span>
        <span class="text-sm font-medium text-foreground tabular-nums">{{ enabledFunctionsCount }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Clock } from 'lucide-vue-next'
import GameLauncher from '@/components/common/GameLauncher.vue'

const dataStore = useDataStore()
const connectionStore = useConnectionStore()
const sessionStore = useSessionStore()
const autoFunctionStore = useAutoFunctionStore()

const { summonerInfo } = storeToRefs(dataStore)
const { isConnected } = storeToRefs(connectionStore)
const sessionDuration = computed(() => sessionStore.formattedTotal)
const { enabledFunctionsCount } = storeToRefs(autoFunctionStore)
</script>
