<template>
  <div
    class="relative group bg-card/80 backdrop-blur-sm border-b-2 rounded-none p-1.5 transition-all duration-200 mb-2"
    :class="[
      player.isBot
        ? 'opacity-60 grayscale cursor-not-allowed'
        : 'hover:shadow-md hover:shadow-primary/10 hover:border-primary/30 cursor-pointer',
      isLocal ? 'border-primary' : 'border-border/40'
    ]"
    :style="isLocal ? { boxShadow: '-4px 0 12px -4px var(--color-primary)' } : {}"
    @click="!player.isBot && $emit('select', player)"
  >
    <!-- 第一行：所有基本信息 -->
    <div class="flex items-center gap-1.5 mb-1">
      <!-- 英雄头像 + 本人标识 -->
      <div class="relative flex-shrink-0">
        <div
          class="w-8 h-8 rounded-md overflow-hidden ring-1 ring-border/60 group-hover:ring-primary/40 transition-all relative"
        >
          <!-- 已选择的英雄 -->
          <img
            v-if="player.championId"
            :src="getChampionIconUrl(player.championId)"
            :alt="getChampionName(player.championId)"
            class="w-full h-full object-cover"
          />
          <!-- 预选英雄（半透明） -->
          <img
            v-else-if="player.championPickIntent"
            :src="getChampionIconUrl(player.championPickIntent)"
            :alt="getChampionName(player.championPickIntent)"
            class="w-full h-full object-cover opacity-50"
          />
          <!-- 未选择 -->
          <div v-else class="w-full h-full bg-muted flex items-center justify-center">
            <div class="w-4 h-4 bg-muted-foreground/20 rounded" />
          </div>

          <!-- 预选指示器 -->
          <div
            v-if="!player.championId && player.championPickIntent"
            class="absolute inset-0 flex items-center justify-center bg-black/30"
          >
            <span class="text-[8px] text-white font-bold">预选</span>
          </div>
        </div>
        <!-- 本人标识 -->
        <div
          v-if="isLocal"
          class="absolute -top-1 -right-1 bg-primary text-primary-foreground text-[8px] px-1 py-0.5 rounded-full font-bold z-10"
        >
          我
        </div>
        <!-- 机器人标识 -->
        <div
          v-else-if="player.isBot"
          class="absolute -top-1 -right-1 bg-gray-500 text-white text-[8px] px-1 py-0.5 rounded-full font-bold z-10"
        >
          机器人
        </div>
      </div>

      <!-- 召唤师姓名和英雄（上下排列） -->
      <div class="flex flex-col justify-center min-w-0">
        <div class="flex items-center gap-1">
          <h3 class="text-xs font-bold text-foreground truncate max-w-24">
            {{ player.displayName || '未知召唤师' }}
          </h3>
          <div
            v-if="player.tier"
            class="px-1 py-0.5 text-[9px] font-bold rounded bg-yellow-500/20 text-yellow-700 dark:text-yellow-400 border border-yellow-500/30 flex-shrink-0"
          >
            {{ player.tier }}
            <div v-if="isRanked && player.position" class="mt-0.5 text-[9px] font-semibold text-primary/80 text-center">
              {{ getPositionShort(player.position) }}
            </div>
          </div>
        </div>
        <div class="flex items-center gap-1">
          <span class="text-[10px] text-muted-foreground truncate">
            {{
              player.championId
                ? getChampionName(player.championId)
                : player.championPickIntent
                  ? `预选: ${getChampionName(player.championPickIntent)}`
                  : '未选英雄'
            }}
          </span>
          <span
            v-if="player.assignedPosition"
            class="text-[8px] px-0.5 py-0 bg-muted/50 rounded text-muted-foreground flex-shrink-0"
          >
            {{ getPositionLabel(player.assignedPosition) }}
          </span>
        </div>
      </div>

      <!-- 分隔符 -->
      <div class="h-8 w-px bg-border flex-shrink-0" />

      <!-- 召唤师技能 -->
      <div class="flex gap-0.5 flex-shrink-0">
        <div
          v-for="spellId in [player.spell1Id, player.spell2Id]"
          :key="spellId"
          class="w-6 h-6 rounded overflow-hidden ring-1 ring-border/40"
        >
          <img
            v-if="spellId && getSpellMeta(spellId)?.icon"
            :src="getSpellMeta(spellId).icon"
            :alt="getSpellMeta(spellId).label"
            class="w-full h-full object-cover"
          />
          <div v-else class="w-full h-full bg-muted" />
        </div>
      </div>

      <!-- 平均KDA (非机器人) -->
      <div v-if="playerStats && !player.isBot" class="flex items-center gap-1 flex-shrink-0">
        <span class="text-[9px] text-muted-foreground">KDA</span>
        <div class="flex items-center gap-0.5">
          <span class="text-xs text-green-500 font-medium">{{ playerStats.avgKills?.toFixed(1) || '0' }}</span>
          <span class="text-[10px] text-muted-foreground">/</span>
          <span class="text-xs text-red-500 font-medium">{{ playerStats.avgDeaths?.toFixed(1) || '0' }}</span>
          <span class="text-[10px] text-muted-foreground">/</span>
          <span class="text-xs text-blue-500 font-medium">{{ playerStats.avgAssists?.toFixed(1) || '0' }}</span>
        </div>
      </div>

      <!-- 胜率 (非机器人) -->
      <div v-if="playerStats && !player.isBot" class="flex items-center gap-0.5 flex-shrink-0">
        <span class="text-xs font-bold" :class="getWinRateColor(playerStats.winRate)">
          {{ playerStats.winRate?.toFixed(0) }}%
        </span>
        <span class="text-[9px] text-muted-foreground">({{ playerStats.totalGames }}场)</span>
      </div>

      <!-- 机器人提示 (仅机器人显示) -->
      <div v-if="player.isBot" class="flex items-center gap-1 flex-shrink-0 ml-auto">
        <span class="text-xs text-muted-foreground">电脑玩家</span>
      </div>

      <!-- 常用英雄 (非机器人) -->
      <div v-else-if="playerStats?.favoriteChampions?.length" class="flex gap-0.5 flex-shrink-0 ml-auto">
        <div
          v-for="champ in playerStats.favoriteChampions.slice(0, 3)"
          :key="champ.championId"
          class="relative w-6 h-6 rounded overflow-hidden ring-1 ring-border/40"
          :title="`${getChampionName(champ.championId)} (${champ.gamesPlayed}场, ${champ.winRate}%)`"
        >
          <img
            :src="getChampionIconUrl(champ.championId)"
            :alt="getChampionName(champ.championId)"
            class="w-full h-full object-cover"
          />
        </div>
      </div>
    </div>

    <!-- 历史战绩：水平一行显示（非机器人） -->
    <div v-if="playerStats?.recentPerformance?.length && !player.isBot">
      <div class="flex items-center gap-1 flex-wrap">
        <div
          v-for="(match, idx) in playerStats.recentPerformance.slice(0, 8)"
          :key="idx"
          class="relative flex items-center gap-1 px-1.5 py-0.5 rounded-md"
          style="width: 118px"
          :class="match.win ? 'bg-green-500/12' : 'bg-red-500/12'"
          :title="`${getQueueName(match.queueId)} - ${getChampionName(match.championId)} - ${match.win ? '胜利' : '失败'} ${match.kills}/${match.deaths}/${match.assists}`"
        >
          <!-- 对局类型标签 -->
          <div
            class="w-5 h-5 rounded-full text-[11px] font-bold text-white shadow-sm leading-none flex-shrink-0 flex items-center justify-center"
            :class="getQueueTypeColor(match.queueId)"
          >
            {{ getQueueTypeShortBadge(match.queueId) }}
          </div>

          <!-- 英雄头像 -->
          <div class="relative flex-shrink-0">
            <img
              v-if="match.championId"
              :src="getChampionIconUrl(match.championId)"
              :alt="getChampionName(match.championId)"
              class="w-7 h-7 rounded-sm object-cover"
            />
            <div v-else class="w-7 h-7 rounded-sm bg-muted flex items-center justify-center">
              <div class="w-2.5 h-2.5 bg-muted-foreground/20 rounded" />
            </div>
          </div>

          <!-- KDA -->
          <span
            class="text-[10px] font-medium flex-shrink-0"
            :class="match.win ? 'text-green-700 dark:text-green-300' : 'text-red-700 dark:text-red-300'"
          >
            {{ match.kills || 0 }}/{{ match.deaths || 0 }}/{{ match.assists || 0 }}
          </span>

          <!-- 胜负标识 -->
          <div class="w-2 h-2 rounded-full flex-shrink-0" :class="match.win ? 'bg-green-500' : 'bg-red-500'"></div>
        </div>
      </div>
    </div>

    <!-- 无战绩提示 -->
    <div v-else-if="!player.isBot" class="text-center py-1">
      <span class="text-xs text-muted-foreground">暂无战绩数据</span>
    </div>

    <!-- 机器人标识 -->
    <div v-if="player.isBot" class="text-center py-1">
      <span class="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded">🤖 机器人</span>
    </div>
  </div>

  <!-- 战术建议功能已移除，现在使用新的多位置分析系统 -->
