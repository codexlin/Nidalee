<script setup lang="ts">
import { computed } from 'vue'
import { Shield } from 'lucide-vue-next'
import { getTierIconUrl } from '@/lib'
import { formatRankLabel } from '../../utils/playerPresentation'

const props = defineProps<{
  soloRank?: PlayerRankSummary | null
  flexRank?: PlayerRankSummary | null
}>()

const ranks = computed(() => [
  {
    key: 'solo',
    label: '单双',
    rank: props.soloRank
  },
  {
    key: 'flex',
    label: '灵活',
    rank: props.flexRank
  }
])

function rankDetail(rank?: PlayerRankSummary | null): string {
  if (!rank) return '未定级'
  return [
    rank.division,
    rank.leaguePoints === null || rank.leaguePoints === undefined ? null : `${rank.leaguePoints} LP`
  ]
    .filter(Boolean)
    .join(' · ')
}

function rankLabel(rank?: PlayerRankSummary | null): string {
  if (!rank) return '未定级'
  return formatRankLabel(rank.tier, rank.division, rank.leaguePoints)
}
</script>

<template>
  <div class="grid grid-cols-2 gap-1">
    <div
      v-for="rank in ranks"
      :key="rank.key"
      class="flex min-w-0 items-center gap-1.5 rounded-md border border-border/45 bg-muted/25 px-1 py-0.5"
      :title="`${rank.label}排位：${rankLabel(rank.rank)}`"
    >
      <div class="flex size-6 flex-none items-center justify-center">
        <img
          v-if="rank.rank"
          :src="getTierIconUrl(rank.rank.tier)"
          :alt="rankLabel(rank.rank)"
          class="size-7 max-w-none object-contain drop-shadow-sm"
        />
        <Shield v-else class="size-4 text-muted-foreground/60" />
      </div>
      <div class="min-w-0 leading-none">
        <div class="text-[8px] font-medium text-muted-foreground">{{ rank.label }}</div>
        <div class="mt-0.5 truncate text-[9px] font-semibold text-foreground tabular-nums">
          {{ rankDetail(rank.rank) }}
        </div>
      </div>
    </div>
  </div>
</template>
