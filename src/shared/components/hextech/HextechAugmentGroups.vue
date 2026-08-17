<script setup lang="ts">
import {
  HEXTECH_RARITY_ORDER,
  filterRecommendedAugments,
  formatAugmentWinRate,
  groupAugmentsByRarity,
  groupAugmentsByTier,
  hasSampledWinRate,
  type HextechGuideAugment,
  type HextechRarityKey
} from '@/shared/hextech/guideAugment'

const props = withDefaults(
  defineProps<{
    augments: HextechGuideAugment[]
    variant?: 'overlay' | 'workbench'
  }>(),
  { variant: 'workbench' }
)

const compact = computed(() => props.variant === 'overlay')
const showAll = ref(false)
const activeKey = ref<HextechRarityKey>('prismatic')
const expanded = ref<Record<string, boolean>>({})

const allGroups = computed(() => groupAugmentsByRarity(props.augments))
const hiddenCount = computed(() =>
  allGroups.value.reduce((count, group) => {
    const after = groupAugmentsByTier(group.items).reduce(
      (sum, band) => sum + filterRecommendedAugments(band.items).length,
      0
    )
    return count + group.items.length - after
  }, 0)
)

const tabGroups = computed(() =>
  allGroups.value
    .map((group) => ({
      ...group,
      shown: groupAugmentsByTier(group.items).reduce(
        (sum, band) => sum + filterRecommendedAugments(band.items).length,
        0
      )
    }))
    .filter((group) => !compact.value || group.shown > 0)
)

const rarityBoards = computed(() => {
  const source = compact.value
    ? tabGroups.value.filter(
        (group) =>
          group.key ===
          (tabGroups.value.some((item) => item.key === activeKey.value) ? activeKey.value : tabGroups.value[0]?.key)
      )
    : HEXTECH_RARITY_ORDER.map((key) => allGroups.value.find((group) => group.key === key)).filter(
        (group): group is NonNullable<typeof group> => Boolean(group)
      )

  return source.map((group) => {
    const bands = groupAugmentsByTier(group.items)
      .map((band) => {
        const items = showAll.value && !compact.value ? band.items : filterRecommendedAugments(band.items)
        const expandKey = `${group.key}-${band.tier}`
        const limit =
          band.tier === 3 || band.tier === 4 || band.tier === 0 ? (compact.value ? 8 : 10) : compact.value ? 4 : 6
        const open = Boolean(expanded.value[expandKey])
        const shown = open ? items : items.slice(0, limit)
        return {
          ...band,
          expandKey,
          items,
          shown,
          hidden: Math.max(0, items.length - shown.length),
          asIcons: band.tier === 3 || band.tier === 4 || band.tier === 0
        }
      })
      .filter((band) => band.items.length > 0)

    return {
      key: group.key,
      label: group.label,
      total: bands.reduce((sum, band) => sum + band.items.length, 0),
      bands
    }
  })
})

const overlayItems = computed(() => rarityBoards.value[0]?.bands.flatMap((band) => band.items) ?? [])

const tierBadge = (item: HextechGuideAugment) =>
  item.tier === 1 || item.tier === 2 || item.tier === 3 || item.tier === 4 ? `T${item.tier}` : ''

const tierBadgeTone: Record<1 | 2 | 3 | 4, string> = {
  1: 'bg-rose-500/95 text-white ring-rose-300/40',
  2: 'bg-amber-400/95 text-amber-950 ring-amber-200/40',
  3: 'bg-sky-500/90 text-white ring-sky-300/35',
  4: 'bg-slate-500/90 text-white ring-slate-300/30'
}

const tierTone = (item: HextechGuideAugment) =>
  item.tier === 1 || item.tier === 2 || item.tier === 3 || item.tier === 4
    ? tierBadgeTone[item.tier]
    : 'bg-black/80 text-white ring-white/10'

watch(
  tabGroups,
  (next) => {
    if (!next.some((group) => group.key === activeKey.value)) {
      activeKey.value = next[0]?.key ?? 'prismatic'
    }
  },
  { immediate: true }
)