</template>

<script setup lang="ts">
import { getChampionIconUrl, getChampionName, getSpellMeta, getQueueName } from '@/lib'

// 判断是否为排位赛
const props = defineProps<{ player: any; playerStats?: any; isLocal?: boolean; isAlly?: boolean }>()

const isRanked = computed(() => {
  // 420/440 为排位赛队列ID
  return (
    props.playerStats?.recentPerformance?.[0]?.queueId === 420 ||
    props.playerStats?.recentPerformance?.[0]?.queueId === 440
  )
})

function getPositionShort(pos: string) {
  // 常见位置英文缩写转中文
  switch (pos?.toUpperCase()) {
    case 'TOP':
      return '上单'
    case 'JUNGLE':
      return '打野'
    case 'MID':
      return '中单'
    case 'BOTTOM':
      return '下路'
    case 'SUPPORT':
      return '辅助'
    default:
      return pos
  }
}

defineEmits<{
  select: [player: any]
}>()

// 位置标签映射
const positionLabels: Record<string, string> = {
  TOP: '上',
  JUNGLE: '野',
  MIDDLE: '中',
  BOTTOM: 'AD',
  UTILITY: '辅'
}

const getPositionLabel = (position: string) => {
  return positionLabels[position?.toUpperCase()] || position
}

