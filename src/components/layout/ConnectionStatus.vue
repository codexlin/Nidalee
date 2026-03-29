<template>
  <div class="flex items-center gap-3 px-4 py-2 rounded-lg border-2 bg-background/50 backdrop-blur-sm shadow-sm">
    <!-- 连接状态指示器 -->
    <div class="flex items-center gap-2">
      <div :class="['animate-pulse h-2 w-2 rounded-full shadow-sm', isConnected ? 'bg-green-500' : 'bg-red-500']" />
      <span class="text-sm font-medium text-foreground">
        {{ isConnected ? summonerInfo?.displayName || '未知召唤师' : '未连接' }}
      </span>
      <GameLauncher v-if="!isConnected" class="ml-2" />
    </div>

    <!-- 等级信息 -->
    <div v-if="isConnected && summonerInfo" class="flex items-center gap-2">
      <div class="h-4 w-px bg-muted-foreground/20 mx-1" />
      <span class="text-xs text-muted-foreground">等级</span>
      <span class="font-bold text-base text-foreground">{{ summonerInfo.summonerLevel }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import GameLauncher from '@/components/common/GameLauncher.vue'

// 直接从 store 获取状态
const dataStore = useDataStore()
const connectionStore = useConnectionStore()

// 从store中解构响应式状态
const { summonerInfo } = storeToRefs(dataStore)
const { isConnected } = storeToRefs(connectionStore)
</script>
