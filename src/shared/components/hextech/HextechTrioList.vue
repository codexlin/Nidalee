<script setup lang="ts">
import { ChevronRight } from 'lucide-vue-next'
import {
  formatHextechGames,
  formatHextechPct,
  type HextechGuideAugment,
  type HextechGuideTrio,
  type HextechRarityKey,
  normalizeRarityKey
} from '@/shared/hextech/guideAugment'

const OVERLAY_TRIO_LIMIT = 3

const props = withDefaults(
  defineProps<{
    trios: HextechGuideTrio[]
    variant?: 'overlay' | 'workbench'
  }>(),
  { variant: 'workbench' }
)

const compact = computed(() => props.variant === 'overlay')
const shownTrios = computed(() => (compact.value ? props.trios.slice(0, OVERLAY_TRIO_LIMIT) : props.trios))

const rarityRing: Record<HextechRarityKey, string> = {
  prismatic: 'ring-rose-400/50',
  gold: 'ring-amber-400/50',
  silver: 'ring-slate-400/40',
  other: 'ring-white/15'
}

const ringFor = (item: HextechGuideAugment) => rarityRing[normalizeRarityKey(item.rarityName, item.rarityDisplayName)]

const fadeImg = (event: Event) => {
  const el = event.target as HTMLImageElement
  el.style.opacity = '0.3'
}

const routeTitle = (trio: HextechGuideTrio) => `${routeNames(trio)} · ${formatHextechPct(trio.winRate)} 胜率`

const routeNames = (trio: HextechGuideTrio) => trio.augments.map((item) => item.name).join(' → ')

const trioKey = (trio: HextechGuideTrio, index: number) =>
  trio.augments.map((item) => item.id).join('-') || `trio-${index}`
</script>

<template>
  <div v-if="compact && shownTrios.length" class="flex flex-col gap-1">
    <article
      v-for="(trio, index) in shownTrios"
      :key="trioKey(trio, index)"
      class="rounded-lg px-2 py-1.5"
      :title="routeTitle(trio)"
    >
      <div class="flex items-center gap-2">
        <div class="flex shrink-0 items-center">
          <template v-for="(item, slot) in trio.augments" :key="item.id">
            <ChevronRight v-if="slot > 0" class="size-3 shrink-0 text-white/20" />
            <img
              v-if="item.iconUrl"
              :src="item.iconUrl"
              :alt="item.name"
              class="size-6 rounded-md ring-1 ring-inset"
              :class="ringFor(item)"
              @error="fadeImg"
            />
            <span v-else class="size-6 rounded-md bg-white/10" />
          </template>
        </div>
        <p class="ml-auto w-12 shrink-0 text-right text-[12px] tabular-nums text-muted-foreground">
          {{ formatHextechPct(trio.winRate) }}
        </p>
      </div>
      <p class="mt-1 truncate text-[11px] leading-tight text-foreground/80">{{ routeNames(trio) }}</p>
    </article>
  </div>

  <div v-else class="flex flex-col gap-1.5">
    <article
      v-for="(trio, index) in shownTrios"
      :key="trioKey(trio, index)"
      class="surface-inset flex items-center gap-2 rounded-xl px-2.5 py-2"
      :title="routeTitle(trio)"
    >
      <div class="flex min-w-0 flex-1 items-center">
        <template v-for="(item, slot) in trio.augments" :key="item.id">
          <ChevronRight v-if="slot > 0" class="mx-0.5 size-3.5 shrink-0 text-muted-foreground/45" />
          <div class="flex min-w-0 flex-1 flex-col items-center gap-1">
            <img
              v-if="item.iconUrl"
              :src="item.iconUrl"
              :alt="item.name"
              class="rounded-md ring-1 ring-inset"
              :class="[ringFor(item), index === 0 ? 'size-10' : 'size-8']"
              @error="fadeImg"
            />
            <span
              class="line-clamp-2 w-full text-center leading-tight text-foreground"
              :class="index === 0 ? 'text-xs font-medium' : 'text-[11px]'"
            >
              {{ item.name }}
            </span>
          </div>
        </template>
      </div>
      <p class="hidden shrink-0 items-center justify-end gap-3 text-sm tabular-nums sm:flex">
        <span class="w-12 text-right font-medium text-sky-600 dark:text-sky-400">{{
          formatHextechPct(trio.winRate)
        }}</span>
        <span class="w-12 text-right text-muted-foreground">{{ formatHextechPct(trio.pickRate) }}</span>
        <span class="hidden w-14 text-right text-muted-foreground md:inline">
          {{ formatHextechGames(trio.games) }}
        </span>
      </p>
    </article>
  </div>
</template>