// 队列类型超短徽章 (用于紧凑布局)
const getQueueTypeShortBadge = (queueId: number): string => {
  switch (queueId) {
    case 420:
      return '单'
    case 440:
      return '组'
    case 450:
      return '乱'
    case 430:
    case 400:
      return '匹'
    case 900:
      return '火'
    case 1020:
      return '云'
    case 700:
      return '杯'
    case 1700:
      return '斗'
    default:
      return '?'
  }
}

// 队列类型颜色
const getQueueTypeColor = (queueId: number): string => {
  switch (queueId) {
    case 420:
    case 440:
      return 'bg-yellow-600' // 排位赛 - 金色
    case 450:
      return 'bg-blue-600' // 大乱斗 - 蓝色
    case 430:
    case 400:
      return 'bg-gray-600' // 匹配 - 灰色
    case 900:
      return 'bg-purple-600' // 无限火力 - 紫色
    case 1020:
      return 'bg-teal-600' // 云顶 - 青色
    case 700:
      return 'bg-red-600' // 冠军杯 - 红色
    case 1700:
      return 'bg-orange-600' // 斗魂竞技场 - 橙色
    default:
      return 'bg-gray-500' // 其他 - 浅灰色
  }
}

// 胜率颜色
const getWinRateColor = (winRate: number) => {
  if (winRate >= 60) return 'text-green-500'
  if (winRate >= 50) return 'text-yellow-500'
  return 'text-red-500'
}
</script>
