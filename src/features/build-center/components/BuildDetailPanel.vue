<template>
  <Card class="gap-0 overflow-hidden p-0 py-0">
    <div class="flex flex-wrap items-center gap-3 border-b border-border/60 px-4 py-3 sm:px-5">
      <FloatIconButton class="p-2" title="返回强度榜" aria-label="返回强度榜" @click="emit('back')">
        <ArrowLeft class="size-4" />
      </FloatIconButton>
      <img :src="getChampionIconUrl(summary.championId)" alt="" class="size-11 rounded-xl ring-1 ring-border" />
      <div class="min-w-0">
        <h2 class="text-lg font-medium leading-tight">{{ getChampionName(summary.championId) }}</h2>
        <p class="mt-0.5 text-xs text-muted-foreground">
          {{ positionLabel }}
          <template v-if="summary.rank">
            <span class="mx-1 text-border">·</span>
            <span class="tabular-nums">#{{ summary.rank }}</span>
          </template>
        </p>
      </div>
      <StatKpiStrip :items="kpis" />
    </div>

    <WorkbenchSection title="推荐符文" show-rates action-slot>
      <div v-if="build.perks?.length" class="overflow-hidden rounded-xl surface-inset">
        <div
          v-for="(rune, index) in build.perks.slice(0, 3)"
          :key="index"
          class="grid items-center gap-2 border-b border-border/40 px-2.5 py-2 last:border-b-0 sm:grid-cols-[1.5rem_minmax(0,1fr)_auto] sm:gap-3 sm:px-3"
        >
          <span class="flex size-5 items-center justify-center rounded bg-primary/10 text-xs font-bold text-primary">
            {{ index + 1 }}
          </span>

          <div class="flex min-w-0 flex-wrap items-start gap-1.5">
            <div
              v-for="id in rune.perks.slice(0, 4)"
              :key="`p-${index}-${id}`"
              class="flex w-11 flex-col items-center gap-0.5"
              :title="runeName(id)"
            >
              <img
                v-if="runeIcon(id)"
                :src="runeIcon(id)"
                alt=""
                class="size-7 rounded-md ring-1 ring-border/50"
                @error="fadeImg"
                @load="clearImgFade"
              />
              <span class="w-full truncate text-center text-xs leading-tight text-muted-foreground">
                {{ runeName(id) }}
              </span>
            </div>
            <span class="mx-0.5 mt-1.5 h-4 w-px shrink-0 bg-border/60" />
            <div
              v-for="id in rune.perks.slice(4, 6)"
              :key="`s-${index}-${id}`"
              class="flex w-10 flex-col items-center gap-0.5"
              :title="runeName(id)"
            >
              <img
                v-if="runeIcon(id)"
                :src="runeIcon(id)"
                alt=""
                class="size-6 rounded-md ring-1 ring-border/50"
                @error="fadeImg"
                @load="clearImgFade"
              />
              <span class="w-full truncate text-center text-xs leading-tight text-muted-foreground">
                {{ runeName(id) }}
              </span>
            </div>
            <span class="mx-0.5 mt-1.5 h-4 w-px shrink-0 bg-border/60" />
            <div
              v-for="id in rune.perks.slice(6, 9)"
              :key="`a-${index}-${id}`"
              class="flex w-9 flex-col items-center gap-0.5"
              :title="runeName(id)"
            >
              <img
                v-if="runeIcon(id)"
                :src="runeIcon(id)"
                alt=""
                class="size-4 rounded ring-1 ring-border/40"
                @error="fadeImg"
                @load="clearImgFade"
              />
              <span class="w-full truncate text-center text-xs leading-tight text-muted-foreground">
                {{ runeName(id) }}
              </span>
            </div>
          </div>

          <RateColumns :win-rate="safeRate(rune.win, rune.play)" :pick-rate="rune.pickRate" :games="rune.play">
            <div class="flex items-center gap-1">
              <Button
                v-if="canSaveRunes"
                size="sm"
                variant="outline"
                class="h-7 px-2 text-xs"
                @click="emit('save-runes', index)"
              >
                <BookmarkPlus class="size-3" />
                保存
              </Button>
              <Button size="sm" class="h-7 px-2.5 text-xs" @click="emit('apply-runes', index)">
                <Wand2 class="size-3" />
                应用
              </Button>
            </div>
          </RateColumns>
        </div>
      </div>
      <p v-if="build.perks?.length && !canSaveRunes" class="pt-2 text-xs text-muted-foreground">
        当前模式支持直接应用，但不参与自动方案匹配。
      </p>
      <p v-if="!build.perks?.length" class="py-4 text-center text-sm text-muted-foreground">
        {{ mode === 'arena' ? '竞技场模式无符文方案，请看下方出装与强化相关数据' : '暂无符文数据' }}
      </p>
    </WorkbenchSection>

    <div>
      <button
        type="button"
        :aria-expanded="detailOpen"
        aria-controls="build-detail-sections"
        class="flex w-full items-center justify-between gap-2 border-b border-border/50 px-4 py-2.5 text-left hover:bg-muted/25 sm:px-5"
        @click="detailOpen = !detailOpen"
      >
        <span class="text-lg font-medium leading-tight">技能 · 出装 · 克制</span>
        <span class="flex items-center gap-1 text-xs text-muted-foreground">
          {{ detailOpen ? '收起' : '展开' }}
          <ChevronDown class="size-3.5 transition-transform" :class="detailOpen && 'rotate-180'" />
        </span>
      </button>

      <div id="build-detail-sections" v-show="detailOpen" class="space-y-3 px-4 py-3 sm:px-5">
        <div class="grid gap-3 lg:grid-cols-2">
          <div class="surface-inset rounded-xl p-3">
            <div class="mb-2 flex items-center justify-between">
              <h4 class="text-sm font-medium">召唤师技能</h4>
              <div class="flex gap-3 text-sm text-muted-foreground">
                <span class="w-10 text-right">胜率</span>
                <span class="w-10 text-right">选取</span>
              </div>
            </div>
            <div class="space-y-0.5">
              <div
                v-for="(spell, i) in (build.summonerSpells || []).slice(0, 3)"
                :key="i"
                class="grid grid-cols-[minmax(0,1fr)_2.5rem_2.5rem] items-center gap-2 rounded-md px-1 py-1.5 hover:bg-muted/30"
              >
                <div class="flex min-w-0 items-center gap-1.5">
                  <img
                    v-for="sid in spell.ids"
                    :key="sid"
                    :src="getSpellMeta(sid).icon"
                    :alt="getSpellMeta(sid).label"
                    class="size-6 rounded-md ring-1 ring-border/50"
                    @error="fadeImg"
                  />
                  <span class="truncate text-sm font-medium">
                    {{ spell.ids.map((id) => getSpellMeta(id).label).join(' + ') }}
                  </span>
                </div>
                <span class="text-right text-sm font-medium tabular-nums text-sky-600 dark:text-sky-400">
                  {{ pct(safeRate(spell.win, spell.play)) }}
                </span>
                <span class="text-right text-sm tabular-nums text-muted-foreground">{{ pct(spell.pickRate) }}</span>
              </div>
            </div>
          </div>

          <div class="surface-inset rounded-xl p-3">
            <div class="mb-2 flex flex-wrap items-baseline justify-between gap-2">
              <h4 class="text-sm font-medium">技能加点</h4>
              <p class="text-xs tabular-nums text-muted-foreground">
                <span class="font-medium text-foreground">
                  {{ skills.masteries?.length ? skills.masteries.join(' → ') : '—' }}
                </span>
                <span class="mx-1.5 text-border">·</span>
                <span class="font-medium text-sky-600 dark:text-sky-400">{{
                  pct(safeRate(skills.win, skills.play))
                }}</span>
                <span class="mx-1 text-border">·</span>
                {{ pct(skills.pickRate) }} 选取
              </p>
            </div>
            <div class="flex flex-wrap gap-px">
              <span
                v-for="(skill, li) in skills.order.slice(0, 18)"
                :key="li"
                class="inline-flex size-5 items-center justify-center text-xs font-bold tabular-nums first:rounded-l-md last:rounded-r-md"
                :class="skillTone(skill)"
                :title="`Lv.${li + 1}`"
              >
                {{ skill }}
              </span>
            </div>
          </div>
        </div>

        <div class="space-y-2">
          <div class="flex items-center justify-between">
            <h4 class="text-sm font-medium">装备路线</h4>
            <div class="flex gap-3 text-sm text-muted-foreground">
              <span class="w-12 text-right">胜率</span>
              <span class="w-12 text-right">选取</span>
            </div>
          </div>
          <div class="grid gap-2 md:grid-cols-2">
            <div v-for="section in earlyItemSections" :key="section.key" class="space-y-1">
              <p class="px-0.5 text-xs font-medium text-muted-foreground">{{ section.title }}</p>
              <ItemRows :rows="section.rows" />
            </div>
          </div>
          <div v-for="section in lateItemSections" :key="section.key" class="space-y-1">
            <p class="px-0.5 text-xs font-medium text-muted-foreground">{{ section.title }}</p>
            <ItemRows :rows="section.rows" />
          </div>
        </div>

        <div class="space-y-2">
          <h4 class="text-sm font-medium">克制关系</h4>
          <div class="grid gap-2 md:grid-cols-2">
            <div class="surface-inset rounded-xl p-2.5">
              <p class="mb-1.5 px-1 text-xs font-medium text-emerald-600 dark:text-emerald-400">优势对局</p>
              <div
                v-for="c in strongCounters"
                :key="`s-${c.championId}`"
                class="grid grid-cols-[minmax(0,1fr)_3.25rem] items-center gap-2 rounded-md px-1 py-1 hover:bg-muted/30"
              >
                <div class="flex min-w-0 items-center gap-1.5">
                  <img
                    :src="getChampionIconUrl(c.championId)"
                    alt=""
                    class="size-6 shrink-0 rounded-md ring-1 ring-border/50"
                  />
                  <span class="truncate text-sm font-medium">{{ getChampionName(c.championId) }}</span>
                </div>
                <span class="text-right text-sm font-medium tabular-nums text-emerald-600 dark:text-emerald-400">
                  {{ pct(c.winRate) }}
                </span>
              </div>
              <p v-if="!strongCounters.length" class="px-1 py-2 text-xs text-muted-foreground">暂无</p>
            </div>
            <div class="surface-inset rounded-xl p-2.5">
              <p class="mb-1.5 px-1 text-xs font-medium text-rose-600 dark:text-rose-400">劣势对局</p>
              <div
                v-for="c in weakCounters"
                :key="`w-${c.championId}`"
                class="grid grid-cols-[minmax(0,1fr)_3.25rem] items-center gap-2 rounded-md px-1 py-1 hover:bg-muted/30"
              >
                <div class="flex min-w-0 items-center gap-1.5">
                  <img
                    :src="getChampionIconUrl(c.championId)"
                    alt=""
                    class="size-6 shrink-0 rounded-md ring-1 ring-border/50"
                  />
                  <span class="truncate text-sm font-medium">{{ getChampionName(c.championId) }}</span>
                </div>
                <span class="text-right text-sm font-medium tabular-nums text-rose-600 dark:text-rose-400">
                  {{ pct(c.winRate) }}
                </span>
              </div>
              <p v-if="!weakCounters.length" class="px-1 py-2 text-xs text-muted-foreground">暂无</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ArrowLeft, BookmarkPlus, ChevronDown, Wand2 } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'
