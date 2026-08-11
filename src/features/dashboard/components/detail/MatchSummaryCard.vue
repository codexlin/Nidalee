<template>
  <div class="space-y-5">
    <!-- 本局摘要 -->
    <div class="surface-raised relative overflow-hidden p-4">
      <span
        class="pointer-events-none absolute -right-1 -bottom-3 z-0 select-none font-black leading-none tabular-nums -rotate-12 origin-bottom-right"
        :class="[gradeWatermarkSizeClass(myGrade), gradeWatermarkClass(myGrade)]"
        aria-hidden="true"
      >
        {{ myGrade }}
      </span>
      <div class="relative z-10 flex flex-wrap items-center gap-4">
        <div class="relative shrink-0">
          <img
            :src="getChampionIconUrl(selectedGame.championId)"
            alt=""
            class="h-14 w-14 rounded-full border border-border"
          />
          <span
            class="absolute -bottom-1 -right-1 h-5 min-w-5 px-1 rounded-md text-xs font-medium text-white inline-flex items-center justify-center"
            :class="selectedGame.win ? 'bg-emerald-600' : 'bg-rose-600'"
          >
            {{ selectedGame.win ? '胜' : '负' }}
          </span>
        </div>
        <div class="min-w-0 flex-1 space-y-1">
          <div class="flex flex-wrap items-baseline gap-2">
            <h3 class="text-lg font-bold truncate">
              {{ resolveChampionName(selectedGame.championId, selectedGame.championName) }}
            </h3>
            <span class="text-sm text-muted-foreground tabular-nums">
              {{ getQueueName(selectedGame.queueId ?? 0) }}
            </span>
          </div>
          <p class="font-mono text-lg font-bold tabular-nums leading-none">
            <span class="text-red-500">{{ selectedGame.kills }}</span>
            <span class="text-muted-foreground/50">/</span>
            <span class="text-muted-foreground">{{ selectedGame.deaths }}</span>
            <span class="text-muted-foreground/50">/</span>
            <span class="text-blue-500">{{ selectedGame.assists }}</span>
            <span class="ml-2 text-sm font-medium text-muted-foreground"> KDA {{ selectedGame.kda.toFixed(2) }} </span>
          </p>
          <p class="text-sm text-muted-foreground tabular-nums flex items-center gap-2">
            <span class="inline-flex items-center gap-1">
              <Timer class="h-3.5 w-3.5" />
              {{ formatMatchDuration(gameDetail.gameDuration || selectedGame.gameDuration || 0) }}
            </span>
            <span class="text-border">·</span>
            <span :class="gradeTextClass(myGrade)" class="font-semibold">评级 {{ myGrade }}</span>
          </p>
        </div>
      </div>
    </div>

    <!-- 双方比分 + 资源 -->
    <div class="surface-inset px-4 py-3 space-y-2.5 text-sm tabular-nums">
      <div class="flex items-center justify-between gap-3">
        <div class="flex items-center gap-2 min-w-0">
          <span
            class="h-6 px-2 inline-flex items-center rounded-md text-sm font-medium text-white"
            :class="blueWon ? 'bg-emerald-600' : 'bg-rose-600'"
          >
            {{ blueWon ? '胜' : '负' }}
          </span>
          <span class="text-base font-medium">蓝队</span>
          <span class="text-muted-foreground">
            {{ gameDetail.blueTeamStats?.kills || 0 }} 杀 ·
            {{ formatMatchNumber(gameDetail.blueTeamStats?.goldEarned || 0) }} 金
          </span>
        </div>
        <div class="flex items-center gap-2 min-w-0 text-right">
          <span class="text-muted-foreground">
            {{ formatMatchNumber(gameDetail.redTeamStats?.goldEarned || 0) }} 金 ·
            {{ gameDetail.redTeamStats?.kills || 0 }} 杀
          </span>
          <span class="text-base font-medium">红队</span>
          <span
            class="h-6 px-2 inline-flex items-center rounded-md text-sm font-medium text-white"
            :class="!blueWon ? 'bg-emerald-600' : 'bg-rose-600'"
          >
            {{ !blueWon ? '胜' : '负' }}
          </span>
        </div>
      </div>
      <div
        class="flex items-center justify-between gap-3 text-sm text-muted-foreground border-t border-border/50 pt-2.5"
      >
        <span>
          小龙 {{ blueObjectives.dragon }} · 大龙 {{ blueObjectives.baron }} · 塔
          {{ blueObjectives.tower }}
          <template v-if="blueObjectives.herald > 0"> · 先锋 {{ blueObjectives.herald }}</template>
          <template v-if="blueObjectives.horde > 0"> · 幼体 {{ blueObjectives.horde }}</template>
          <template v-if="blueObjectives.inhibitor > 0"> · 水晶 {{ blueObjectives.inhibitor }}</template>
        </span>
        <span class="text-border shrink-0 text-xs">资源</span>
        <span class="text-right">
          小龙 {{ redObjectives.dragon }} · 大龙 {{ redObjectives.baron }} · 塔
          {{ redObjectives.tower }}
          <template v-if="redObjectives.herald > 0"> · 先锋 {{ redObjectives.herald }}</template>
          <template v-if="redObjectives.horde > 0"> · 幼体 {{ redObjectives.horde }}</template>
          <template v-if="redObjectives.inhibitor > 0"> · 水晶 {{ redObjectives.inhibitor }}</template>
        </span>
      </div>

      <div
        v-if="blueFirstMarkers.length || redFirstMarkers.length"
        class="flex items-start justify-between gap-3 border-t border-border/50 pt-2.5"
      >
        <div class="flex flex-wrap gap-1.5 min-w-0 flex-1 justify-start">
          <span
            v-for="marker in blueFirstMarkers"
            :key="marker.key"
            class="inline-flex h-6 items-center rounded-md px-2 text-sm font-medium bg-sky-500/12 text-sky-700 dark:text-sky-300"
            :title="marker.title"
          >
            {{ marker.label }}
          </span>
        </div>
        <div class="flex flex-wrap gap-1.5 min-w-0 flex-1 justify-end">
          <span
            v-for="marker in redFirstMarkers"
            :key="marker.key"
            class="inline-flex h-6 items-center rounded-md px-2 text-sm font-medium bg-rose-500/12 text-rose-700 dark:text-rose-300"
            :title="marker.title"
          >
            {{ marker.label }}
          </span>
        </div>
      </div>
    </div>

    <!-- 单项最佳 -->
    <div class="grid grid-cols-3 gap-3">
      <div class="surface-inset p-3 text-center space-y-1.5">
        <img
          :src="getChampionIconUrl(gameDetail.bestPlayerChampionId as number)"
          alt=""
          class="h-9 w-9 mx-auto rounded-full border border-border"
        />
        <p class="text-lg font-semibold tabular-nums leading-none">
          {{ formatMatchNumber(gameDetail.maxDamage) }}
        </p>
        <p class="text-xs text-muted-foreground">最高英雄伤害</p>
      </div>
      <div class="surface-inset p-3 text-center space-y-1.5">
        <img
          :src="getChampionIconUrl(gameDetail.maxTankChampionId as number)"
          alt=""
          class="h-9 w-9 mx-auto rounded-full border border-border"
        />
        <p class="text-lg font-semibold tabular-nums leading-none">
          {{ formatMatchNumber(gameDetail.maxTank) }}
        </p>
        <p class="text-xs text-muted-foreground">最高承受伤害</p>
      </div>
      <div class="surface-inset p-3 text-center space-y-1.5">
        <img
          v-if="gameDetail.maxStreak > 0 && gameDetail.maxStreakChampionId"
          :src="getChampionIconUrl(gameDetail.maxStreakChampionId as number)"
          alt=""
          class="h-9 w-9 mx-auto rounded-full border border-border"
        />
        <div
          v-else
          class="h-9 w-9 mx-auto rounded-full border border-border bg-muted/40 flex items-center justify-center text-xs text-muted-foreground"
        >
          —
        </div>
        <p class="text-lg font-semibold tabular-nums leading-none">
          {{ multiKillLabel(gameDetail.maxStreak) }}
        </p>
        <p class="text-xs text-muted-foreground">最多连杀</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getChampionIconUrl, getQueueName, resolveChampionName } from '@/lib'
import { Timer } from 'lucide-vue-next'
import { computed } from 'vue'
import { formatMatchDuration, formatMatchNumber, multiKillLabel } from '../../utils/matchDetailFormatters'
import { displayGrade, gradeTextClass, gradeWatermarkClass, gradeWatermarkSizeClass } from '../../utils/matchGrade'

type TeamObjectives = {
  dragon: number
  baron: number
  tower: number
  inhibitor: number
  herald: number
  horde: number
}

type FirstMarker = {
  key: string
  label: string
  title: string
}

const props = defineProps<{
  selectedGame: MatchPerformance
  gameDetail: GameDetail
  blueWon: boolean
  blueObjectives: TeamObjectives
  redObjectives: TeamObjectives
  blueFirstMarkers: FirstMarker[]
  redFirstMarkers: FirstMarker[]
}>()

const myGrade = computed(() => displayGrade(props.selectedGame))
</script>