const rarityTone: Record<HextechRarityKey, string> = {
  prismatic: 'text-rose-700 dark:text-rose-300',
  gold: 'text-amber-800 dark:text-amber-200',
  silver: 'text-slate-600 dark:text-slate-300',
  other: 'text-muted-foreground'
}

const rarityBar: Record<HextechRarityKey, string> = {
  prismatic: 'bg-rose-400',
  gold: 'bg-amber-400',
  silver: 'bg-slate-400',
  other: 'bg-muted-foreground/50'
}

const rarityFrame: Record<HextechRarityKey, string> = {
  prismatic: 'ring-rose-400/35 bg-rose-500/5',
  gold: 'ring-amber-400/35 bg-amber-500/5',
  silver: 'ring-slate-400/30 bg-slate-500/5',
  other: 'ring-border/50 bg-muted/20'
}

const rarityFill: Record<HextechRarityKey, string> = {
  prismatic: 'bg-rose-400',
  gold: 'bg-amber-400',
  silver: 'bg-slate-400',
  other: 'bg-muted-foreground'
}

const fadeImg = (event: Event) => {
  const el = event.target as HTMLImageElement
  el.style.opacity = '0.3'
}

const toggleRest = (key: string) => {
  expanded.value = { ...expanded.value, [key]: !expanded.value[key] }
}

const cardTitle = (item: HextechGuideAugment) => {
  const tier = tierBadge(item)
  return tier
    ? `${item.name} · ${tier} · ${formatAugmentWinRate(item)} 胜率`
    : `${item.name} · ${formatAugmentWinRate(item)} 胜率`
}
</script>