import { getChampionIconUrl, getChampionName, getPerkIconUrlByCommunityDragon, getSpellMeta } from '@/lib'
import ItemRows from './ItemRows.vue'
import WorkbenchSection from './WorkbenchSection.vue'
import StatKpiStrip from './StatKpiStrip.vue'
import RateColumns from './RateColumns.vue'
import type { StatKpiItem } from './StatKpiStrip.vue'

const props = withDefaults(
  defineProps<{
    build: OpggChampionBuild
    mode?: string
    canSaveRunes?: boolean
  }>(),
  { mode: 'ranked', canSaveRunes: true }
)

const emit = defineEmits<{
  back: []
  'apply-runes': [index: number]
  'save-runes': [index: number]
}>()

const { data: communityDragonPerks } = useCommunityDragonPerksQuery()

const detailOpen = ref(true)
const summary = computed(() => props.build.summary)
const skills = computed(() => props.build.championSkills)

watch(
  () => summary.value.championId,
  () => {
    detailOpen.value = true
  }
)

const positionMap: Record<string, string> = {
  TOP: '上单',
  JUNGLE: '打野',
  MID: '中单',
  ADC: '下路',
  SUPPORT: '辅助',
  none: '大乱斗',
  aram: '大乱斗',
  urf: '无限火力',
  arena: '斗魂竞技场'
}
const positionLabel = computed(() => {
  if (props.mode === 'aram') return '大乱斗'
  if (props.mode === 'urf') return '无限火力'
  if (props.mode === 'arena') return '斗魂竞技场'
  return positionMap[summary.value.position] || summary.value.position || '未知分路'
})

