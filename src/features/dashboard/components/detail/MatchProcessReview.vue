<template>
  <section class="surface-raised p-4 flex flex-col gap-0">
    <div class="flex items-baseline justify-between gap-2 pb-3">
      <h3 class="text-base font-medium">过程复盘</h3>
      <span v-if="review?.fromCache" class="text-xs text-muted-foreground">缓存</span>
    </div>

    <div v-if="loading" class="flex items-center gap-2 border-t border-border/50 pt-3 text-sm text-muted-foreground">
      <Spinner class="size-4 text-primary" />
      正在整理本局过程…
    </div>

    <p v-else-if="error" class="border-t border-border/50 pt-3 text-sm text-muted-foreground">
      {{ error }}
    </p>

    <template v-else-if="insight">
      <p v-if="insight.degradationMessage" class="border-t border-border/50 pt-3 text-sm text-muted-foreground">
        {{ insight.degradationMessage }}
      </p>

      <template v-if="insight.hasTimeline">
        <!-- 1. 对位：左右对称 -->
        <div class="border-t border-border/50 pt-3 flex flex-col gap-3">
          <p class="text-sm font-medium">对位</p>

          <template v-if="opponentCompare">
            <div class="grid grid-cols-[1fr_auto_1fr] items-center gap-3">
              <div class="flex flex-col items-end gap-1.5 min-w-0">
                <img
                  :src="getChampionIconUrl(opponentCompare.myChampionId)"
                  alt=""
                  class="size-11 rounded-full border border-border"
                />
                <p class="text-sm font-medium truncate max-w-full">{{ myName }}</p>
                <p class="text-xs text-muted-foreground">我</p>
              </div>

              <div class="flex flex-col items-center gap-1 px-1">
                <span class="text-xs font-medium text-muted-foreground">VS</span>
                <p
                  v-if="primaryPhase"
                  class="text-xs tabular-nums font-medium text-center leading-snug"
                  :class="diffTone(primaryPhase.overallAdvantagePct)"
                >
                  {{ primaryPhase.label }}
                  <br />
                  {{ formatSigned(primaryPhase.overallAdvantagePct) }}%
                </p>
              </div>

              <div class="flex flex-col items-start gap-1.5 min-w-0">
                <img
                  v-if="opponentCompare.opponentChampionId"
                  :src="getChampionIconUrl(opponentCompare.opponentChampionId)"
                  alt=""
                  class="size-11 rounded-full border border-border"
                />
                <div
                  v-else
                  class="size-11 rounded-full border border-border bg-muted/40 flex items-center justify-center text-xs text-muted-foreground"
                >
                  ?
                </div>
                <p class="text-sm font-medium truncate max-w-full">{{ opponentName }}</p>
                <p class="text-xs text-muted-foreground">对位</p>
              </div>
            </div>

            <div v-if="endCompare" class="surface-inset rounded-xl px-3 py-2.5 flex flex-col gap-2">
              <div
                v-for="kpi in endKpis"
                :key="kpi.label"
                class="grid grid-cols-[1fr_auto_1fr] items-baseline gap-2 tabular-nums"
              >
                <p class="text-sm font-medium text-right" :class="diffTone(kpi.diff)">
                  {{ kpi.mine }}
                </p>
                <p class="text-xs text-muted-foreground w-12 text-center">{{ kpi.label }}</p>
                <p class="text-sm font-medium text-left" :class="diffTone(-kpi.diff)">
                  {{ kpi.theirs }}
                </p>
              </div>
            </div>

            <p
              v-if="secondaryPhases.length"
              class="text-sm text-muted-foreground tabular-nums text-center leading-relaxed"
            >
              <span v-for="(phase, i) in secondaryPhases" :key="phase.phase">
                <template v-if="i > 0">
                  <span class="text-border mx-1.5">·</span>
                </template>
                {{ phase.label }}
                <span :class="diffTone(phase.overallAdvantagePct)" class="font-medium">
                  {{ formatSigned(phase.overallAdvantagePct) }}%
                </span>
              </span>
            </p>
          </template>

          <p v-else class="text-sm text-muted-foreground">未能识别同位置对位。若刚更新过后端，请重启应用后再打开。</p>
        </div>

        <!-- 2. 时间轴：与详情页同一套 ScrollArea -->
        <div v-if="keyMoments.length" class="border-t border-border/50 pt-3 flex flex-col gap-2">
          <div class="flex items-baseline justify-between gap-2">
            <p class="text-sm font-medium">
              时间轴
              <span class="text-muted-foreground font-normal tabular-nums">
                · {{ visibleMoments.length }}/{{ keyMoments.length }}
              </span>
            </p>
            <button
              v-if="keyMoments.length > highlightMoments.length"
              type="button"
              class="text-xs text-muted-foreground hover:text-foreground transition-colors focus-visible:outline-none focus-visible:ring-ring/50 focus-visible:ring-[3px] rounded-md"
              @click="showAllMoments = !showAllMoments"
            >
              {{ showAllMoments ? '只看关键' : '查看全部' }}
            </button>
          </div>

          <ScrollArea class="h-48 w-full">
            <div class="pr-3">
              <div class="grid grid-cols-[1fr_auto_1fr] gap-x-2 px-0.5 pb-1.5 text-xs text-muted-foreground">
                <span class="text-right">我</span>
                <span class="min-w-28 text-center">时间 · 事件</span>
                <span class="text-left">对位</span>
              </div>

              <ol>
                <li
                  v-for="(moment, index) in visibleMoments"
                  :key="`${moment.timestampMs}-${index}`"
                  class="grid grid-cols-[1fr_auto_1fr] gap-x-2 pb-2.5 last:pb-1"
                >
                  <p
                    class="min-w-0 self-center text-right text-sm leading-snug text-muted-foreground"
                    :class="moment.detail ? '' : 'text-muted-foreground/35'"
                  >
                    {{ moment.detail || '—' }}
                  </p>

                  <div class="relative flex min-w-28 flex-col items-center self-stretch px-1">
                    <div class="flex items-baseline justify-center gap-1.5">
                      <time class="text-sm tabular-nums text-muted-foreground shrink-0">
                        {{ formatClock(moment.timestampMs) }}
                      </time>
                      <p class="text-sm font-medium leading-snug text-center">
                        {{ moment.label }}
                      </p>
                    </div>
                    <span
                      class="z-10 mt-1.5 size-1.5 shrink-0 rounded-full ring-2 ring-background"
                      :class="momentDotClass(moment.label)"
                    />
                    <span
                      v-if="index < visibleMoments.length - 1"
                      class="absolute top-8 bottom-0 w-px bg-border/60"
                      aria-hidden="true"
                    />
                  </div>

                  <p
                    class="min-w-0 self-center text-left text-sm leading-snug text-muted-foreground"
                    :class="moment.opponentDetail ? '' : 'text-muted-foreground/35'"
                  >
                    {{ moment.opponentDetail || '—' }}
                  </p>
                </li>
              </ol>
            </div>
          </ScrollArea>
        </div>

        <!-- 3. 要点 + 建议合并 -->
        <div
          v-if="highlightChips.length || topActions.length"
          class="border-t border-border/50 pt-3 flex flex-col gap-2.5"
        >
          <p class="text-sm font-medium">本局要点与建议</p>

          <div v-if="highlightChips.length" class="flex flex-wrap gap-1.5">
            <Badge
              v-for="chip in highlightChips"
              :key="chip.key"
              variant="outline"
              class="h-7 rounded-xl px-2.5 text-xs font-medium tabular-nums border-border/70"
              :class="chip.tone"
            >
              {{ chip.label }}
            </Badge>
          </div>

          <ul v-if="topActions.length" class="flex flex-col gap-1.5">
            <li v-for="action in topActions" :key="action.key" class="text-sm leading-snug">
              <span class="font-medium">{{ action.title }}</span>
              <span class="text-muted-foreground"> — {{ action.detail }}</span>
            </li>
          </ul>
        </div>
      </template>
    </template>
  </section>
