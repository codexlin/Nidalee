<template>
  <div
    class="surface-chip flex items-center gap-2 px-3 py-1.5"
    :class="isConnected ? '' : 'border-destructive/50'"
  >
    <div class="flex items-center gap-2">
      <div
        :class="['h-1.5 w-1.5 shrink-0 rounded-full', isConnected ? 'bg-emerald-500' : 'bg-destructive']"
      />
      <span class="max-w-28 truncate text-sm font-medium text-foreground">
        {{ isConnected ? summonerInfo?.displayName || '未知召唤师' : '未连接' }}
      </span>
    </div>

    <GameLauncher v-if="!isConnected" />

    <template v-if="isConnected && summonerInfo">
      <div class="hidden h-3.5 w-px bg-border sm:block" />
      <div class="hidden items-center gap-1 sm:flex">
        <span class="text-xs text-muted-foreground">等级</span>
        <span class="text-sm font-medium text-foreground tabular-nums">{{ summonerInfo.summonerLevel }}</span>
      </div>
    </template>

    <template v-if="isConnected">
      <div class="hidden h-3.5 w-px bg-border md:block" />
      <div class="hidden items-center gap-1 md:flex">
        <Clock class="size-3.5 text-muted-foreground" />
        <span class="text-sm font-medium text-foreground tabular-nums">{{ sessionDuration }}</span>
      </div>
      <div class="hidden h-3.5 w-px bg-border md:block" />
      <div class="hidden items-center gap-1 md:flex">
        <span class="text-xs text-muted-foreground">自动</span>
        <span class="text-sm font-medium text-foreground tabular-nums">{{ enabledFunctionsCount }}</span>
      </div>
    </template>
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
