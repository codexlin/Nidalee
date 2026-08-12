<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  stats: PlayerMatchStats
  rankedRating?: RankedPlayerRating | null
}>()

const visibleTraits = computed(() => props.stats.traits.slice(0, 3))

function traitIndicatorClass(type: string): string {
  if (type === 'good') return 'bg-emerald-500/80'
  if (type === 'bad') return 'bg-rose-500/80'
  return 'bg-muted-foreground/60'
}

function winRateClass(winRate: number): string {
  if (winRate > 50) return 'text-emerald-500'
  if (winRate < 50) return 'text-rose-500'
  return 'text-foreground'
}

const ratingGrade = computed(() => {
  const grade = props.rankedRating?.grade
  if (grade === 'sPlus') return 'S+'
  return grade?.toUpperCase() ?? '—'
})

function ratingClass(grade: RankedRatingGrade | undefined): string {
  if (grade === 'sPlus') return 'text-orange-500'
  if (grade === 's') return 'text-violet-400'
  if (grade === 'a') return 'text-emerald-500'
  if (grade === 'b') return 'text-sky-500'
  if (grade === 'c') return 'text-stone-400'
  if (grade === 'd') return 'text-rose-500'
  return 'text-muted-foreground'
}
</script>

<template>
  <section class="space-y-1.5">
    <div class="grid grid-cols-[0.85fr_0.75fr_1.4fr] divide-x divide-border/40 rounded-md bg-muted/20 py-1">
      <div class="min-w-0 px-1.5">
        <div class="text-[8px] leading-3 text-muted-foreground">胜率</div>
        <div class="text-xs font-bold leading-4 tabular-nums" :class="winRateClass(stats.winRate)">
          {{ stats.winRate.toFixed(0) }}%
          <span class="text-[8px] font-normal text-muted-foreground">{{ stats.totalGames }}场</span>
        </div>
      </div>
      <div class="min-w-0 px-1.5">
        <div class="text-[8px] leading-3 text-muted-foreground">KDA</div>
        <div class="text-xs font-bold leading-4 text-foreground tabular-nums">{{ stats.avgKda.toFixed(2) }}</div>
      </div>
      <div class="min-w-0 px-1.5">
        <div class="text-[8px] leading-3 text-muted-foreground">竞技评级</div>
        <Tooltip v-if="rankedRating">
          <TooltipTrigger as-child>
            <div class="flex cursor-help items-baseline gap-1 leading-4">
              <span class="text-xs font-black" :class="ratingClass(rankedRating.grade)">{{ ratingGrade }}</span>
              <span class="truncate text-[9px] font-medium text-muted-foreground">{{ rankedRating.label }}</span>
            </div>
          </TooltipTrigger>
          <TooltipContent class="max-w-72 text-xs">{{ rankedRating.summary }}</TooltipContent>
        </Tooltip>
        <div v-else class="text-xs font-bold leading-4 text-muted-foreground">—</div>
      </div>
    </div>

    <div v-if="visibleTraits.length" class="flex flex-wrap gap-1">
      <Tooltip v-for="trait in visibleTraits" :key="trait.name">
        <TooltipTrigger as-child>
          <span
            class="inline-flex items-center gap-1 rounded-md border border-border/45 bg-muted/20 px-1.5 py-px text-[10px] font-medium text-foreground/75"
          >
            <i class="size-1 rounded-full" :class="traitIndicatorClass(trait.type)" />
            {{ trait.name }}
          </span>
        </TooltipTrigger>
        <TooltipContent class="max-w-64 text-xs">{{ trait.description }}</TooltipContent>
      </Tooltip>
    </div>
  </section>
</template>