</template>

<script setup lang="ts">
import { getChampionIconUrl, getChampionName } from '@/lib'
import { invoke } from '@tauri-apps/api/core'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'

const props = defineProps<{
  gameId: number | null
  puuid: string | null
  cachedEvidence: MatchEvidence | null
  participants?: ParticipantInfo[] | null
  myParticipantId?: number | null
}>()

const loading = ref(false)
const error = ref<string | null>(null)
const review = ref<GameProcessReview | null>(null)
const showAllMoments = ref(false)
let loadSeq = 0

const insight = computed(() => review.value?.insight ?? null)
const keyMoments = computed(() => review.value?.keyMoments ?? [])
const opponentCompare = computed(() => review.value?.opponentCompare ?? null)

const primaryPhase = computed(
  () => opponentCompare.value?.phases.find((p) => p.phase === 'early') ?? opponentCompare.value?.phases[0] ?? null
)

const secondaryPhases = computed(
  () => opponentCompare.value?.phases.filter((p) => p.phase !== primaryPhase.value?.phase) ?? []
)

const opponentParticipant = computed(() => {
  const id = opponentCompare.value?.opponentParticipantId
  if (id === null || id === undefined || !props.participants?.length) return null
  return props.participants.find((p) => p.participantId === id) ?? null
})