<template>
  <div v-if="allGroups.length" class="flex flex-col" :class="compact ? 'gap-2' : 'gap-3'">
    <div v-if="!compact && hiddenCount > 0" class="flex justify-end">
      <button type="button" class="text-xs text-muted-foreground hover:text-foreground" @click="showAll = !showAll">
        {{ showAll ? '隐藏低胜率' : `已隐藏 ${hiddenCount} 个低胜率` }}
      </button>
    </div>
    <div v-if="tabGroups.length > 1" class="flex gap-0.5 rounded-lg bg-black/25 p-0.5" :class="!compact && 'hidden'">
      <button
        v-for="group in tabGroups"
        :key="group.key"
        type="button"
        class="flex-1 rounded-md px-2 py-1 text-[11px] font-medium transition-colors"
        :class="
          activeKey === group.key
            ? `${rarityTone[group.key]} bg-white/10`
            : 'text-muted-foreground hover:text-foreground'
        "
        @click="activeKey = group.key"
      >
        {{ group.label }}
      </button>
    </div>

    <div v-if="compact">
      <div v-if="overlayItems.length" class="grid grid-cols-4 gap-x-1.5 gap-y-2">
        <div v-for="item in overlayItems" :key="item.id" class="min-w-0" :title="cardTitle(item)">
          <div class="relative mx-auto w-10">
            <img
              v-if="item.iconUrl"
              :src="item.iconUrl"
              :alt="item.name"
              class="size-10 rounded-md ring-1 ring-inset"
              :class="rarityFrame[activeKey]"
              @error="fadeImg"
            />
            <span v-else class="block size-10 rounded-md bg-white/10" />
            <span
              v-if="tierBadge(item)"
              class="pointer-events-none absolute -right-1 -top-1 rounded px-0.5 py-px text-[8px] font-bold leading-none ring-1 ring-inset"
              :class="tierTone(item)"
            >
              {{ tierBadge(item) }}
            </span>
            <span
              class="pointer-events-none absolute inset-x-0 bottom-0 rounded-b-md bg-black/70 px-0.5 text-center text-[9px] tabular-nums text-white"
            >
              {{ hasSampledWinRate(item) ? Math.round(item.winRate * 100) : '—' }}
            </span>
          </div>
          <p class="mt-1 line-clamp-2 text-center text-[10px] leading-tight text-foreground/85">
            {{ item.name }}
          </p>
        </div>
      </div>
      <p v-else class="py-3 text-center text-xs text-muted-foreground">该品质暂无达到门槛的增强</p>
    </div>

    <div v-else class="grid grid-cols-1 gap-3 lg:grid-cols-3">
      <section v-for="group in rarityBoards" :key="group.key" class="min-w-0">
        <div class="mb-2 flex items-center gap-2">
          <span class="h-3 w-0.5 rounded-full" :class="rarityBar[group.key]" />
          <h3 class="text-sm font-medium" :class="rarityTone[group.key]">{{ group.label }}</h3>
          <span class="text-[10px] tabular-nums text-muted-foreground">{{ group.total }}</span>
        </div>

        <div v-if="group.bands.length" class="flex flex-col gap-3">
          <div v-for="band in group.bands" :key="band.expandKey">
            <p class="mb-1.5 text-[10px] font-medium tracking-wide text-muted-foreground">
              {{ band.label }}
              <span class="tabular-nums opacity-70">{{ band.items.length }}</span>
            </p>

            <div v-if="band.asIcons" class="flex flex-wrap gap-1.5">
              <div v-for="item in band.shown" :key="item.id" class="relative" :title="cardTitle(item)">
                <img
                  v-if="item.iconUrl"
                  :src="item.iconUrl"
                  :alt="item.name"
                  class="size-9 rounded-md ring-1 ring-inset"
                  :class="rarityFrame[group.key]"
                  @error="fadeImg"
                />
                <span
                  class="pointer-events-none absolute inset-x-0 bottom-0 rounded-b-md bg-black/70 px-0.5 text-center text-[9px] tabular-nums text-white"
                >
                  {{ hasSampledWinRate(item) ? Math.round(item.winRate * 100) : '—' }}
                </span>
              </div>
            </div>

            <div v-else class="grid grid-cols-2 gap-1.5">
              <article
                v-for="item in band.shown"
                :key="item.id"
                class="rounded-xl p-2 ring-1 ring-inset"
                :class="rarityFrame[group.key]"
                :title="cardTitle(item)"
              >
                <div class="flex items-start gap-2">
                  <div class="relative shrink-0">
                    <img
                      v-if="item.iconUrl"
                      :src="item.iconUrl"
                      :alt="item.name"
                      class="size-10 rounded-lg ring-1 ring-inset ring-white/10"
                      @error="fadeImg"
                    />
                    <span
                      class="absolute -right-1 -top-1 rounded bg-background/90 px-1 text-[9px] font-semibold tabular-nums"
                      :class="rarityTone[group.key]"
                    >
                      {{ band.label }}
                    </span>
                  </div>
                  <div class="min-w-0 flex-1">
                    <p class="line-clamp-2 text-[11px] font-medium leading-tight text-foreground">{{ item.name }}</p>
                    <p class="mt-1 text-sm font-semibold tabular-nums" :class="rarityTone[group.key]">
                      {{ formatAugmentWinRate(item) }}
                    </p>
                  </div>
                </div>
                <div class="mt-2 h-0.5 overflow-hidden rounded-full bg-black/20 dark:bg-white/10">
                  <div
                    class="h-full rounded-full"
                    :class="rarityFill[group.key]"
                    :style="{
                      width: `${hasSampledWinRate(item) ? Math.min(100, Math.max(0, item.winRate * 100)) : 0}%`
                    }"
                  />
                </div>
              </article>
            </div>

            <button
              v-if="band.hidden > 0 || expanded[band.expandKey]"
              type="button"
              class="mt-1.5 text-[11px] text-muted-foreground hover:text-foreground"
              @click="toggleRest(band.expandKey)"
            >
              {{ expanded[band.expandKey] ? '收起' : `还有 ${band.hidden} 个` }}
            </button>
          </div>
        </div>
        <p v-else class="text-xs text-muted-foreground">该品质暂无达到门槛的增强</p>
      </section>
    </div>
  </div>
  <p v-else class="py-4 text-center text-sm text-muted-foreground">暂无增强数据</p>
</template>
