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
    <!-- 基础信息行 -->
    <div class="flex items-center gap-1.5 mb-1">
      <!-- 英雄头像 + 本人标识 -->
      <div class="relative flex-shrink-0">
        <div
          class="w-8 h-8 rounded-md overflow-hidden ring-1 ring-border/60 group-hover:ring-primary/40 transition-all relative"
        >
          <img
            v-if="player.championId"
            :src="getChampionIconUrl(player.championId)"
            :alt="getChampionName(player.championId)"
            class="w-full h-full object-cover"
          />
          <img
            v-else-if="player.championPickIntent"
            :src="getChampionIconUrl(player.championPickIntent)"
            :alt="getChampionName(player.championPickIntent)"
            class="w-full h-full object-cover opacity-50"
          />
          <div v-else class="w-full h-full bg-muted flex items-center justify-center">
            <div class="w-4 h-4 bg-muted-foreground/20 rounded" />
          </div>

          <div
            v-if="!player.championId && player.championPickIntent"
            class="absolute inset-0 flex items-center justify-center bg-black/30"
          >
            <span class="text-[8px] text-white font-bold">预选
            </span>
          </div>
        </div>
        <div
          v-if="isLocal"
          class="absolute -top-1 -right-1 bg-primary text-primary-foreground text-[8px] px-1 py-0.5 rounded-full font-bold z-10"
        >
          我
        </div>
        <div
          v-else-if="player.isBot"
          class="absolute -top-1 -right-1 bg-gray-500 text-white text-[8px] px-1 py-0.5 rounded-full font-bold z-10"
        >
          机器人
        </div>
      </div>

      <!-- 召唤师姓名和英雄 -->
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

      <!-- 平均KDA -->
      <div v-if="playerStats && !player.isBot" class="flex items-center gap-1 flex-shrink-0">
        <span class="text-[9px] text-muted-foreground">KDA</span>
        <div class="flex items-center gap-0.5">
          <span class="text-xs text-green-600 dark:text-green-400 font-medium">{{ playerStats.avgKills?.toFixed(1) || '0' }}</span>
          <span class="text-[10px] text-muted-foreground">/</span>
          <span class="text-xs text-red-600 dark:text-red-400 font-medium">{{ playerStats.avgDeaths?.toFixed(1) || '0' }}</span>
          <span class="text-[10px] text-muted-foreground">/</span>
          <span class="text-xs text-blue-600 dark:text-blue-400 font-medium">{{ playerStats.avgAssists?.toFixed(1) || '0' }}</span>
        </div>
      </div>

      <!-- 胜率 -->
      <div v-if="playerStats && !player.isBot" class="flex items-center gap-0.5 flex-shrink-0">
        <span class="text-xs font-bold" :class="getWinRateColor(playerStats.winRate)">
          {{ playerStats.winRate?.toFixed(0) }}%
        </span>
        <span class="text-[9px] text-muted-foreground">({{ playerStats.totalGames }}场)</span>
      </div>

      <!-- 常用英雄缩略 -->
      <div v-if="!player.isBot && playerStats?.favoriteChampions?.length" class="flex gap-0.5 flex-shrink-0 ml-auto">
        <div
          v-for="champ in playerStats.favoriteChampions.slice(0, 3)"
          :key="champ.championId"
          class="relative w-6 h-6 rounded overflow-hidden ring-1 ring-border/40"
          :title="`${getChampionName(champ.championId)} (${champ.gamesPlayed || champ.games || 0}场)`"
        >
          <img
            :src="getChampionIconUrl(champ.championId)"
            :alt="getChampionName(champ.championId)"
            class="w-full h-full object-cover"
          />
        </div>
      </div>
    </div>

    <!-- 内容区域 -->
    <div v-if="player.isBot" class="text-center py-1">
      <span class="text-xs text-muted-foreground bg-muted/60 dark:bg-muted/50 px-2 py-0.5 rounded">🤖 机器人，无需分析</span>
    </div>
    <div v-else class="mt-2">
      <Tabs v-model="activeTab" class="w-full">
        <TabsList class="grid w-full grid-cols-3 bg-accent/10 dark:bg-muted/30 rounded-md p-0.5">
          <TabsTrigger value="overview">概览</TabsTrigger>
          <TabsTrigger value="analysis">深度分析</TabsTrigger>
          <TabsTrigger value="advice" class="flex items-center justify-center gap-1">
            建议
            <span
              v-if="adviceList.length"
              :class="[
                'text-[10px] px-1.5 py-0.5 rounded-full',
                criticalAdviceCount
                  ? 'bg-destructive/20 dark:bg-destructive/30 text-destructive dark:text-destructive-foreground font-semibold'
                  : 'bg-primary/10 dark:bg-primary/20 text-primary dark:text-primary-foreground'
              ]"
            >
              {{ criticalAdviceCount || adviceList.length }}
            </span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="overview" class="space-y-2 mt-2">
          <div v-if="recentPerformance.length > 0" class="flex flex-col gap-1">
            <div
              v-for="(match, idx) in recentPerformance.slice(0, 8)"
              :key="idx"
              class="relative flex items-center gap-1 px-1.5 py-0.5 rounded-md"
              style="width: 118px"
              :class="[match.win ? 'bg-green-500/12 dark:bg-green-500/20' : 'bg-red-500/12 dark:bg-red-500/20']"
              :title="`${getQueueName(match.queueId)} - ${getChampionName(match.championId)} - ${match.win ? '胜利' : '失败'} ${match.kills}/${match.deaths}/${match.assists}`"
            >
              <div
                class="w-5 h-5 rounded-full text-[11px] font-bold text-white shadow-sm leading-none flex-shrink-0 flex items-center justify-center"
                :class="getQueueTypeColor(match.queueId)"
              >
                {{ getQueueTypeShortBadge(match.queueId) }}
              </div>
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
              <span
                class="text-[10px] font-medium flex-shrink-0"
                :class="match.win ? 'text-green-700 dark:text-green-300' : 'text-red-700 dark:text-red-300'"
              >
                {{ match.kills || 0 }}/{{ match.deaths || 0 }}/{{ match.assists || 0 }}
              </span>
              <div class="w-2 h-2 rounded-full flex-shrink-0" :class="match.win ? 'bg-green-500' : 'bg-red-500'"></div>
            </div>
          </div>
          <div v-else class="text-center text-xs text-muted-foreground py-2">暂无近期战绩</div>
        </TabsContent>

        <TabsContent value="analysis" class="space-y-3 mt-2">
          <div v-if="analysisChampions.length" class="space-y-1">
            <h4 class="text-xs font-semibold text-muted-foreground">常用英雄</h4>
            <div class="space-y-1">
              <div
                v-for="champ in analysisChampions.slice(0, 5)"
                :key="champ.championId"
                class="flex items-center justify-between rounded border border-border/40 px-2 py-1"
              >
                <div class="flex items-center gap-2">
                  <img
                    :src="getChampionIconUrl(champ.championId)"
                    :alt="champ.championName || getChampionName(champ.championId)"
                    class="w-6 h-6 rounded"
                  />
                  <div class="flex flex-col">
                    <span class="text-xs font-medium">{{ champ.championName || getChampionName(champ.championId) }}</span>
                    <span class="text-[10px] text-muted-foreground">{{ champ.games }}场</span>
                  </div>
                </div>
                <div class="text-right">
                  <span class="text-xs font-semibold" :class="champ.winRate >= 55 ? 'text-green-600 dark:text-green-400' : champ.winRate >= 50 ? 'text-blue-600 dark:text-blue-400' : 'text-red-600 dark:text-red-400'">
                    {{ champ.winRate.toFixed(0) }}%
                  </span>
                  <div class="text-[10px] text-muted-foreground">{{ champ.wins }}胜</div>
                </div>
              </div>
            </div>
          </div>

          <div v-if="topTraits.length" class="space-y-1">
            <h4 class="text-xs font-semibold text-muted-foreground">特征标签</h4>
            <div class="grid grid-cols-2 gap-1">
              <div
                v-for="trait in topTraits.slice(0, 4)"
                :key="trait.name"
                class="rounded border border-dashed border-border/40 px-2 py-1 space-y-0.5"
              >
                <div class="text-xs font-semibold text-foreground">{{ trait.name }}</div>
                <div class="text-[10px] text-muted-foreground leading-snug">{{ trait.description }}</div>
                <div class="text-[10px] text-primary/80">评分 {{ trait.score.toFixed(1) }}</div>
              </div>
            </div>
          </div>

          <div v-if="!analysisChampions.length && !topTraits.length" class="text-center text-xs text-muted-foreground py-2">
            暂无深度分析数据
          </div>
        </TabsContent>

        <TabsContent value="advice" class="space-y-2 mt-2">
          <div v-if="hasAdvice" class="space-y-2">
            <div
              v-for="(advice, idx) in adviceList"
              :key="idx"
              class="p-2 rounded border border-border/50 bg-background/60 dark:bg-background/45 space-y-1"
            >
              <div class="flex items-start justify-between gap-2">
                <div class="flex flex-col">
                  <span class="text-xs font-semibold text-foreground">{{ advice.title }}</span>
                  <span class="text-[10px] text-muted-foreground">{{ formatAdviceCategory(advice.category) }}</span>
                </div>
                <Badge :variant="getAdvicePriorityVariant(advice.priority)" class="text-[10px]">
                  优先级 {{ advice.priority }}
                </Badge>
              </div>
              <p class="text-[10px] text-muted-foreground leading-snug">{{ advice.problem }}</p>
              <ul v-if="advice.suggestions?.length" class="space-y-0.5">
                <li v-for="(suggestion, sidx) in advice.suggestions" :key="sidx" class="flex items-start gap-1 text-[10px] text-foreground/90 dark:text-foreground/80">
                  <span class="text-primary dark:text-primary/80 mt-0.5">•</span>
                  <span class="leading-snug">{{ suggestion }}</span>
                </li>
              </ul>
              <p v-else class="text-[10px] text-muted-foreground">暂无具体建议，后端正在收集中...</p>
            </div>
          </div>
          <div v-else class="text-center text-xs text-muted-foreground py-2">
            暂无建议数据，尝试触发深度分析以获取更多洞察
          </div>
        </TabsContent>
      </Tabs>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { Badge } from '@/components/ui/badge'
