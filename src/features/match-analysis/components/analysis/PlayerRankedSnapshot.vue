<script setup lang="ts">
import { computed } from 'vue'
import { getPositionLabel } from '@/common/positionLabels'
import { getChampionIconUrl, getChampionName, getRoleIconUrl } from '@/lib'
import { buildRankedPositionSnapshot } from '../../utils/playerPresentation'

const props = defineProps<{
  ranked: RankedAnalysis
}>()

const profile = computed(() => props.ranked.positionProfile)
const positionSnapshot = computed(() => buildRankedPositionSnapshot(profile.value))
const primary = computed(() => positionSnapshot.value.primary)
const current = computed(() => positionSnapshot.value.current)
const currentPositionNote = computed(() => {
  if (!current.value) return '本局位置待确认'
  if (positionSnapshot.value.currentKind === 'same') return '本局同路'
  if (current.value.sample.games > 0) {
    return '本局 ' + getPositionLabel(current.value.position) + ' ' + current.value.sample.games + '场'
  }
  return '本局 ' + getPositionLabel(current.value.position) + ' 无近期样本'
})
const currentPositionClass = computed(() =>
  positionSnapshot.value.currentKind === 'same'
    ? 'text-emerald-500'
    : positionSnapshot.value.currentKind === 'different'
      ? 'text-amber-500'
      : 'text-muted-foreground'
)
const champion = computed(() => props.ranked.currentChampion)
</script>

<template>
  <section class="flex flex-col gap-1 text-[9px]">
    <div class="surface-inset flex min-w-0 items-center gap-1.5 px-1.5 py-1">
      <img
        v-if="primary && getRoleIconUrl(primary.position)"
        :src="getRoleIconUrl(primary.position)"
        alt=""
        class="size-5 flex-none object-contain opacity-85"
      />
      <div class="min-w-0 flex-1">
        <div class="truncate text-muted-foreground">最近位置</div>
        <div v-if="primary" class="truncate font-medium text-foreground tabular-nums">
          {{ getPositionLabel(primary.position) }} · {{ primary.sample.games }}场 ·
          {{ primary.sample.winRate.toFixed(0) }}%
        </div>
        <div v-else class="truncate text-muted-foreground">近期位置样本不足</div>
      </div>
      <span class="max-w-20 flex-none truncate font-medium" :class="currentPositionClass">
        {{ currentPositionNote }}
      </span>
    </div>

    <div class="surface-inset flex min-w-0 items-center gap-1.5 px-1.5 py-1">
      <img
        v-if="champion"
        :src="getChampionIconUrl(champion.championId)"
        :alt="champion.championName || getChampionName(champion.championId)"
        class="size-5 flex-none rounded object-cover"
      />
      <div class="min-w-0 flex-1">
        <div class="truncate text-muted-foreground">当前英雄</div>
        <div v-if="champion" class="truncate font-medium text-foreground tabular-nums">
          {{ champion.championName || getChampionName(champion.championId) }} ·
          <template v-if="champion.sample.games > 0">
            {{ champion.sample.games }}场 · {{ champion.sample.winRate.toFixed(0) }}%
          </template>
          <template v-else>当前排位近期未使用</template>
        </div>
        <div v-else class="truncate text-muted-foreground">近期排位无使用样本</div>
      </div>
    </div>
  </section>
</template>
