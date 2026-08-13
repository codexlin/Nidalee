<script setup lang="ts">
import type { OverlayOffer } from '../composables/useAugmentOverlay'

const props = defineProps<{
  augment: OverlayOffer
  pending: boolean
}>()

const rarityClass = computed(() => {
  const rarity = props.augment.rarity.toLowerCase()
  if (rarity.includes('prism') || rarity.includes('彩') || rarity === '2') return 'ring-violet-400/70'
  if (rarity.includes('gold') || rarity.includes('金') || rarity === '1') return 'ring-amber-400/70'
  if (rarity.includes('silver') || rarity.includes('银') || rarity === '0') return 'ring-slate-400/50'
  return 'ring-border/60'
})

const formatPct = (value: number | null) => {
  if (value === null || Number.isNaN(value)) return '—'
  return `${(value * 100).toFixed(1)}%`
}
</script>

<template>
  <div
    class="flex min-w-0 flex-1 items-center gap-2.5 rounded-xl px-2.5 py-2 surface-overlay"
    :class="[
      augment.missing ? 'opacity-40' : rarityClass,
      augment.recommended ? 'ring-2 ring-amber-400/80' : '',
    ]"
  >
    <div class="flex size-11 shrink-0 items-center justify-center overflow-hidden rounded-md bg-muted/40 ring-1 ring-inset ring-border/50">
      <img
        v-if="!augment.missing && augment.iconUrl"
        :src="augment.iconUrl"
        :alt="augment.name"
        class="size-full object-cover"
      />
      <span v-else class="text-xs tabular-nums text-muted-foreground">{{
        String(augment.detectedSlot + 1).padStart(2, '0')
      }}</span>
    </div>
    <div class="min-w-0 flex-1">
      <p class="flex min-w-0 items-center gap-1.5 truncate text-sm font-medium leading-tight">
        <span class="truncate">{{ augment.missing ? '' : augment.name }}</span>
        <span
          v-if="augment.recommended"
          class="shrink-0 rounded-md bg-amber-400/20 px-1 py-px text-[10px] font-semibold text-amber-200"
        >
          推荐
        </span>
      </p>
      <p v-if="!augment.missing" class="mt-0.5 flex gap-3 text-xs tabular-nums text-muted-foreground">
        <span>
          胜率
          <strong class="ml-1 font-medium text-foreground">{{
            pending && augment.winRate === null ? '…' : formatPct(augment.winRate)
          }}</strong>
        </span>
        <span>
          选取
          <strong class="ml-1 font-medium text-foreground">{{
            pending && augment.pickRate === null ? '…' : formatPct(augment.pickRate)
          }}</strong>
        </span>
      </p>
    </div>
  </div>
</template>
