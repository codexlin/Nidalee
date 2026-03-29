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

    <!-- 等级和段位信息 -->
    <div v-if="isConnected && summonerInfo" class="flex items-center gap-2">
      <div class="h-4 w-px bg-muted-foreground/20 mx-1" />
      <span class="text-xs text-muted-foreground">等级</span>
      <span class="font-bold text-base text-foreground">{{ summonerInfo.summonerLevel }}</span>
      <span
        v-if="summonerInfo.soloRankTier"
        class="px-2 py-0.5 rounded bg-muted/60 text-primary font-semibold text-sm ml-1"
      >
        {{ formatRankTier(summonerInfo.soloRankTier) }} {{ summonerInfo.soloRankDivision }}
      </span>
    </div>

    <!-- 会话时长 + 自动功能 -->
    <div v-if="isConnected" class="flex items-center gap-2">
      <div class="h-4 w-px bg-muted-foreground/20 mx-1" />
      <!-- 会话时长 -->
      <div class="flex items-center gap-1">
        <Clock class="h-3.5 w-3.5 text-muted-foreground" />
        <span class="text-xs text-muted-foreground">会话</span>
        <span class="text-sm font-medium text-foreground">{{ sessionDuration }}</span>
      </div>
      <!-- 自动功能 -->
      <div class="flex items-center gap-1 px-2 py-1 rounded bg-purple-500/10 border border-purple-500/20">
        <Sparkles class="h-3.5 w-3.5 text-purple-500" />
        <span class="text-xs text-muted-foreground">自动</span>
        <span class="text-sm font-bold text-purple-600 dark:text-purple-400">{{ enabledFunctionsCount }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Clock, Sparkles } from 'lucide-vue-next'
import GameLauncher from '@/components/common/GameLauncher.vue'

// 直接从 store 获取状态
const dataStore = useDataStore()
const connectionStore = useConnectionStore()
const sessionStore = useSessionStore()
const autoFunctionStore = useAutoFunctionStore()

// 从store中解构响应式状态
const { summonerInfo } = storeToRefs(dataStore)
const { isConnected } = storeToRefs(connectionStore)
const sessionDuration = computed(() => sessionStore.formattedTotal)
const { enabledFunctionsCount } = storeToRefs(autoFunctionStore)

// 格式化段位
const formatRankTier = (tier: string): string => {
  const tierMap: Record<string, string> = {
    IRON: '坚韧黑铁',
    BRONZE: '英勇青铜',
    SILVER: '不屈白银',
    GOLD: '荣耀黄金',
    PLATINUM: '华贵铂金',
    EMERALD: '流光翡翠',
    DIAMOND: '璀璨钻石',
    MASTER: '超凡大师',
    GRANDMASTER: '傲世宗师',
    CHALLENGER: '最强王者'
  }
  return tierMap[tier] || tier
}
</script>