const myParticipant = computed(() => {
  const id = props.myParticipantId
  if (id === null || id === undefined || !props.participants?.length) return null
  return props.participants.find((p) => p.participantId === id) ?? null
})

const myName = computed(() => {
  const me = myParticipant.value
  if (me?.summonerName && me.summonerName !== '未知玩家') return me.summonerName
  const champId = opponentCompare.value?.myChampionId
  if (champId) return getChampionName(champId)
  return '我'
})

const opponentName = computed(() => {
  const opp = opponentParticipant.value
  if (opp?.summonerName && opp.summonerName !== '未知玩家') return opp.summonerName
  const champId = opponentCompare.value?.opponentChampionId
  if (champId) return getChampionName(champId)
  return '对位'
})

const endCompare = computed(() => {
  const me = myParticipant.value
  const opp = opponentParticipant.value
  if (!me || !opp) return null
  const myKda =
    me.stats.deaths > 0 ? (me.stats.kills + me.stats.assists) / me.stats.deaths : me.stats.kills + me.stats.assists
  const oppKda =
    opp.stats.deaths > 0
      ? (opp.stats.kills + opp.stats.assists) / opp.stats.deaths
      : opp.stats.kills + opp.stats.assists
  const myVision = me.stats.visionScore ?? 0
  const oppVision = opp.stats.visionScore ?? 0
  return {
    myKda: myKda.toFixed(1),
    oppKda: oppKda.toFixed(1),
    kdaDiff: myKda - oppKda,
    myDamage: me.stats.totalDamageDealtToChampions,
    oppDamage: opp.stats.totalDamageDealtToChampions,
    damageDiff: me.stats.totalDamageDealtToChampions - opp.stats.totalDamageDealtToChampions,
    myGold: me.stats.goldEarned,
    oppGold: opp.stats.goldEarned,
    goldDiff: me.stats.goldEarned - opp.stats.goldEarned,
    myVision,
    oppVision,
    visionDiff: myVision - oppVision
  }
})

const endKpis = computed(() => {
  const e = endCompare.value
  if (!e) return []
  return [
    { label: 'KDA', mine: e.myKda, theirs: e.oppKda, diff: e.kdaDiff },
    {
      label: '伤害',
      mine: formatCompact(e.myDamage),
      theirs: formatCompact(e.oppDamage),
      diff: e.damageDiff
    },
    {
      label: '经济',
      mine: formatCompact(e.myGold),
      theirs: formatCompact(e.oppGold),
      diff: e.goldDiff
    },
    {
      label: '视野',
      mine: String(e.myVision),
      theirs: String(e.oppVision),
      diff: e.visionDiff
    }
  ]
})

