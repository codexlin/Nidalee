<template>
  <Sheet :open="!!visible" @update:open="(v) => (visible = v)">
    <SheetContent
      side="right"
      class="w-full sm:w-[min(1000px,92vw)] sm:max-w-none p-0 gap-0 flex flex-col h-full overflow-hidden"
    >
      <SheetHeader class="shrink-0 space-y-1 px-6 pt-6 pb-3 pr-12 text-left border-b border-border/60">
        <SheetTitle class="text-lg font-bold">对局详情</SheetTitle>
        <SheetDescription v-if="selectedGame" class="text-sm text-muted-foreground">
          {{ getChampionName(selectedGame.championId) }}
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
                  <h3 class="text-lg font-bold truncate">{{ getChampionName(selectedGame.championId) }}</h3>
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
                  <span class="ml-2 text-sm font-medium text-muted-foreground">
                    KDA {{ selectedGame.kda.toFixed(2) }}
                  </span>
                </p>
                <p class="text-sm text-muted-foreground tabular-nums flex items-center gap-2">
                  <span class="inline-flex items-center gap-1">
                    <Timer class="h-3.5 w-3.5" />
                    {{ formatDuration(gameDetailData.gameDuration || selectedGame.gameDuration || 0) }}
                  </span>
                  <span class="text-border">·</span>
                  <span :class="gradeTextClass(myGrade)" class="font-semibold">评级 {{ myGrade }}</span>
                </p>
              </div>
            </div>
          </div>

          <!-- 双方比分 + 资源（LCU teams 返回龙/塔等） -->
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
                  {{ gameDetailData.blueTeamStats?.kills || 0 }} 杀 ·
                  {{ formatNumber(gameDetailData.blueTeamStats?.goldEarned || 0) }} 金
                </span>
              </div>
              <div class="flex items-center gap-2 min-w-0 text-right">
                <span class="text-muted-foreground">
                  {{ formatNumber(gameDetailData.redTeamStats?.goldEarned || 0) }} 金 ·
                  {{ gameDetailData.redTeamStats?.kills || 0 }} 杀
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

            <!-- 首占：蓝队的靠左，红队的靠右 -->
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

          <!-- 蓝队 -->
          <TeamBlock
            title="蓝队"
            team-id="100"
            :won="blueWon"
            :bans="getTeamBans('100')"
            :participants="getTeamParticipants('100')"
            :my-participant-id="myParticipantId"
            :game-version="gameVersion"
            @open-summoner="openSummonerDetails"
            @copy-name="copyName"
          />

          <!-- 红队 -->
          <TeamBlock
            title="红队"
            team-id="200"
            :won="!blueWon"
            :bans="getTeamBans('200')"
            :participants="getTeamParticipants('200')"
            :my-participant-id="myParticipantId"
            :game-version="gameVersion"
            @open-summoner="openSummonerDetails"
            @copy-name="copyName"
          />

          <!-- 单项最佳 -->
          <div class="grid grid-cols-3 gap-3">
            <div class="surface-inset p-3 text-center space-y-1.5">
              <img
                :src="getChampionIconUrl(gameDetailData.bestPlayerChampionId as number)"
                alt=""
                class="h-9 w-9 mx-auto rounded-full border border-border"
              />
              <p class="text-lg font-semibold tabular-nums leading-none">
                {{ formatNumber(gameDetailData.maxDamage) }}
              </p>
              <p class="text-xs text-muted-foreground">最高英雄伤害</p>
            </div>
            <div class="surface-inset p-3 text-center space-y-1.5">
              <img
                :src="getChampionIconUrl(gameDetailData.maxTankChampionId as number)"
                alt=""
                class="h-9 w-9 mx-auto rounded-full border border-border"
              />
              <p class="text-lg font-semibold tabular-nums leading-none">
                {{ formatNumber(gameDetailData.maxTank) }}
              </p>
              <p class="text-xs text-muted-foreground">最高承受伤害</p>
            </div>
            <div class="surface-inset p-3 text-center space-y-1.5">
              <img
                v-if="gameDetailData.maxStreak > 0 && gameDetailData.maxStreakChampionId"
                :src="getChampionIconUrl(gameDetailData.maxStreakChampionId as number)"
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
                {{ multiKillLabel(gameDetailData.maxStreak) }}
              </p>
              <p class="text-xs text-muted-foreground">最多连杀</p>
            </div>
          </div>

          <!-- 过程复盘：仅排位（单双/灵活）；匹配与娱乐局不展示 -->
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

  <!-- 召唤师详情：与对局抽屉并列，避免嵌套层级冲突 -->
  <Sheet v-model:open="isDetailsOpen">
    <SheetContent side="right" class="w-full sm:w-[min(1000px,92vw)] sm:max-w-none overflow-y-auto p-0 gap-0">
      <div
        class="sticky top-0 z-10 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-b border-border p-6 pr-12"
      >
        <SheetHeader class="space-y-0 text-left">
          <SheetTitle class="flex items-center gap-4 text-left">
            <div v-if="currentRestult" class="flex items-center gap-4">
              <div class="w-14 h-14 rounded-full bg-muted flex items-center justify-center ring-1 ring-border">
                <span class="text-lg font-bold text-foreground">{{
                  currentRestult.displayName?.charAt(0)?.toUpperCase() || '?'
                }}</span>
              </div>
              <div>
                <h3 class="text-lg font-bold text-foreground">{{ currentRestult.displayName || '未知召唤师' }}</h3>
                <p class="text-sm text-muted-foreground">召唤师详情与战绩分析</p>
              </div>
            </div>
            <div v-else-if="selectedPlayer" class="flex items-center gap-4">
              <div class="w-14 h-14 rounded-full bg-muted flex items-center justify-center ring-1 ring-border">
                <span class="text-lg font-bold text-foreground">{{
                  selectedPlayer.displayName?.charAt(0)?.toUpperCase() || '?'
                }}</span>
              </div>
              <div>
                <h3 class="text-lg font-bold text-foreground">{{ selectedPlayer.displayName || '未知召唤师' }}</h3>
                <p class="text-sm text-muted-foreground">召唤师详情与战绩分析</p>
              </div>
            </div>
          </SheetTitle>
        </SheetHeader>
      </div>

      <div class="p-6 pt-4 space-y-6">
        <div v-if="searchLoading" class="flex items-center justify-center py-8 gap-3">
          <Spinner class="size-5 text-primary" />
          <span class="text-sm text-muted-foreground">正在查询召唤师战绩…</span>
        </div>

        <div v-else-if="currentRestult" class="space-y-6">
          <SummonerCard :summoner-info="currentRestult.summonerInfo" />
          <GameStats :is-connected="true" :match-history-loading="false" :match-statistics="currentRestult.matches" />
        </div>

        <div v-else class="flex items-center justify-center py-8">
          <div class="text-center">
            <Info class="h-10 w-10 text-muted-foreground mx-auto mb-3" />
            <h3 class="text-base font-semibold mb-1 text-foreground">暂无战绩数据</h3>
            <p class="text-sm text-muted-foreground">未能获取到该召唤师的战绩信息</p>
          </div>
        </div>
      </div>
    </SheetContent>
  </Sheet>