import { getChampionIconUrl, getChampionName, getSpellMeta, getQueueName } from '@/lib'

const props = defineProps<{ player: any; playerStats?: any; isLocal?: boolean; isAlly?: boolean }>()

const emit = defineEmits<{ select: [player: any] }>()

const activeTab = ref('overview')

const adviceList = computed(() => props.playerStats?.advice ?? [])
const hasAdvice = computed(() => adviceList.value.length > 0)
const criticalAdviceCount = computed(() => adviceList.value.filter((item: any) => item?.priority >= 4).length)
const analysisChampions = computed(() => props.playerStats?.favoriteChampions ?? [])
const topTraits = computed(() => props.playerStats?.traits ?? [])
const recentPerformance = computed(() => props.playerStats?.recentPerformance ?? [])

const isRanked = computed(() => {
  const queueId = recentPerformance.value?.[0]?.queueId
  return queueId === 420 || queueId === 440
})

function getPositionShort(pos: string) {
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

const getQueueTypeColor = (queueId: number): string => {
  switch (queueId) {
    case 420:
    case 440:
      return 'bg-yellow-600'
    case 450:
      return 'bg-blue-600'
    case 430:
    case 400:
      return 'bg-gray-600'
    case 900:
      return 'bg-purple-600'
    case 1020:
      return 'bg-teal-600'
    case 700:
      return 'bg-red-600'
    case 1700:
      return 'bg-orange-600'
    default:
      return 'bg-gray-500'
  }
}

const getWinRateColor = (winRate: number) => {
  if (winRate >= 60) return 'text-green-600 dark:text-green-400'
  if (winRate >= 50) return 'text-blue-600 dark:text-blue-400'
  return 'text-red-600 dark:text-red-400'
}

const adviceCategoryLabels: Record<string, string> = {
  Laning: '对线',
  Farming: '补刀',
  Teamfight: '团战',
  Vision: '视野',
  Positioning: '站位',
  Decision: '决策',
  Champion: '英雄'
}

const formatAdviceCategory = (category: string) => {
  return adviceCategoryLabels[category] || category
}

const getAdvicePriorityVariant = (priority: number) => {
  if (priority >= 4) return 'destructive'
  if (priority >= 3) return 'default'
  return 'secondary'
}
</script>
