<script setup lang="ts">
import { computed } from 'vue'
import { getCompactQueueDisplayName } from '@/common/queueCatalog'
import { formatDuration, formatGameMode, getChampionIconUrl, resolveChampionName } from '@/lib'
import { displayGrade, gradeTextClass } from '@/shared/utils/matchGrade'

const props = defineProps<{
  matches: MatchPerformance[]
  view: 'recent' | 'sample'
  scopeLabel: string
  recentCount: number
  sampleCount: number
  showScopeSwitch?: boolean
}>()

const emit = defineEmits<{
  'update:view': [view: 'recent' | 'sample']
}>()

const visibleMatches = computed(() => props.matches.slice(0, 6))

function getModeLabel(match: MatchPerformance): string {
  if (match.queueId !== undefined) return getCompactQueueDisplayName(match.queueId)
  if (match.gameMode) return formatGameMode(match.gameMode)
  return '未知模式'
}
</script>

<template>
  <section class="flex flex-col gap-1">
    <div class="flex min-w-0 items-center gap-1.5">
      <h4 class="flex-none text-xs font-bold text-foreground">对局记录</h4>
      <div
        v-if="showScopeSwitch"
        class="flex flex-none rounded-md border border-border/50 bg-muted/15 p-px"
        aria-label="对局记录范围"
      >
        <button
          type="button"
          class="rounded px-1.5 py-0.5 text-[9px] leading-none transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          :class="
            view === 'recent' ? 'bg-foreground/10 text-foreground' : 'text-muted-foreground hover:text-foreground'
          "
          :aria-pressed="view === 'recent'"
          :disabled="recentCount === 0"
          title="按时间倒序显示全部模式中的近期有效对局（不含重开局）"
          @click.stop="emit('update:view', 'recent')"
        >
          最近
        </button>
        <button
          type="button"
          class="rounded px-1.5 py-0.5 text-[9px] leading-none transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
          :class="
            view === 'sample' ? 'bg-foreground/10 text-foreground' : 'text-muted-foreground hover:text-foreground'
          "
          :aria-pressed="view === 'sample'"
          :disabled="sampleCount === 0"
          title="显示用于计算排位表现、位置画像和竞技评级的当前排位样本"
          @click.stop="emit('update:view', 'sample')"
        >
          参考样本
        </button>
      </div>
      <p class="min-w-0 flex-1 truncate text-right text-[9px] text-muted-foreground">
        {{ scopeLabel }} · {{ visibleMatches.length }} 场
      </p>
    </div>

    <div v-if="visibleMatches.length" class="grid grid-cols-2 gap-1">
      <article
        v-for="match in visibleMatches"
        :key="match.gameId ?? `${match.gameCreation}-${match.championId}`"
        class="surface-inset relative flex min-w-0 items-center gap-1 overflow-hidden border-l-2 py-0.5 pr-1 pl-0.5"
        :class="match.win ? 'border-l-emerald-500/80' : 'border-l-rose-500/80'"
      >
        <img
          :src="getChampionIconUrl(match.championId)"
          :alt="resolveChampionName(match.championId, match.championName)"
          class="relative z-10 size-6 flex-none rounded object-cover"
        />
        <div class="relative z-10 min-w-0 flex-1">
          <div class="flex min-w-0 items-baseline gap-1 leading-3.5 tabular-nums">
            <span class="min-w-0 flex-1 truncate text-[9px] font-semibold">
              <span class="text-red-500">{{ match.kills }}</span>
              <span class="text-muted-foreground/50">/</span>
              <span class="text-muted-foreground">{{ match.deaths }}</span>
              <span class="text-muted-foreground/50">/</span>
              <span class="text-blue-500">{{ match.assists }}</span>
            </span>
            <span v-if="match.gameDuration" class="flex-none font-mono text-[8px] text-muted-foreground">
              {{ formatDuration(match.gameDuration) }}
            </span>
          </div>
          <div class="flex min-w-0 items-center gap-1 leading-3.5">
            <span class="min-w-0 flex-1 truncate text-[8px] text-muted-foreground" :title="getModeLabel(match)">
              {{ getModeLabel(match) }}
            </span>
            <span
              class="ml-auto flex-none rounded border border-border/50 bg-background/50 px-1 py-px text-[8px] font-bold leading-none"
              :class="gradeTextClass(displayGrade(match))"
            >
              {{ displayGrade(match) }}
            </span>
          </div>
        </div>
      </article>
    </div>
    <p v-else class="py-2 text-center text-[9px] text-muted-foreground">该范围暂无可显示的对局</p>
  </section>
</template>