</template>

<script setup lang="ts">
import { getChampionIconUrl, getChampionName, getQueueName } from '@/lib'
import { invoke } from '@tauri-apps/api/core'
import { useClipboard } from '@vueuse/core'
import { toast } from 'vue-sonner'
import { Info, Timer } from 'lucide-vue-next'
import { displayGrade, gradeTextClass, gradeWatermarkClass, gradeWatermarkSizeClass } from '../../utils/matchGrade'
import TeamBlock from './TeamBlock.vue'
import MatchProcessReview from './MatchProcessReview.vue'

const props = defineProps<{
  selectedGame: MatchPerformance | null
}>()

const visible = defineModel<boolean>('visible')

const activityLogger = useActivityLogger()
const { formatRelativeTime } = useFormatters()

const loading = ref(false)
const gameDetailData = ref<GameDetail | null>(null)
const dataStore = useDataStore()
const analysisStore = usePersonalMatchAnalysisStore()
const gameVersion = computed(() => dataStore.gameVersion)

const processPuuid = computed(() => analysisStore.lastPuuid ?? dataStore.summonerInfo?.puuid ?? null)

/** 过程复盘仅单双/灵活排位 */
const isRankedProcessReview = computed(() => {
  const q = props.selectedGame?.queueId
  return q === 420 || q === 440
})

const cachedMatchEvidence = computed(() => {
  const gameId = props.selectedGame?.gameId
  if (gameId === null || gameId === undefined) return null
  return analysisStore.getMatchEvidence(gameId)
})

const isDetailsOpen = ref(false)
const selectedPlayer = ref<{ displayName: string } | null>(null)

const { fetchSummonerInfo, currentRestult, loading: searchLoading } = useSearchMatches()

const myGrade = computed(() => (props.selectedGame ? displayGrade(props.selectedGame) : 'D'))

const blueWon = computed(() => {
  const result = getTeamResult('100')
  if (result === '胜利') return true
  if (result === '失败') return false
  // 队伍胜负未知时，用本场结果兜底（仅影响条带展示）
  return !!props.selectedGame?.win
})

