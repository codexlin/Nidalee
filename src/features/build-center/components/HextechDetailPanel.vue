<template>
  <Card class="gap-0 overflow-hidden p-0 py-0">
    <div class="flex flex-wrap items-center gap-3 border-b border-border/60 px-4 py-3 sm:px-5">
      <FloatIconButton class="p-2" title="返回强度榜" aria-label="返回强度榜" @click="emit('back')">
        <ArrowLeft class="size-4" />
      </FloatIconButton>
      <img
        :src="summary.iconUrl || getChampionIconUrl(summary.championId)"
        alt=""
        class="size-11 rounded-xl ring-1 ring-border"
      />
      <div class="min-w-0">
        <h2 class="text-lg font-medium leading-tight">
          {{ summary.name || getChampionName(summary.championId) }}
        </h2>
        <p class="mt-0.5 text-xs text-muted-foreground">
          海克斯
          <template v-if="summary.gamePatch">
            <span class="mx-1 text-border">·</span>
            <span class="tabular-nums">{{ summary.gamePatch }}</span>
          </template>
          <template v-if="summary.roles?.length">
            <span class="mx-1 text-border">·</span>
            {{ summary.roles.map(roleLabel).join(' / ') }}
          </template>
        </p>
      </div>
      <StatKpiStrip :items="kpis" />
    </div>

    <WorkbenchSection v-if="guideTrios.length" title="推荐三连" show-rates>
      <HextechTrioList :trios="guideTrios" variant="workbench" />
    </WorkbenchSection>

    <WorkbenchSection title="推荐增强">
      <HextechAugmentGroups :augments="detail.augments" variant="workbench" />
    </WorkbenchSection>

    <div>
      <button
        type="button"
        class="flex w-full items-center justify-between gap-2 border-b border-border/50 px-4 py-2.5 text-left hover:bg-muted/25 sm:px-5"
        @click="detailOpen = !detailOpen"
      >
        <span class="text-lg font-medium leading-tight">技能 · 出装</span>
        <span class="flex items-center gap-1 text-xs text-muted-foreground">
          {{ detailOpen ? '收起' : '展开' }}
          <ChevronDown class="size-3.5 transition-transform" :class="detailOpen && 'rotate-180'" />
        </span>
      </button>

      <div v-show="detailOpen" class="space-y-3 px-4 py-3 sm:px-5">
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
                v-for="(spell, i) in detail.summonerSpells.slice(0, 3)"
                :key="i"
                class="grid grid-cols-[minmax(0,1fr)_2.5rem_2.5rem] items-center gap-2 rounded-md px-1 py-1.5 hover:bg-muted/30"
              >
                <div class="flex min-w-0 items-center gap-1.5">
                  <img
                    v-for="sid in spell.spellIds"
                    :key="sid"
                    :src="getSpellMeta(sid).icon"
                    :alt="getSpellMeta(sid).label"
                    class="size-6 rounded-md ring-1 ring-border/50"
                    @error="fadeImg"
                  />
                  <span class="truncate text-sm font-medium">
                    {{ spell.spellIds.map((id) => getSpellMeta(id).label).join(' + ') }}
                  </span>
                </div>
                <span class="text-right text-sm font-medium tabular-nums text-sky-600 dark:text-sky-400">
                  {{ pct(spell.winRate) }}
                </span>
                <span class="text-right text-sm tabular-nums text-muted-foreground">{{ pct(spell.pickRate) }}</span>
              </div>
              <p v-if="!detail.summonerSpells.length" class="px-1 py-2 text-xs text-muted-foreground">暂无</p>
            </div>
          </div>

          <div class="surface-inset rounded-xl p-3">
            <div class="mb-2 flex flex-wrap items-baseline justify-between gap-2">
              <h4 class="text-sm font-medium">技能加点</h4>
              <p v-if="bestSkill" class="text-xs tabular-nums text-muted-foreground">
                <span class="font-medium text-sky-600 dark:text-sky-400">{{ pct(bestSkill.winRate) }}</span>
                <span class="mx-1 text-border">·</span>
                {{ pct(bestSkill.pickRate) }} 选取
              </p>
            </div>
            <div v-if="bestSkill" class="flex flex-wrap gap-px">
              <span
                v-for="(skill, li) in bestSkill.skillKeys.slice(0, 18)"
                :key="li"
                class="inline-flex size-5 items-center justify-center text-xs font-bold tabular-nums first:rounded-l-md last:rounded-r-md"
                :class="skillTone(skill)"
                :title="`Lv.${li + 1}`"
              >
                {{ skill }}
              </span>
            </div>
            <p v-else class="py-2 text-xs text-muted-foreground">暂无</p>
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
            <div class="space-y-1">
              <p class="px-0.5 text-xs font-medium text-muted-foreground">出门装</p>
              <ItemRows :rows="toItemRows(detail.startingItems)" />
            </div>
            <div class="space-y-1">
              <p class="px-0.5 text-xs font-medium text-muted-foreground">核心装</p>
              <ItemRows :rows="toItemRows(detail.coreItems)" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { ArrowLeft, ChevronDown } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'
