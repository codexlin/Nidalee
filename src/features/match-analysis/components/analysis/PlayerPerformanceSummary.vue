<script setup lang="ts">
import { computed } from 'vue'
import { gradeFromKda, gradeTextClass } from '@/shared/utils/matchGrade'

const props = defineProps<{
  analysis: PlayerAnalysisResult
}>()

const stats = computed(() => props.analysis.stats)
const rankedRating = computed(() => props.analysis.ranked?.rating)
const isRanked = computed(() => props.analysis.depth === 'ranked')

const ratingGrade = computed(() => {
  const grade = rankedRating.value?.grade
  if (grade === 'sPlus') return 'S+'
  if (grade) return grade.toUpperCase()
  return isRanked.value ? '—' : gradeFromKda(stats.value.avgKda)
})

const confidenceLabel = computed(() => {
  if (props.analysis.basis.confidence === 'high') return '高可信'
  if (props.analysis.basis.confidence === 'medium') return '中可信'
  return '低可信'
})

const confidenceClass = computed(() => {
  if (props.analysis.basis.confidence === 'high') return 'bg-emerald-500/80'
  if (props.analysis.basis.confidence === 'medium') return 'bg-amber-500/80'
  return 'bg-rose-500/80'
})

function winRateClass(winRate: number): string {
  if (winRate > 50) return 'text-emerald-500'
  if (winRate < 50) return 'text-rose-500'
  return 'text-foreground'
}
</script>

<template>
  <section class="space-y-1">
    <div class="grid grid-cols-3 divide-x divide-border/40 rounded-md bg-muted/20 py-1">
      <div class="min-w-0 px-1.5">
        <div class="text-[8px] leading-3 text-muted-foreground">胜率</div>
        <div class="text-xs font-bold leading-4 tabular-nums" :class="winRateClass(stats.winRate)">
          {{ stats.winRate.toFixed(0) }}%
          <span class="text-[8px] font-normal text-muted-foreground">{{ stats.totalGames }}场</span>
        </div>
      </div>
      <div class="min-w-0 px-1.5">
        <div class="text-[8px] leading-3 text-muted-foreground">KDA</div>
        <div class="text-xs font-bold leading-4 tabular-nums">{{ stats.avgKda.toFixed(2) }}</div>
      </div>
      <div class="min-w-0 px-1.5">
        <div class="text-[8px] leading-3 text-muted-foreground">
          {{ isRanked ? '竞技评级' : '近期表现' }}
        </div>
        <Tooltip v-if="rankedRating">
          <TooltipTrigger as-child>
            <div class="flex cursor-help items-baseline gap-1 leading-4">
              <span class="text-xs font-black" :class="gradeTextClass(ratingGrade)">{{ ratingGrade }}</span>
              <span class="truncate text-[9px] text-muted-foreground">{{ rankedRating.label }}</span>
            </div>
          </TooltipTrigger>
          <TooltipContent class="max-w-72 text-xs">{{ rankedRating.summary }}</TooltipContent>
        </Tooltip>
        <div v-else class="text-xs font-black leading-4" :class="gradeTextClass(ratingGrade)">
          {{ ratingGrade }}
        </div>
      </div>
    </div>

    <Tooltip v-if="isRanked">
      <TooltipTrigger as-child>
        <div
          class="inline-flex max-w-full cursor-help items-center gap-1 rounded-md bg-muted/15 px-1.5 py-px text-[9px] text-muted-foreground"
        >
          <i class="size-1 flex-none rounded-full" :class="confidenceClass" />
          <span class="truncate">当前排位样本 {{ analysis.basis.primaryGames }} 场 · {{ confidenceLabel }}</span>
        </div>
      </TooltipTrigger>
      <TooltipContent class="max-w-72 text-xs">
        仅使用当前排位队列样本；样本不足时保持低可信，不混入其他模式。
      </TooltipContent>
    </Tooltip>
  </section>
</template>