const pct = (v?: number | null) => (v === null || v === undefined || Number.isNaN(v) ? '—' : `${(v * 100).toFixed(1)}%`)
const safeRate = (win: number, play: number) => (play > 0 ? win / play : 0)

const kpis = computed((): StatKpiItem[] => {
  const rateLabel = props.mode === 'arena' ? '吃鸡' : '胜率'
  const items: StatKpiItem[] = [
    { label: rateLabel, value: pct(summary.value.winRate) },
    { label: '选取', value: pct(summary.value.pickRate) }
  ]
  if (props.mode === 'ranked' || props.mode === 'arena') {
    items.push({ label: '禁用', value: pct(summary.value.banRate) })
  }
  items.push({
    label: 'KDA',
    value: summary.value.kda === null || summary.value.kda === undefined ? '—' : summary.value.kda.toFixed(2)
  })
  return items
})

const perkById = computed(() => {
  const map = new Map<number, { name: string }>()
  for (const perk of communityDragonPerks.value ?? []) map.set(perk.id, perk)
  return map
})

const runeIcon = (id: number) =>
  communityDragonPerks.value ? getPerkIconUrlByCommunityDragon(id, communityDragonPerks.value) : ''

const runeName = (id: number) => perkById.value.get(id)?.name || `符文${id}`