const teamObjectives = (teamId: number) => {
  const team = gameDetailData.value?.teams?.find((t) => t.teamId === teamId)
  return {
    dragon: team?.dragonKills ?? 0,
    baron: team?.baronKills ?? 0,
    tower: team?.towerKills ?? 0,
    inhibitor: team?.inhibitorKills ?? 0,
    herald: team?.riftHeraldKills ?? 0,
    horde: team?.hordeKills ?? 0
  }
}

const blueObjectives = computed(() => teamObjectives(100))
const redObjectives = computed(() => teamObjectives(200))

type FirstMarker = {
  key: string
  label: string
  title: string
}

const FIRST_DEFS: Array<{
  key: string
  label: string
  pick: (t: TeamInfo | undefined) => boolean | null | undefined
}> = [
  { key: 'blood', label: '一血', pick: (t) => t?.firstBlood },
  { key: 'tower', label: '一塔', pick: (t) => t?.firstTower },
  { key: 'dragon', label: '首条小龙', pick: (t) => t?.firstDragon },
  { key: 'herald', label: '首条先锋', pick: (t) => t?.firstRiftHerald },
  { key: 'baron', label: '首条大龙', pick: (t) => t?.firstBaron },
  { key: 'inhib', label: '首座水晶', pick: (t) => t?.firstInhibitor }
]

const markersForTeam = (team: TeamInfo | undefined, side: '蓝队' | '红队'): FirstMarker[] => {
  if (!team) return []
  return FIRST_DEFS.filter((def) => !!def.pick(team)).map((def) => ({
    key: def.key,
    label: def.label,
    title: `${side}拿下${def.label}`
  }))
}

const blueFirstMarkers = computed(() => {
  const blue = gameDetailData.value?.teams?.find((t) => t.teamId === 100)
  return markersForTeam(blue, '蓝队')
})

const redFirstMarkers = computed(() => {
  const red = gameDetailData.value?.teams?.find((t) => t.teamId === 200)
  return markersForTeam(red, '红队')
})

const multiKillLabel = (n: number) => {
  if (!n || n <= 1) return '无'
  if (n === 2) return '双杀'
  if (n === 3) return '三杀'
  if (n === 4) return '四杀'
  if (n >= 5) return '五杀'
  return String(n)
}

const myParticipantId = computed(() => {
  const game = props.selectedGame
  const detail = gameDetailData.value
  if (!game || !detail?.participants?.length) return null

  const exact = detail.participants.find(
    (p) =>
      p.championId === game.championId &&
      p.stats.kills === game.kills &&
      p.stats.deaths === game.deaths &&
      p.stats.assists === game.assists
  )
  if (exact) return exact.participantId

  const byChamp = detail.participants.find((p) => p.championId === game.championId)
  return byChamp?.participantId ?? null
})

watch(
  () => props.selectedGame,
  async (newGame) => {
    if (newGame?.gameId === null || newGame?.gameId === undefined) {
      gameDetailData.value = null
      return
    }

    loading.value = true
    try {
      const result = await invoke<GameDetail>('get_game_detail', {
        gameId: newGame.gameId
      })
      gameDetailData.value = result
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err)
      console.error('获取游戏详细信息失败:', message)
      activityLogger.logError.apiError(`获取游戏详细信息失败: ${message}`)
      gameDetailData.value = null
    } finally {
      loading.value = false
    }
  }
)

const openSummonerDetails = async (participant: ParticipantInfo) => {
  selectedPlayer.value = {
    displayName: participant.summonerName
  }
  isDetailsOpen.value = true
  if (
    participant.summonerName &&
    participant.summonerName !== '未知玩家' &&
    participant.summonerName !== '未知召唤师'
  ) {
    await fetchSummonerInfo([participant.summonerName])
  }
}

const formatDuration = (seconds: number) => {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}分 ${s}秒`
}
const formatNumber = (num: number) => num.toLocaleString()

const getTeamResult = (teamId: string) => {
  if (!gameDetailData.value) return '未知'
  const team = gameDetailData.value.teams.find((t) => t.teamId && t.teamId.toString() === teamId)
  if (!team || !team.win) return '未知'
  return team.win === 'Win' ? '胜利' : '失败'
}

const getTeamBans = (teamId: string) => {
  const teams = gameDetailData.value?.teams
  if (!teams) return []
  const team = teams.find((t) => t.teamId && t.teamId.toString() === teamId)
  return team?.bans || []
}

const getTeamParticipants = (teamId: string) => {
  if (!gameDetailData.value?.participants) return []
  return gameDetailData.value.participants.filter((p) => p.teamId.toString() === teamId)
}

const clipboard = useClipboard()

function copyName(name: string) {
  clipboard.copy(name)
  toast.success('已复制召唤师名到剪贴板')
}
</script>