import HextechAugmentGroups from '@/shared/components/hextech/HextechAugmentGroups.vue'
import HextechTrioList from '@/shared/components/hextech/HextechTrioList.vue'
import type { HextechGuideTrio } from '@/shared/hextech/guideAugment'
import { getChampionIconUrl, getChampionName, getSpellMeta } from '@/lib'
import ItemRows from './ItemRows.vue'
import WorkbenchSection from './WorkbenchSection.vue'
import StatKpiStrip from './StatKpiStrip.vue'
import type { StatKpiItem } from './StatKpiStrip.vue'

const props = defineProps<{
  detail: HextechChampionDetail
}>()

const emit = defineEmits<{
  back: []
}>()

const detailOpen = ref(true)
const summary = computed(() => props.detail.summary)
const bestSkill = computed(() => props.detail.skillOrders[0] ?? null)

const augmentById = computed(() => {
  const map = new Map<number, HextechAugmentStat>()
  for (const a of props.detail.augments) map.set(a.id, a)
  return map
})

const guideTrios = computed((): HextechGuideTrio[] =>
  (props.detail.augmentTrios ?? []).map((trio) => ({
    augments: trio.augmentIds.map((id) => {
      const stat = augmentById.value.get(id)
      if (stat) return stat
      return {
        id,
        name: `#${id}`,
        iconUrl: '',
        rarityName: '',
        rarityDisplayName: '',
        winRate: 0,
        pickRate: 0,
        games: null,
        tier: null
      }
    }),
    winRate: trio.winRate,
    pickRate: trio.pickRate,
    games: trio.games
  }))
)

watch(
  () => summary.value.championId,
  () => {
    detailOpen.value = true
  }
)

const kpis = computed((): StatKpiItem[] => [
  { label: '胜率', value: pct(summary.value.winRate) },
  { label: '选取', value: pct(summary.value.pickRate) },
  {
    label: 'Tier',
    value: summary.value.tier && summary.value.tier > 0 ? String(summary.value.tier) : '—'
  }
])

const pct = (n: number | null | undefined) => `${((n ?? 0) * 100).toFixed(1)}%`

const fadeImg = (e: Event) => {
  const el = e.target as HTMLImageElement
  el.style.opacity = '0.3'
}

const roleLabel = (name: string) => {
  const map: Record<string, string> = {
    tank: '坦克',
    fighter: '战士',
    mage: '法师',
    assassin: '刺客',
    marksman: '射手',
    support: '辅助'
  }
  return map[name.toLowerCase()] || name
}

const skillTone = (skill: string) => {
  if (skill === 'Q') return 'bg-sky-500/20 text-sky-700 dark:text-sky-300'
  if (skill === 'W') return 'bg-emerald-500/20 text-emerald-700 dark:text-emerald-300'
  if (skill === 'E') return 'bg-amber-500/20 text-amber-700 dark:text-amber-300'
  if (skill === 'R') return 'bg-rose-500/20 text-rose-700 dark:text-rose-300'
  return 'bg-muted text-muted-foreground'
}

const toItemRows = (combos: HextechItemCombo[]): OpggItem[] =>
  combos.map((c) => ({
    id: c.itemIds[0] ?? 0,
    ids: c.itemIds,
    icons: [],
    win: c.wins,
    play: c.games,
    pickRate: c.pickRate
  }))
</script>