const skillTone = (skill: string) => {
  const s = skill.toUpperCase()
  if (s === 'Q') return 'bg-sky-500/20 text-sky-700 dark:text-sky-300'
  if (s === 'W') return 'bg-violet-500/20 text-violet-700 dark:text-violet-300'
  if (s === 'E') return 'bg-amber-500/20 text-amber-800 dark:text-amber-300'
  if (s === 'R') return 'bg-rose-500/20 text-rose-700 dark:text-rose-300'
  return 'bg-muted text-muted-foreground'
}

const coreIds = computed(() => new Set((props.build.items?.coreItems || []).map((i) => i.id)))

const earlyItemSections = computed(() =>
  [
    { key: 'start', title: '出门装备', rows: (props.build.items?.startItems || []).slice(0, 3) },
    { key: 'boots', title: '鞋子', rows: (props.build.items?.boots || []).slice(0, 3) }
  ].filter((s) => s.rows.length)
)

const lateItemSections = computed(() =>
  [
    { key: 'core', title: '核心装备', rows: (props.build.items?.coreItems || []).slice(0, 4) },
    {
      key: 'last',
      title: '后续装备',
      rows: (props.build.items?.lastItems || [])
        .filter((i) => !coreIds.value.has(i.id))
        .sort((a, b) => b.pickRate - a.pickRate)
        .slice(0, 6)
    }
  ].filter((s) => s.rows.length)
)

const strongCounters = computed(() => (props.build.counters?.strongAgainst || []).slice(0, 6))
const weakCounters = computed(() => (props.build.counters?.weakAgainst || []).slice(0, 6))

const fadeImg = (e: Event) => {
  ;(e.target as HTMLImageElement).style.opacity = '0.35'
}

const clearImgFade = (e: Event) => {
  ;(e.target as HTMLImageElement).style.opacity = ''
}
</script>
