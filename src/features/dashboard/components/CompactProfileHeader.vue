<template>
  <Card class="relative overflow-hidden rounded-2xl shadow-xl bg-gradient-to-br from-white/80 to-muted/60 dark:from-background/80 dark:to-muted/40 border border-border">
    <!-- 装饰性光晕 (右下角) -->
    <div class="absolute -bottom-20 -right-20 w-64 h-64 bg-purple-500/10 rounded-full blur-3xl pointer-events-none" />

    <!-- 装饰性光晕 (左上角，更淡) -->
    <div class="absolute -top-16 -left-16 w-48 h-48 bg-primary/5 rounded-full blur-2xl pointer-events-none" />

    <!-- 主内容：左中右三栏布局 -->
    <div class="relative px-5">
      <div class="flex items-center justify-between gap-6">
        <!-- 左栏：头像 + 基本信息 -->
        <div class="flex items-center gap-4 shrink-0">
          <div class="relative shrink-0">
            <img
              v-if="summonerInfo?.profileIconId"
              :src="getProfileIconUrl(summonerInfo.profileIconId)"
              class="relative h-20 w-20 rounded-full ring-2 ring-border/50 shadow-md"
            />
            <div
              v-else
              class="relative h-20 w-20 rounded-full bg-muted ring-2 ring-border/50 flex items-center justify-center"
            >
              <User class="h-9 w-9 text-muted-foreground" />
            </div>
            <span
              class="absolute -bottom-1 -right-1 bg-background text-foreground text-sm font-bold px-2 rounded-full ring-1 ring-border shadow-sm"
            >
              {{ summonerInfo?.summonerLevel || 0 }}
            </span>
          </div>
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2 flex-wrap mb-1">
              <h3 class="font-bold text-lg truncate">
                {{ summonerInfo?.gameName || summonerInfo?.displayName || '未知召唤师' }}
              </h3>
              <span v-if="summonerInfo?.tagLine" class="text-muted-foreground text-sm font-medium"
                >#{{ summonerInfo.tagLine }}</span
              >
              <Badge
                :variant="isConnected ? 'default' : 'secondary'"
                :class="[
                  'gap-1 px-2 h-6 text-xs shrink-0',
                  isConnected ? 'bg-green-500/10 text-green-700 dark:text-green-400' : ''
                ]"
              >
                <div
                  :class="[
                    'w-1.5 h-1.5 rounded-full shrink-0',
                    isConnected ? 'bg-green-500 animate-pulse' : 'bg-gray-400'
                  ]"
                />
                {{ isConnected ? '已连接' : '未连接' }}
              </Badge>
            </div>
            <div class="flex items-center gap-2 text-sm text-muted-foreground">
              <span
                >挑战积分:
                <span class="text-foreground font-medium">{{
                  (summonerInfo?.challengePoints || 0).toLocaleString()
                }}</span></span
              >
            </div>
          </div>
        </div>

        <!-- 中栏：段位信息（镜像设计） -->
        <div class="flex-1 flex items-center justify-center gap-4">
          <!-- 单双排位（左侧） -->
          <div class="flex items-center gap-3 px-4 rounded-xl">
            <div class="relative shrink-0 p-1.5">
              <img
                v-if="soloRank.tier !== 'UNRANKED'"
                :src="getTierIconUrl(soloRank.tier)"
                class="h-14 w-14 breath-glow"
                :style="getRankGlowStyle(soloRank.tier)"
              />
              <div
                v-else
                class="h-14 w-14 rounded-full bg-muted/40 flex items-center justify-center border border-border/30"
              >
                <Shield class="h-6 w-6 text-muted-foreground" />
              </div>
            </div>
            <div class="flex flex-col flex-1 min-w-0 gap-1">
              <div class="flex items-center gap-1 text-sm text-muted-foreground">
                <User class="h-3.5 w-3.5" />
                <span>单双排位</span>
              </div>
              <span class="text-lg font-semibold truncate">
                {{ soloRank.tier === 'UNRANKED' ? '未定级' : formatRankTierShort(soloRank.tier) }}
                <span
                  v-if="soloRank.tier !== 'UNRANKED' && soloRank.rank"
                  class="text-muted-foreground/70 font-normal"
                  >{{ soloRank.rank }}</span
                >
              </span>
            </div>
            <div class="flex flex-col items-end shrink-0 gap-1">
              <span class="text-lg font-bold text-foreground tabular-nums"
                >{{ soloRank.leaguePoints
                }}<span class="text-sm font-normal text-muted-foreground/70 ml-0.5">LP</span></span
              >
              <span
                class="text-sm font-semibold"
                :class="
                  soloRank.winRate >= 50 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                "
              >
                {{ soloRank.winRate }}%
              </span>
            </div>
          </div>

          <!-- 分隔线 -->
          <div class="w-px h-12 bg-border/40" />

          <!-- 灵活组排（右侧，镜像） -->
          <div class="flex items-center gap-4 px-4 rounded-xl">
            <div class="flex flex-col items-end shrink-0 gap-1">
              <span class="text-lg font-bold text-foreground tabular-nums"
                >{{ flexRank.leaguePoints
                }}<span class="text-sm font-normal text-muted-foreground/70 ml-0.5">LP</span></span
              >
              <span
                class="text-sm font-semibold"
                :class="
                  flexRank.winRate >= 50 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'
                "
              >
                {{ flexRank.winRate }}%
              </span>
            </div>
            <div class="flex flex-col flex-1 min-w-0 text-right gap-1">
              <div class="flex items-center justify-end gap-1 text-sm text-muted-foreground">
                <span>灵活组排</span>
                <Users class="h-3.5 w-3.5" />
              </div>
              <span class="text-lg font-semibold truncate">
                {{ flexRank.tier === 'UNRANKED' ? '未定级' : formatRankTierShort(flexRank.tier) }}
                <span
                  v-if="flexRank.tier !== 'UNRANKED' && flexRank.rank"
                  class="text-muted-foreground/70 font-normal"
                  >{{ flexRank.rank }}</span
                >
              </span>
            </div>
            <div class="relative shrink-0 p-1.5">
              <img
                v-if="flexRank.tier !== 'UNRANKED'"
                :src="getTierIconUrl(flexRank.tier)"
                class="h-14 w-14 breath-glow"
                :style="getRankGlowStyle(flexRank.tier)"
              />
              <div
                v-else
                class="h-14 w-14 rounded-full bg-muted/40 flex items-center justify-center border border-border/30"
              >
                <Shield class="h-6 w-6 text-muted-foreground" />
              </div>
            </div>
          </div>
        </div>

        <!-- 右栏：今日统计（严格对齐，Grid 布局） -->
        <div class="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1.5 text-sm items-center">
          <span class="text-muted-foreground">今日对局</span>
          <span class="font-semibold text-foreground text-right tabular-nums">{{ todayMatches?.total || 0 }}</span>

          <span class="text-muted-foreground">今日战绩</span>
          <div class="flex items-center justify-end gap-1 tabular-nums">
            <span class="font-semibold text-green-600 dark:text-green-400">{{ todayMatches?.wins || 0 }}</span>
            <span class="text-muted-foreground">/</span>
            <span class="font-semibold text-red-600 dark:text-red-400">{{ todayMatches?.losses || 0 }}</span>
          </div>

          <span class="text-muted-foreground">今日胜率</span>
          <span
            class="font-semibold text-right tabular-nums"
            :class="winRate >= 50 ? 'text-green-600 dark:text-green-400' : 'text-red-600 dark:text-red-400'"
            >{{ winRate.toFixed(0) }}%</span
          >
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { getProfileIconUrl, getTierIconUrl } from '@/lib'
import { Shield, User, Users } from 'lucide-vue-next'
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
}>()

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