/** 阵亡主导项 + 资源错过 → 少量芯片 */
const highlightChips = computed(() => {
  const chips: Array<{ key: string; label: string; tone: string }> = []
  const death = insight.value?.deathBreakdown
  if (death && death.totalDeaths > 0) {
    const parts = [
      { n: death.gankOrMulti, label: '多人集火' },
      { n: death.solo, label: '被单杀' },
      { n: death.towerOrMinion, label: '塔刀' }
    ].filter((p) => p.n > 0)
    parts.sort((a, b) => b.n - a.n)
    const top = parts[0]
    if (top) {
      chips.push({
        key: 'death',
        label: `${top.label} ${top.n}/${death.totalDeaths}`,
        tone:
          top.n / death.totalDeaths >= 0.55
            ? 'bg-amber-500/10 text-amber-800 dark:text-amber-300'
            : 'bg-muted/40 text-muted-foreground'
      })
    }
  }

  const obj = insight.value?.objectiveProcess
  if (obj) {
    const missed = obj.dragonsMissed + obj.heraldsMissed + obj.baronsMissed
    if (missed > 0) {
      chips.push({
        key: 'obj-miss',
        label: `错过资源 ${missed}`,
        tone: 'bg-rose-500/10 text-rose-700 dark:text-rose-300'
      })
    }
    const taken = obj.dragonsTaken + obj.heraldsTaken + obj.baronsTaken
    const seen = obj.dragonsSeen + obj.heraldsSeen + obj.baronsSeen
    if (taken > 0 && seen > 0) {
      chips.push({
        key: 'obj-take',
        label: `资源到场 ${taken}/${seen}`,
        tone: 'bg-muted/40 text-muted-foreground'
      })
    }
  }

  return chips
})

/** 关键时刻：资源事件优先，阵亡最多保留 3 条 */
const highlightMoments = computed(() => {
  const moments = keyMoments.value
  const objectives = moments.filter(
    (m) => m.label.startsWith('错过') || m.label.startsWith('参与') || m.label.startsWith('己方')
  )
  const deaths = moments
    .filter(
      (m) => m.label.includes('集火') || m.label.includes('单杀') || m.label.includes('塔刀') || m.label === '阵亡'
    )
    .slice(0, 3)

  const merged = [...objectives, ...deaths].sort((a, b) => a.timestampMs - b.timestampMs)
  const seen = new Set<string>()
  return merged.filter((m) => {
    const key = `${m.timestampMs}-${m.label}`
    if (seen.has(key)) return false
    seen.add(key)
    return true
  })
})

const visibleMoments = computed(() => (showAllMoments.value ? keyMoments.value : highlightMoments.value))

const topActions = computed(() => (insight.value?.actions ?? []).slice(0, 2))

const formatSigned = (n: number) => (n > 0 ? `+${n.toFixed(0)}` : n.toFixed(0))

const formatCompact = (n: number) => {
  if (Math.abs(n) >= 1000) return `${(n / 1000).toFixed(1)}k`
  return String(Math.round(n))
}

const formatClock = (ms: number) => {
  const total = Math.max(0, Math.floor(ms / 1000))
  const m = Math.floor(total / 60)
  const s = total % 60
  return `${m}:${String(s).padStart(2, '0')}`
}

const diffTone = (n: number) => {
  if (n > 0.5) return 'text-emerald-600 dark:text-emerald-400'
  if (n < -0.5) return 'text-rose-600 dark:text-rose-400'
  return 'text-foreground'
}

const momentDotClass = (label: string) => {
  if (label.startsWith('参与')) return 'bg-emerald-500'
  if (label.startsWith('错过')) return 'bg-rose-500'
  if (label.includes('单杀') || label.includes('集火') || label.includes('塔刀') || label === '阵亡') {
    return 'bg-amber-500'
  }
  return 'bg-muted-foreground/50'
}

watch(
  () => [props.gameId, props.puuid, props.cachedEvidence?.gameId] as const,
  async ([gameId, puuid]) => {
    const seq = ++loadSeq
    review.value = null
    error.value = null
    showAllMoments.value = false

    if (gameId === null || gameId === undefined) return
    if (!puuid) {
      error.value = '未找到当前召唤师，无法做过程复盘。'
      return
    }

    loading.value = true
    try {
      const next = await invoke<GameProcessReview>('get_game_process_review', {
        puuid,
        gameId,
        cachedEvidence: props.cachedEvidence ?? null
      })
      if (seq !== loadSeq) return
      review.value = next
    } catch (err: unknown) {
      if (seq !== loadSeq) return
      const message = err instanceof Error ? err.message : String(err)
      error.value = message || '过程复盘加载失败'
      review.value = null
    } finally {
      if (seq === loadSeq) loading.value = false
    }
  },
  { immediate: true }
)
</script>
