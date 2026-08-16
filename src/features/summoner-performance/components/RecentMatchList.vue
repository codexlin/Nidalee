<template>
  <div v-if="games.length" class="space-y-4">
    <div class="space-y-1">
      <h4 class="text-base font-semibold flex items-center">
        <Calendar class="h-5 w-5 mr-2 text-muted-foreground" />
        最近对局
      </h4>
      <p class="text-xs text-muted-foreground">右下角为自研评级（S+～D）。点击查看详情。</p>
    </div>
    <div class="grid gap-2" style="grid-template-columns: repeat(auto-fit, minmax(240px, 1fr))">
      <div
        v-for="game in visibleGames"
        :key="game.gameId ?? `${game.gameCreation}-${game.championId}`"
        class="surface-inset-interactive group relative flex cursor-pointer overflow-hidden"
        @click="emit('open-game-detail', game)"
      >
        <div :class="game.win ? 'bg-emerald-600' : 'bg-rose-600'" class="w-1 shrink-0"></div>
        <span
          class="pointer-events-none absolute -right-1 -bottom-2 z-0 select-none font-black leading-none tabular-nums -rotate-12 origin-bottom-right"
          :class="[gradeWatermarkSizeClass(displayGrade(game)), gradeWatermarkClass(displayGrade(game))]"
          aria-hidden="true"
        >
          {{ displayGrade(game) }}
        </span>
        <div class="relative z-10 flex-1 p-3">
          <div class="flex items-center justify-between mb-2">
            <div class="flex items-center gap-2 min-w-0">
              <img
                v-if="game.championId"
                :src="getChampionIconUrl(game.championId)"
                alt=""
                class="h-9 w-9 shrink-0 rounded-full border-2 border-primary/20"
              />
              <span class="font-semibold text-sm truncate">{{
                resolveChampionName(game.championId, game.championName)
              }}</span>
            </div>
            <div class="flex items-center gap-2 shrink-0">
              <span class="flex items-center gap-1 text-xs text-muted-foreground tabular-nums">
                <Timer class="w-3 h-3 shrink-0" />
                {{ formatGameTime(game.gameDuration ?? 0) }}
              </span>
              <span
                class="h-5 px-1.5 inline-flex items-center rounded-md text-xs font-medium text-white"
                :class="game.win ? 'bg-emerald-600' : 'bg-rose-600'"
              >
                {{ game.win ? '胜' : '负' }}
              </span>
            </div>
          </div>
          <div class="pr-6">
            <span class="font-mono font-bold text-base tabular-nums leading-none">
              <span class="text-red-500">{{ game.kills }}</span>
              <span class="text-muted-foreground/50">/</span>
              <span class="text-muted-foreground">{{ game.deaths }}</span>
              <span class="text-muted-foreground/50">/</span>
              <span class="text-blue-500">{{ game.assists }}</span>
            </span>
            <div class="flex items-center gap-2 mt-2 text-xs text-muted-foreground">
              <span class="flex items-center gap-1">
                <Clock class="w-3 h-3 shrink-0" />
                {{ formatRelativeTime(game.gameCreation ?? 0) }}
              </span>
              <span class="text-border">·</span>
              <span class="truncate">{{ getQueueName(game.queueId ?? 0) }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
    <div v-if="games.length > showCount" class="flex justify-center mt-4">
      <FloatIconButton variant="pill" title="加载更多" @click="emit('load-more')"> 加载更多 </FloatIconButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getChampionIconUrl, getQueueName, resolveChampionName } from '@/lib'
import FloatIconButton from '@/components/common/FloatIconButton.vue'
import { Calendar, Clock, Timer } from 'lucide-vue-next'
import { computed } from 'vue'
import { displayGrade, gradeWatermarkClass, gradeWatermarkSizeClass } from '@/shared/utils/matchGrade'

const props = defineProps<{
  games: MatchPerformance[]
  showCount: number
}>()

const emit = defineEmits<{
  (e: 'load-more'): void
  (e: 'open-game-detail', game: MatchPerformance): void
}>()

const { formatGameTime, formatRelativeTime } = useFormatters()

const visibleGames = computed(() => props.games.slice(0, props.showCount))
</script>
