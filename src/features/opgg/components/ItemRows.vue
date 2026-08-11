<template>
  <div class="overflow-hidden rounded-lg surface-inset">
    <div
      v-for="(item, index) in rows"
      :key="index"
      class="grid grid-cols-[minmax(0,1fr)_3.5rem_3.5rem] items-center gap-2 border-b border-border/40 px-2.5 py-1.5 last:border-b-0"
    >
      <div class="flex min-w-0 items-center gap-1.5">
        <Tooltip v-for="(itemId, ii) in item.ids" :key="`${index}-${ii}-${itemId}`">
          <TooltipTrigger as-child>
            <img
              :src="getItemIconUrl(itemId, gameVersion)"
              alt=""
              class="size-7 shrink-0 rounded-md ring-1 ring-border/50"
              @error="onError"
            />
          </TooltipTrigger>
          <TooltipContent class="text-xs">{{ nameOf(itemId) }}</TooltipContent>
        </Tooltip>
        <span class="truncate text-sm font-medium" :title="labelOf(item.ids)">{{ labelOf(item.ids) }}</span>
      </div>
      <span class="text-right text-sm font-medium tabular-nums text-sky-600 dark:text-sky-400">
        {{ pct(item.play > 0 ? item.win / item.play : 0) }}
      </span>
      <span class="text-right text-sm tabular-nums text-muted-foreground">{{ pct(item.pickRate) }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getItemIconUrl, getAllItems } from '@/lib'
import type { DDragonItem } from '@/lib/dataApi'

defineProps<{
  rows: OpggItem[]
}>()

const dataStore = useDataStore()
const gameVersion = computed(() => dataStore.gameVersion)
const allItems = ref<Record<string, DDragonItem>>({})

const loadItems = async (v?: string) => {
  if (!v) return
  const data = await getAllItems(v)
  allItems.value = data.data || {}
}

onMounted(() => loadItems(gameVersion.value))
watch(gameVersion, (v) => loadItems(v))

const nameOf = (id: number) => allItems.value?.[String(id)]?.name || `物品${id}`

/** 多件套装：名称用 + 连接；连续相同物品合并为 ×N（如 多兰之戒 + 生命药水 ×2） */
const labelOf = (ids: number[]) => {
  if (!ids.length) return '—'
  const parts: { name: string; count: number }[] = []
  for (const id of ids) {
    const name = nameOf(id)
    const last = parts[parts.length - 1]
    if (last && last.name === name) last.count += 1
    else parts.push({ name, count: 1 })
  }
  return parts.map((p) => (p.count > 1 ? `${p.name} ×${p.count}` : p.name)).join(' + ')
}

const pct = (n: number) => `${(n * 100).toFixed(1)}%`
const onError = (e: Event) => {
  ;(e.target as HTMLImageElement).style.opacity = '0.35'
}
</script>
