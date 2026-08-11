<template>
  <HoverCard :open-delay="260" :close-delay="100">
    <HoverCardTrigger as-child>
      <slot />
    </HoverCardTrigger>
    <HoverCardContent
      side="top"
      :side-offset="10"
      class="w-[300px] overflow-hidden rounded-xl border border-border bg-popover p-0 shadow-md"
    >
      <!-- 顶栏 -->
      <div class="relative border-b border-border/60 px-2.5 py-2">
        <div
          class="pointer-events-none absolute inset-0 opacity-40"
          :class="
            grade === 'S+' || grade === 'S'
              ? 'bg-gradient-to-br from-amber-500/15 to-transparent'
              : 'bg-gradient-to-br from-primary/10 to-transparent'
          "
        />
        <div class="relative z-10 flex items-center gap-2 min-w-0">
          <div class="relative shrink-0">
            <img
              :src="getChampionIconUrl(participant.championId)"
              alt=""
              class="h-9 w-9 rounded-full border border-border shadow-sm"
            />
            <span
              class="absolute -bottom-0.5 -right-0.5 bg-background text-foreground text-[9px] min-w-[16px] h-3.5 px-0.5 flex items-center justify-center rounded ring-1 ring-border tabular-nums leading-none"
            >
              {{ stats.champLevel || '?' }}
            </span>
          </div>
          <div class="min-w-0 flex-1">
            <p class="truncate text-sm font-semibold tracking-tight leading-tight">
              {{ participant.summonerName }}
            </p>
            <p class="mt-0.5 truncate text-[11px] text-muted-foreground leading-tight">
              {{ getChampionName(participant.championId) }}
              <span class="text-border mx-1">·</span>
              <span :class="gradeTextClass(grade)" class="font-semibold">评级 {{ grade }}</span>
            </p>
          </div>
          <div class="flex flex-col gap-0.5 shrink-0">
            <template v-for="(spellId, idx) in spells" :key="idx">
              <img
                v-if="spellId && getSpellMeta(spellId).icon"
                :src="getSpellMeta(spellId).icon"
                :alt="getSpellMeta(spellId).label"
                :title="getSpellMeta(spellId).label"
                class="size-[18px] rounded bg-muted/50 ring-1 ring-border/50"
              />
            </template>
          </div>
        </div>
      </div>

      <!-- 雷达占主视觉；竖线分割右侧明细 -->
      <div class="flex items-stretch gap-0 px-2 pb-2 pt-1.5">
        <div class="min-w-0 flex-1 pr-2">
          <ThemedChart type="radar" :data="radarChartData" :options="radarChartOptions" height="176px" />
        </div>

        <div class="w-px shrink-0 self-stretch bg-border/70 my-1" aria-hidden="true" />

        <div class="w-[86px] shrink-0 py-1 pl-2">
          <div class="flex h-full flex-col justify-center gap-[5px] text-[11px] tabular-nums">
            <div
              v-for="row in detailRows"
              :key="row.label"
              class="flex items-baseline gap-1.5 leading-none"
            >
              <span class="shrink-0 text-muted-foreground">{{ row.label }}</span>
              <span class="min-w-0 font-medium text-foreground truncate" :class="row.toneClass">{{
                row.value
              }}</span>
            </div>
          </div>
        </div>
      </div>
    </HoverCardContent>
  </HoverCard>
</template>

<script setup lang="ts">
import { getChampionIconUrl, getChampionName, getSpellMeta } from '@/lib'
import { gradeFromStats, gradeTextClass } from '../../utils/matchGrade'
import { HoverCard, HoverCardContent, HoverCardTrigger } from '@/components/ui/hover-card'
import ThemedChart from '@/shared/components/charts/ThemedChart.vue'
import { themeColor, themeColors } from '@/lib/themeColor'
import type { ChartData, ChartOptions, ScriptableScaleContext, TooltipItem } from 'chart.js'

const props = defineProps<{
  participant: ParticipantInfo
  teamKills: number
  teamMaxGold?: number
  teamMaxDamage?: number
  teamMaxVision?: number
  isMaxGold?: boolean
  isMaxDamage?: boolean
}>()

useSummonerSpells()

const stats = computed(() => props.participant.stats)
const grade = computed(() =>
  gradeFromStats(stats.value?.kills ?? 0, stats.value?.deaths ?? 0, stats.value?.assists ?? 0)
)
const spells = computed(() => [props.participant.spell1Id ?? 0, props.participant.spell2Id ?? 0] as const)

const killParticipation = computed(() => {
  const team = props.teamKills
  if (team <= 0) return null
  const involvements = (stats.value?.kills ?? 0) + (stats.value?.assists ?? 0)
  return Math.min(100, (involvements / team) * 100)
})

const totalCs = computed(
  () => (stats.value?.totalMinionsKilled ?? 0) + (stats.value?.neutralMinionsKilled ?? 0)
)

