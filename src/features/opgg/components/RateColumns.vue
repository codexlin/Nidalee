<template>
  <div class="flex items-center justify-end gap-3 tabular-nums">
    <span class="w-12 text-right text-sm font-medium text-sky-600 dark:text-sky-400">
      {{ formatPct(winRate) }}
    </span>
    <span class="hidden w-12 text-right text-sm text-muted-foreground sm:inline">
      {{ formatPct(pickRate) }}
    </span>
    <span class="hidden w-14 text-right text-sm text-muted-foreground md:inline">
      {{ formatPlay(games) }}
    </span>
    <slot />
  </div>
</template>

<script setup lang="ts">
defineProps<{
  winRate?: number | null
  pickRate?: number | null
  games?: number | null
}>()

const formatPct = (v?: number | null) => {
  if (v === null || v === undefined || Number.isNaN(v)) return '—'
  return `${(v * 100).toFixed(1)}%`
}

const formatPlay = (n?: number | null) => {
  if (n === null || n === undefined) return '—'
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return String(n)
}
</script>
