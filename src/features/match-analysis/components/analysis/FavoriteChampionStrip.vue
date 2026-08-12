<script setup lang="ts">
import { getChampionIconUrl, resolveChampionName } from '@/lib'

defineProps<{
  champions: AnalysisChampionStats[]
}>()
</script>

<template>
  <section class="flex flex-col gap-1">
    <div class="flex items-center justify-between gap-2">
      <h4 class="text-xs font-medium text-muted-foreground">常用英雄</h4>
      <span class="text-[9px] text-muted-foreground">样本内选择</span>
    </div>
    <div class="grid grid-cols-3 gap-1.5">
      <div
        v-for="champion in champions.slice(0, 3)"
        :key="champion.championId"
        class="flex min-w-0 items-center gap-1.5 rounded-lg bg-muted/25 px-1.5 py-1"
      >
        <img
          :src="getChampionIconUrl(champion.championId)"
          :alt="resolveChampionName(champion.championId, champion.championName)"
          class="size-6 flex-none rounded-md object-cover"
        />
        <div class="min-w-0 flex-1">
          <div class="truncate text-[10px] font-medium text-foreground">
            {{ resolveChampionName(champion.championId, champion.championName) }}
          </div>
          <div class="text-[9px] text-muted-foreground tabular-nums">
            {{ champion.games }} 场 · {{ champion.winRate.toFixed(0) }}%
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
