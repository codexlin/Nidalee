<template>
  <Card class="relative overflow-hidden">
    <!-- 装饰性光晕 (右下角) -->
    <div class="absolute -bottom-20 -right-20 w-64 h-64 bg-purple-500/10 rounded-full blur-3xl pointer-events-none" />

    <!-- 装饰性光晕 (左上角，更淡) -->
    <div class="absolute -top-16 -left-16 w-48 h-48 bg-primary/5 rounded-full blur-2xl pointer-events-none" />

    <!-- 主内容 -->
    <div class="relative p-4">
      <div class="flex items-start gap-4">
        <!-- Avatar with Level Badge -->
        <div class="relative group shrink-0">
          <div class="absolute inset-0 bg-gradient-to-br from-primary/20 to-transparent rounded-full blur-sm opacity-0 group-hover:opacity-100 transition-opacity" />
          <img
            v-if="summonerInfo?.profileIconId"
            :src="getProfileIconUrl(summonerInfo.profileIconId)"
            class="relative h-16 w-16 rounded-full ring-1 ring-border/50 group-hover:ring-primary/30 transition-all"
          />
          <div
            v-else
            class="relative h-16 w-16 rounded-full bg-muted ring-1 ring-border/50 flex items-center justify-center"
          >
            <User class="h-7 w-7 text-muted-foreground" />
          </div>
          <span class="absolute -bottom-1 -right-1 bg-background text-foreground text-xs font-bold px-1.5 rounded-full ring-1 ring-border shadow-sm">
            {{ summonerInfo?.summonerLevel || 0 }}
          </span>
        </div>

        <!-- Middle Section: Summoner Info + Ranked Stats -->
        <div class="flex-1 min-w-0 space-y-3">
          <!-- Name + Status Row -->
          <div class="flex items-center gap-2 flex-wrap">
            <h3 class="font-semibold text-base">{{ summonerInfo?.gameName || summonerInfo?.displayName || '未知召唤师' }}</h3>
            <span v-if="summonerInfo?.tagLine" class="text-muted-foreground text-sm">#{{ summonerInfo.tagLine }}</span>
            <Badge
              :variant="isConnected ? 'default' : 'secondary'"
              :class="['gap-1 text-xs', isConnected ? 'bg-green-500/10 text-green-700 dark:text-green-400' : '']"
            >
              <div :class="['w-1.5 h-1.5 rounded-full shrink-0', isConnected ? 'bg-green-500 animate-pulse' : 'bg-gray-400']" />
              {{ isConnected ? '已连接' : '未连接' }}
            </Badge>
          </div>

          <!-- Info Row -->
          <div class="flex items-center gap-3 text-sm text-muted-foreground">
            <span>挑战积分: <span class="text-foreground font-medium">{{ summonerInfo?.challengePoints?.toLocaleString() || 0 }}</span></span>
            <span>•</span>
            <span>会话: <span class="text-foreground font-medium">{{ sessionDuration }}</span></span>
          </div>

          <!-- Ranked Stats Row (Compact) -->
          <div class="flex items-center gap-3">
            <!-- Solo Rank -->
            <div
              class="flex items-center gap-2 px-3 py-1.5 rounded-lg border border-border/50 bg-muted/30 hover:bg-muted/50 transition-colors cursor-pointer group"
              @click="handleRankClick('solo')"
            >
              <div class="relative">
                <img
                  v-if="soloRank.tier !== 'UNRANKED'"
                  :src="getTierIconUrl(soloRank.tier)"
                  class="h-7 w-7 transition-transform group-hover:scale-110 breath-glow"
                  :style="getRankGlowStyle(soloRank.tier)"
                />
                <div v-else class="h-7 w-7 rounded-full bg-muted flex items-center justify-center">
                  <Shield class="h-4 w-4 text-muted-foreground" />
                </div>
              </div>
              <div class="flex flex-col">
                <span class="text-xs text-muted-foreground leading-tight">单双</span>
                <span class="text-sm font-semibold leading-tight">{{ soloRank.tier === 'UNRANKED' ? '未定级' : formatRankTierShort(soloRank.tier) }}</span>
              </div>
              <div class="flex flex-col items-end">
                <span class="text-xs font-medium leading-tight">{{ soloRank.leaguePoints }}</span>
                <span class="text-xs leading-tight" :class="soloRank.winRate >= 50 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
                  {{ soloRank.winRate }}%
                </span>
              </div>
            </div>

            <!-- Flex Rank -->
            <div
              class="flex items-center gap-2 px-3 py-1.5 rounded-lg border border-border/50 bg-muted/30 hover:bg-muted/50 transition-colors cursor-pointer group"
              @click="handleRankClick('flex')"
            >
              <div class="relative">
                <img
                  v-if="flexRank.tier !== 'UNRANKED'"
                  :src="getTierIconUrl(flexRank.tier)"
                  class="h-7 w-7 transition-transform group-hover:scale-110 breath-glow"
                  :style="getRankGlowStyle(flexRank.tier)"
                />
                <div v-else class="h-7 w-7 rounded-full bg-muted flex items-center justify-center">
                  <Shield class="h-4 w-4 text-muted-foreground" />
                </div>
              </div>
              <div class="flex flex-col">
                <span class="text-xs text-muted-foreground leading-tight">灵活</span>
                <span class="text-sm font-semibold leading-tight">{{ flexRank.tier === 'UNRANKED' ? '未定级' : formatRankTierShort(flexRank.tier) }}</span>
              </div>
              <div class="flex flex-col items-end">
                <span class="text-xs font-medium leading-tight">{{ flexRank.leaguePoints }}</span>
                <span class="text-xs leading-tight" :class="flexRank.winRate >= 50 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
                  {{ flexRank.winRate }}%
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Right Section: Stats -->
        <div class="flex items-center gap-3 pl-3 border-l border-border/50 shrink-0">
          <!-- Auto Functions -->
          <div class="flex flex-col items-center px-3 py-2 rounded-lg bg-purple-500/10 border border-purple-500/20">
            <Sparkles class="h-4 w-4 text-purple-500" />
            <span class="text-xs font-medium text-purple-700 dark:text-purple-300 mt-0.5">自动</span>
            <span class="text-sm font-bold text-purple-600 dark:text-purple-400">{{ enabledFunctionsCount }}</span>
          </div>

          <!-- Today's Stats -->
          <div class="flex flex-col items-center px-3 py-2">
            <span class="text-xs text-muted-foreground">今日</span>
            <span class="text-lg font-bold">{{ todayMatches?.total || 0 }}</span>
            <div class="flex items-center gap-1 text-xs">
              <span class="font-bold text-green-600 dark:text-green-400">{{ todayMatches?.wins || 0 }}</span>
              <span class="text-muted-foreground">/</span>
              <span class="font-bold text-red-600 dark:text-red-400">{{ todayMatches?.losses || 0 }}</span>
            </div>
          </div>

          <!-- Win Rate Badge -->
          <Badge
            :variant="winRate >= 60 ? 'default' : winRate >= 50 ? 'secondary' : 'destructive'"
            class="h-fit px-3 py-1.5"
          >
            <span class="text-sm font-bold">{{ winRate.toFixed(0) }}%</span>
          </Badge>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { getProfileIconUrl, getTierIconUrl } from '@/lib'