const formatCompact = (n: number) => {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k`
  return n.toLocaleString()
}

const multiKillLabel = (n: number) => {
  if (n >= 5) return '五杀'
  if (n === 4) return '四杀'
  if (n === 3) return '三杀'
  if (n === 2) return '双杀'
  return '—'
}

const clamp01 = (n: number) => Math.max(0, Math.min(1, n))

/**
 * 队内最高向上取整到「好看」的刻度，避免第一名永远贴雷达外圈。
 * 大数（伤害/经济）整千；中等（视野等）整五；小数向上取整。
 */
const ceilScale = (n: number) => {
  const v = Math.max(0, n)
  if (v <= 0) return 1
  if (v >= 1000) return Math.ceil(v / 1000) * 1000
  if (v >= 10) return Math.ceil(v / 5) * 5
  return Math.ceil(v)
}

type RadarAxis = {
  key: string
  display: string
  name: string
  value: number
  emphasize?: boolean
}

const radarAxes = computed((): RadarAxis[] => {
  const dmg = stats.value?.totalDamageDealtToChampions || 0
  const gold = stats.value?.goldEarned || 0
  const vision = stats.value?.visionScore || 0
  const deaths = stats.value?.deaths || 0
  const maxDmg = ceilScale(Math.max(props.teamMaxDamage || 0, dmg))
  const maxGold = ceilScale(Math.max(props.teamMaxGold || 0, gold))
  const maxVision = ceilScale(Math.max(props.teamMaxVision || 0, vision))
  const kp = killParticipation.value
  const survive = deaths <= 0 ? 1 : clamp01(1 - deaths / 12)

  return [
    {
      key: 'dmg',
      name: '伤害',
      display: formatCompact(dmg),
      value: clamp01(dmg / maxDmg),
      emphasize: props.isMaxDamage
    },
    {
      key: 'gold',
      name: '经济',
      display: formatCompact(gold),
      value: clamp01(gold / maxGold),
      emphasize: props.isMaxGold
    },
    {
      key: 'kp',
      name: '参团',
      display: kp === null ? '—' : `${kp.toFixed(0)}%`,
      value: clamp01((kp ?? 0) / 100)
    },
    {
      key: 'vision',
      name: '视野',
      display: String(vision),
      value: clamp01(vision / maxVision)
    },
    {
      key: 'survive',
      name: '生存',
      display: `${deaths}死`,
      value: survive
    }
  ]
})

const radarChartData = computed((): ChartData<'radar'> => {
  const c = themeColors()
  const axes = radarAxes.value
  return {
    // 仅维度名：具体数值在右侧明细，避免轴标拥挤重复
    labels: axes.map((a) => a.name),
    datasets: [
      {
        label: '本局',
        data: axes.map((a) => Math.round(a.value * 100)),
        backgroundColor: themeColor('--primary', 0.22),
        borderColor: c.primary,
        borderWidth: 2,
        pointBackgroundColor: c.primary,
        pointBorderColor: c.background,
        pointBorderWidth: 1,
        pointRadius: 3,
        pointHoverRadius: 5
      }
    ]
  }
})

const radarChartOptions = computed((): ChartOptions<'radar'> => {
  const c = themeColors()
  const axes = radarAxes.value
  return {
    responsive: true,
    maintainAspectRatio: false,
    layout: { padding: 4 },
    plugins: {
      legend: { display: false },
      tooltip: {
        callbacks: {
          title: (items) => {
            const idx = items[0]?.dataIndex ?? 0
            return axes[idx]?.name ?? ''
          },
          label: (context: TooltipItem<'radar'>) => {
            const axis = axes[context.dataIndex]
            return axis ? axis.display : ''
          }
        }
      }
    },
    scales: {
      r: {
        beginAtZero: true,
        min: 0,
        max: 100,
        ticks: {
          display: false,
          maxTicksLimit: 3,
          backdropColor: 'transparent'
        },
        grid: { color: c.border },
        angleLines: { color: c.border },
        pointLabels: {
          color: (ctx: ScriptableScaleContext) => {
            const axis = axes[ctx.index]
            if (axis?.emphasize) return 'oklch(0.63 0.2 25)'
            return c.foreground
          },
          font: {
            size: 12,
            weight: 700,
            family: '"HarmonyOS Sans SC", "Microsoft YaHei", sans-serif'
          },
          padding: 5
        }
      }
    }
  }
})

const detailRows = computed(() => [
  { label: '击杀', value: String(stats.value?.kills ?? 0) },
  { label: '死亡', value: String(stats.value?.deaths ?? 0) },
  { label: '助攻', value: String(stats.value?.assists ?? 0) },
  { label: '补刀', value: String(totalCs.value) },
  {
    label: '连杀',
    value: (stats.value?.largestKillingSpree ?? 0) > 0 ? String(stats.value?.largestKillingSpree) : '—'
  },
  { label: '多杀', value: multiKillLabel(stats.value?.largestMultiKill ?? 0) },
  { label: '推塔', value: String(stats.value?.turretKills ?? 0) },
  {
    label: '眼位',
    value: `${stats.value?.wardsPlaced ?? 0}/${stats.value?.wardsKilled ?? 0}`
  },
  { label: '承伤', value: formatCompact(stats.value?.totalDamageTaken || 0) },
  {
    label: '伤害',
    value: formatCompact(stats.value?.totalDamageDealtToChampions || 0),
    toneClass: props.isMaxDamage ? 'text-red-600 dark:text-red-400' : undefined
  }
])
</script>
