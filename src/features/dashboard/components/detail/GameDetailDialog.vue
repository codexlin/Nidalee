<template>
  <Sheet :open="!!visible" @update:open="(v) => (visible = v)">
    <SheetContent
      side="right"
      class="w-full sm:w-[min(1100px,90vw)] sm:max-w-none p-0 gap-0 flex flex-col h-full overflow-hidden"
    >
      <SheetHeader class="shrink-0 space-y-1 px-6 pt-6 pb-3 pr-12 text-left border-b border-border/60">
        <div class="flex items-center justify-between gap-3">
          <SheetTitle class="text-lg font-bold">对局详情</SheetTitle>
          <p class="text-xs text-muted-foreground shrink-0 tabular-nums">
            <kbd
              class="rounded border border-border bg-muted/60 px-1.5 py-0.5 font-sans text-[11px] text-foreground/80"
            >
              Esc
            </kbd>
            <span class="ml-1">退出</span>
          </p>
        </div>
        <SheetDescription v-if="selectedGame" class="text-sm text-muted-foreground">
          {{ resolveChampionName(selectedGame.championId, selectedGame.championName) }}
          <span class="text-border mx-1">·</span>
          {{ getQueueName(selectedGame.queueId ?? 0) }}
          <span class="text-border mx-1">·</span>
          {{ formatRelativeTime(selectedGame.gameCreation ?? 0) }}
        </SheetDescription>
      </SheetHeader>

      <ScrollArea class="flex-1 min-h-0 border-none">
        <div v-if="loading" class="flex items-center justify-center py-16 gap-3 px-6">
          <Spinner class="size-6 text-primary" />
          <span class="text-sm text-muted-foreground">正在加载对局详情…</span>
        </div>

        <div v-else-if="gameDetailData && selectedGame" class="space-y-5 px-6 py-4 pb-6">
          <MatchSummaryCard
            :selected-game="selectedGame"
            :game-detail="gameDetailData"
            :blue-won="blueWon"
            :blue-objectives="blueObjectives"
            :red-objectives="redObjectives"
            :blue-first-markers="blueFirstMarkers"
            :red-first-markers="redFirstMarkers"
          />

          <TeamBlock
            title="蓝队"
            team-id="100"
            :won="blueWon"
            :bans="getTeamBans('100')"
            :participants="getTeamParticipants('100')"
            :my-participant-id="myParticipantId"
            :game-version="gameVersion"
            @open-summoner="openFromParticipant"
            @copy-name="copyName"
          />

          <TeamBlock
            title="红队"
            team-id="200"
            :won="!blueWon"
            :bans="getTeamBans('200')"
            :participants="getTeamParticipants('200')"
            :my-participant-id="myParticipantId"
            :game-version="gameVersion"
            @open-summoner="openFromParticipant"
            @copy-name="copyName"
          />

          <MatchProcessReview
            v-if="isRankedProcessReview"
            :game-id="selectedGame.gameId ?? null"
            :puuid="processPuuid"
            :cached-evidence="cachedMatchEvidence"
            :participants="gameDetailData.participants"
            :my-participant-id="myParticipantId"
          />

          <p class="pt-1 text-right text-xs text-muted-foreground/80 tabular-nums">
            版本 {{ gameDetailData.gameVersion }}
            <span class="text-border mx-1.5">·</span>
            对局 {{ gameDetailData.gameId }}
          </p>
        </div>
      </ScrollArea>
    </SheetContent>
  </Sheet>

  <SummonerDetailSheet
    v-model:open="isSummonerSheetOpen"
    :selected-player="selectedPlayer"
    :current-result="currentResult"
    :loading="summonerLoading"
    @open-game-detail="openNestedGameDetail"
  />
</template>

<script setup lang="ts">
import { getQueueName, resolveChampionName } from '@/lib'
import { useClipboard } from '@vueuse/core'
import { toast } from 'vue-sonner'
import { useGameDetail } from '../../composables/useGameDetail'
import { useSummonerDetailSheet } from '../../composables/useSummonerDetailSheet'
import MatchProcessReview from './MatchProcessReview.vue'
import MatchSummaryCard from './MatchSummaryCard.vue'
import SummonerDetailSheet from './SummonerDetailSheet.vue'
import TeamBlock from './TeamBlock.vue'

const props = defineProps<{
  selectedGame: MatchPerformance | null
  analysisPuuid: string | null
}>()

const visible = defineModel<boolean>('visible')
const emit = defineEmits<{
  (e: 'open-game-detail', game: MatchPerformance, puuid: string): void
}>()

const { formatRelativeTime } = useFormatters()

const {
  loading,
  gameDetailData,
  gameVersion,
  processPuuid,
  isRankedProcessReview,
  cachedMatchEvidence,
  blueWon,
  blueObjectives,
  redObjectives,
  blueFirstMarkers,
  redFirstMarkers,
  myParticipantId,
  getTeamBans,
  getTeamParticipants
} = useGameDetail(
  () => props.selectedGame,
  () => props.analysisPuuid
)

const {
  isOpen: isSummonerSheetOpen,
  selectedPlayer,
  currentResult,
  loading: summonerLoading,
  openFromParticipant
} = useSummonerDetailSheet()

const clipboard = useClipboard()

function copyName(name: string) {
  clipboard.copy(name)
  toast.success('已复制召唤师名到剪贴板')
}

function openNestedGameDetail(game: MatchPerformance, puuid: string) {
  isSummonerSheetOpen.value = false
  emit('open-game-detail', game, puuid)
}

watch(
  () => [visible.value, props.analysisPuuid] as const,
  ([open, puuid], [previousOpen, previousPuuid]) => {
    if (!open || (previousOpen && puuid !== previousPuuid)) isSummonerSheetOpen.value = false
  }
)
</script>
