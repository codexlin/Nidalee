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
    <div class="flex items-start gap-1.5">
      <div class="flex-1">
        <div class="flex items-center gap-1.5 mb-1">
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
                <span class="text-[8px] text-white font-bold">预选</span>
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
                <div
                  v-if="isRanked && player.position"
                  class="mt-0.5 text-[9px] font-semibold text-primary/80 text-center"
                >
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
                class="inline-flex items-center gap-0.5 text-[8px] px-0.5 py-0 bg-muted/50 rounded text-muted-foreground flex-shrink-0"
              >
                <img
                  v-if="getRoleIconUrl(player.assignedPosition)"
                  :src="getRoleIconUrl(player.assignedPosition)"
                  alt=""
                  class="h-2.5 w-2.5 object-contain opacity-80"
                />
                {{ getPositionLabel(player.assignedPosition) }}
              </span>
            </div>
          </div>
          <div class="h-8 w-px bg-border flex-shrink-0" />
          <div class="flex gap-0.5 flex-shrink-0">
            <div
              v-for="(spellId, spellIdx) in [player.spell1Id, player.spell2Id]"
              :key="spellIdx"
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
          <div v-if="playerStats && !player.isBot" class="flex items-center gap-1 flex-shrink-0">
            <span class="text-[9px] text-muted-foreground">KDA</span>
            <div class="flex items-center gap-0.5">
              <span class="text-xs text-green-600 dark:text-green-400 font-medium">{{
                playerStats.avgKills?.toFixed(1) || '0'
              }}</span>
              <span class="text-[10px] text-muted-foreground">/</span>
              <span class="text-xs text-red-600 dark:text-red-400 font-medium">{{
                playerStats.avgDeaths?.toFixed(1) || '0'
              }}</span>
              <span class="text-[10px] text-muted-foreground">/</span>
              <span class="text-xs text-blue-600 dark:text-blue-400 font-medium">{{
                playerStats.avgAssists?.toFixed(1) || '0'
              }}</span>
            </div>
          </div>
          <div v-if="playerStats && !player.isBot" class="flex items-center gap-0.5 flex-shrink-0">
            <span class="text-xs font-bold" :class="getWinRateColor(playerStats.winRate)">
              {{ playerStats.winRate?.toFixed(0) }}%
            </span>
            <span class="text-[9px] text-muted-foreground">({{ playerStats.totalGames }}场)</span>
          </div>
          <div class="ml-auto flex items-center gap-1">
            <HoverCard v-if="hasAdvice">
              <HoverCardTrigger as-child>
                <Button
                  size="icon"
                  variant="ghost"
                  class="h-7 w-7 text-amber-500 hover:text-amber-500/90 hover:bg-amber-500/10"
                  @click.stop
                >
                  <Lightbulb class="h-4 w-4" />
                </Button>
              </HoverCardTrigger>
              <HoverCardContent class="max-w-xs space-y-2">
                <div v-for="(advice, idx) in adviceList.slice(0, 4)" :key="idx" class="space-y-1">
                  <div class="flex items-start justify-between gap-2">
                    <div class="text-xs font-semibold text-foreground leading-tight">{{ advice.title }}</div>
                    <Badge :variant="getAdvicePriorityVariant(advice.priority)" class="text-[10px]">
                      P{{ advice.priority }}
                    </Badge>
                  </div>
                  <p class="text-[10px] text-muted-foreground leading-snug">{{ advice.problem }}</p>
                </div>
                <div v-if="adviceList.length > 4" class="text-[10px] text-muted-foreground">
                  还有 {{ adviceList.length - 4 }} 条建议...
                </div>
              </HoverCardContent>
            </HoverCard>
          </div>
        </div>

        <div v-if="player.isBot" class="mt-2 text-center py-1">
          <span class="text-xs text-muted-foreground bg-muted/60 dark:bg-muted/50 px-2 py-0.5 rounded"
            >🤖 机器人，无需分析</span
          >
        </div>
        <div v-else class="mt-2 space-y-3">
          <div v-if="recentPerformance.length > 0" class="flex flex-col gap-1">
            <div class="text-xs font-semibold text-muted-foreground">近期战绩</div>
            <div class="flex flex-wrap gap-1">
              <div
                v-for="(match, idx) in recentPerformance.slice(0, 6)"
                :key="idx"
                class="relative flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[10px]"
                :class="[match.win ? 'bg-green-500/12 dark:bg-green-500/20' : 'bg-red-500/12 dark:bg-red-500/20']"
                :title="`${getQueueName(match.queueId ?? 0)} - ${getChampionName(match.championId)} - ${match.win ? '胜利' : '失败'} ${match.kills}/${match.deaths}/${match.assists}`"
              >
                <div
                  class="w-4.5 h-4.5 rounded-full text-[9px] font-bold text-white leading-none flex-shrink-0 flex items-center justify-center"
                  :class="getQueueTypeColor(match.queueId ?? 0)"
                >
                  {{ getQueueTypeShortBadge(match.queueId ?? 0) }}
                </div>
                <img
                  v-if="match.championId"
                  :src="getChampionIconUrl(match.championId)"
                  :alt="getChampionName(match.championId)"
                  class="w-5 h-5 rounded-sm object-cover"
                />
                <span class="font-medium">{{ match.kills || 0 }}/{{ match.deaths || 0 }}/{{ match.assists || 0 }}</span>
              </div>
            </div>
          </div>

          <div v-if="analysisChampions.length" class="space-y-1">
            <div class="text-xs font-semibold text-muted-foreground">常用英雄</div>
            <div class="space-y-1">
              <div
                v-for="champ in analysisChampions.slice(0, 3)"
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
                    <span class="text-xs font-medium">{{
                      champ.championName || getChampionName(champ.championId)
                    }}</span>
                    <span class="text-[10px] text-muted-foreground">{{ champ.games }}场</span>
                  </div>
                </div>
                <div class="text-right">
                  <span
                    class="text-xs font-semibold"
                    :class="
                      champ.winRate >= 55
                        ? 'text-green-600 dark:text-green-400'
                        : champ.winRate >= 50
                          ? 'text-blue-600 dark:text-blue-400'
                          : 'text-red-600 dark:text-red-400'
                    "
                  >
                    {{ champ.winRate.toFixed(0) }}%
                  </span>
                  <div class="text-[10px] text-muted-foreground">{{ champ.wins }}胜</div>
                </div>
              </div>
            </div>
          </div>

          <div v-if="topTraits.length" class="space-y-1">
            <div class="text-xs font-semibold text-muted-foreground">特征标签</div>
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

          <div
            v-if="!analysisChampions.length && !topTraits.length && adviceList.length === 0"
            class="text-center text-xs text-muted-foreground py-2"
          >
            暂无深度分析数据
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Button } from '@/components/ui/button'
import { HoverCard, HoverCardContent, HoverCardTrigger } from '@/components/ui/hover-card'
import { Badge } from '@/components/ui/badge'
import { Lightbulb } from 'lucide-vue-next'
import { getChampionIconUrl, getChampionName, getRoleIconUrl, getSpellMeta, getQueueName } from '@/lib'
import type { UIPlayerData } from '@/types/match-analysis'

/** 对局分析玩家展示：基于 UIPlayerData，兼容选人阶段的 assignedPosition */
type CompactPlayer = UIPlayerData & { assignedPosition?: string | null }

const props = defineProps<{
  player: CompactPlayer
  playerStats?: PlayerMatchStats | null
  isLocal?: boolean
  isAlly?: boolean
}>()

const emit = defineEmits<{ select: [player: CompactPlayer] }>()

const adviceList = computed(() => props.playerStats?.advice ?? [])
const hasAdvice = computed(() => adviceList.value.length > 0)
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

const getAdvicePriorityVariant = (priority: number) => {
  if (priority >= 4) return 'destructive'
  if (priority >= 3) return 'default'
  return 'secondary'
}
</script>