import { User, Shield, Sparkles } from 'lucide-vue-next'
import { Badge } from '@/components/ui/badge'
import { Card } from '@/components/ui/card'

interface RankInfo {
  tier: string
  rank: string
  leaguePoints: number
  winRate: number
}

interface TodayMatches {
  total: number
  wins: number
  losses: number
}

const props = defineProps<{
  isConnected: boolean
  summonerInfo: any
  todayMatches: TodayMatches
  winRate: number
  soloRank: RankInfo
  flexRank: RankInfo
  sessionDuration: string
  enabledFunctionsCount: number
}>()

const emit = defineEmits<{
  (e: 'rank-click', queueType: 'solo' | 'flex'): void
}>()

const handleRankClick = (queueType: 'solo' | 'flex') => {
  emit('rank-click', queueType)
}

// 段位短名称映射
const formatRankTierShort = (tier: string): string => {
  const tierMap: Record<string, string> = {
    IRON: '黑铁',
    BRONZE: '青铜',
    SILVER: '白银',
    GOLD: '黄金',
    PLATINUM: '铂金',
    EMERALD: '翡翠',
    DIAMOND: '钻石',
    MASTER: '大师',
    GRANDMASTER: '宗师',
    CHALLENGER: '王者'
  }
  return tierMap[tier] || tier
}

// 段位光晕颜色
const rankGlowColorMap: Record<string, string> = {
  IRON: '#6e6e6e',
  BRONZE: '#b87333',
  SILVER: '#bfc1c2',
  GOLD: '#f7c873',
  PLATINUM: '#3fd8e0',
  EMERALD: '#34d399',
  DIAMOND: '#60a5fa',
  MASTER: '#a78bfa',
  GRANDMASTER: '#f87171',
  CHALLENGER: '#ffe066'
}

const getRankGlowStyle = (tier: string) => {
  const color = rankGlowColorMap[tier]
  if (!color) return {}
  return {
    '--glow-color': color,
    '--glow-color-a': color + '80'
  } as any
}
</script>

<style scoped>
@keyframes breath-glow {
  0% {
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.1),
      0 0 8px 1px var(--glow-color),
      0 0 16px 2px var(--glow-color-a);
  }
  50% {
    box-shadow:
      0 0 0 2px rgba(255, 255, 255, 0.2),
      0 0 14px 3px var(--glow-color),
      0 0 28px 4px var(--glow-color-a);
  }
  100% {
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.1),
      0 0 8px 1px var(--glow-color),
      0 0 16px 2px var(--glow-color-a);
  }
}

.breath-glow {
  animation: breath-glow 2.4s ease-in-out infinite;
  border-radius: 50%;
}
</style>
