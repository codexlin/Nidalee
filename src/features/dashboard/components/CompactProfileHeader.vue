<template>
  <Card class="relative overflow-hidden">
    <!-- 装饰性光晕 (右下角) -->
    <div class="absolute -bottom-20 -right-20 w-64 h-64 bg-purple-500/10 rounded-full blur-3xl pointer-events-none" />

    <!-- 装饰性光晕 (左上角，更淡) -->
    <div class="absolute -top-16 -left-16 w-48 h-48 bg-primary/5 rounded-full blur-2xl pointer-events-none" />

    <!-- 主内容 -->
    <div class="relative">
      <!-- Top Bar: Avatar + Basic Info + Today's Quick Stats -->
      <div class="flex items-center gap-4 p-4 border-b border-border/50">
        <!-- Avatar with Level Badge -->
        <div class="relative group">
          <div class="absolute inset-0 bg-gradient-to-br from-primary/20 to-transparent rounded-full blur-sm opacity-0 group-hover:opacity-100 transition-opacity" />
          <img
            v-if="summonerInfo?.profileIconId"
            :src="getProfileIconUrl(summonerInfo.profileIconId)"
            class="relative h-14 w-14 rounded-full ring-1 ring-border/50 group-hover:ring-primary/30 transition-all"
          />
          <div
            v-else
            class="relative h-14 w-14 rounded-full bg-muted ring-1 ring-border/50 flex items-center justify-center"
          >
            <User class="h-6 w-6 text-muted-foreground" />
          </div>
          <span class="absolute -bottom-1 -right-1 bg-background text-foreground text-xs font-bold px-1.5 rounded-full ring-1 ring-border shadow-sm">
            {{ summonerInfo?.summonerLevel || 0 }}
          </span>
        </div>

        <!-- Summoner Name + Challenge Points -->
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <h3 class="font-semibold text-base truncate">{{ summonerInfo?.gameName || summonerInfo?.displayName || '未知召唤师' }}</h3>
            <h3 v-if="summonerInfo?.tagLine" class="font-semibold text-base text-muted-foreground">
              #{{ summonerInfo.tagLine }}
            </h3>
            <!-- Connection Status Badge -->
            <Badge
              :variant="isConnected ? 'default' : 'secondary'"
              :class="['shrink-0 gap-1', isConnected ? 'bg-green-500/10 text-green-700 dark:text-green-400 hover:bg-green-500/20' : '']"
            >
              <div :class="['w-1.5 h-1.5 rounded-full', isConnected ? 'bg-green-500 animate-pulse' : 'bg-gray-400']" />
              {{ isConnected ? '已连接' : '未连接' }}
            </Badge>
          </div>
          <div class="flex items-center gap-3 text-sm text-muted-foreground mt-0.5">
            <span
              >挑战积分: <span class="text-foreground font-medium">{{ summonerInfo?.challengePoints?.toLocaleString() || 0 }}</span></span
            >
            <span>•</span>
            <span>会话: <span class="text-foreground font-medium">{{ sessionDuration }}</span></span>
          </div>
        </div>

        <!-- Today's Quick Stats -->
        <div class="flex items-center gap-4 pl-4 border-l border-border/50">
          <div class="text-center">
            <p class="text-xs text-muted-foreground">今日</p>
            <p class="font-semibold">{{ todayMatches?.total || 0 }}</p>
          </div>
          <div class="flex items-center gap-1 text-sm">
            <span class="font-bold text-green-600 dark:text-green-400">{{ todayMatches?.wins || 0 }}</span>
            <span class="text-muted-foreground">/</span>
            <span class="font-bold text-red-600 dark:text-red-400">{{ todayMatches?.losses || 0 }}</span>
          </div>
          <Badge
            :variant="winRate >= 60 ? 'default' : winRate >= 50 ? 'secondary' : 'destructive'"
            class="font-semibold"
          >
            {{ winRate.toFixed(0) }}%
          </Badge>
        </div>
      </div>

      <!-- Bottom: Ranked Stats (Compact) -->
      <div class="grid grid-cols-2 gap-3 p-4">
        <!-- Solo/Duo Rank -->
        <div
          class="flex items-center gap-3 p-3 rounded-lg border border-border/50 bg-card/50 hover:bg-card hover:border-border transition-all cursor-pointer group"
          @click="handleRankClick('solo')"
        >
          <div class="relative shrink-0">
            <img
              :src="getRankIconUrl(soloRank.tier)"
              class="h-10 w-10 transition-transform group-hover:scale-105"
            />
            <div class="absolute inset-0 bg-primary/20 rounded-full blur-md opacity-0 group-hover:opacity-100 transition-opacity" />
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-xs text-muted-foreground">单双排位</p>
            <p class="font-semibold truncate">{{ soloRank.tier }} {{ soloRank.rank }}</p>
          </div>
          <div class="text-right shrink-0">
            <p class="font-bold text-foreground">{{ soloRank.leaguePoints }}<span class="text-xs font-normal text-muted-foreground">LP</span></p>
            <p class="text-xs font-medium" :class="soloRank.winRate >= 50 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
              {{ soloRank.winRate }}%
            </p>
          </div>
        </div>

        <!-- Flex Rank -->
        <div
          class="flex items-center gap-3 p-3 rounded-lg border border-border/50 bg-card/50 hover:bg-card hover:border-border transition-all cursor-pointer group"
          @click="handleRankClick('flex')"
        >
          <div class="relative shrink-0">
            <img
              :src="getRankIconUrl(flexRank.tier)"
              class="h-10 w-10 transition-transform group-hover:scale-105"
            />
            <div class="absolute inset-0 bg-primary/20 rounded-full blur-md opacity-0 group-hover:opacity-100 transition-opacity" />
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-xs text-muted-foreground">灵活组排</p>
            <p class="font-semibold truncate">{{ flexRank.tier }} {{ flexRank.rank }}</p>
          </div>
          <div class="text-right shrink-0">
            <p class="font-bold text-foreground">{{ flexRank.leaguePoints }}<span class="text-xs font-normal text-muted-foreground">LP</span></p>
            <p class="text-xs font-medium" :class="flexRank.winRate >= 50 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'">
              {{ flexRank.winRate }}%
            </p>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { getProfileIconUrl, getRankIconUrl } from '@/lib'
import { User } from 'lucide-vue-next'
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
}>()

const emit = defineEmits<{
  (e: 'rank-click', queueType: 'solo' | 'flex'): void
}>()

const handleRankClick = (queueType: 'solo' | 'flex') => {
  emit('rank-click', queueType)
}
</script>
