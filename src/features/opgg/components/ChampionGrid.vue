<template>
  <aside
    class="flex h-full min-h-0 w-[18.5rem] shrink-0 flex-col overflow-hidden border-r border-border/60 sm:w-[20.5rem]"
  >
    <div class="shrink-0 space-y-2 border-b border-border/60 p-3">
      <div class="relative">
        <Search class="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
        <Input
          v-model="query"
          class="h-8 pl-8 text-sm"
          placeholder="搜索英雄…"
          aria-label="搜索英雄"
        />
      </div>
      <div class="flex flex-wrap gap-0.5 rounded-full surface-inset p-0.5">
        <button
          v-for="tag in roleFilters"
          :key="tag.value"
          type="button"
          class="rounded-full px-2.5 py-1 text-sm font-medium transition-colors"
          :class="
            activeRole === tag.value
              ? 'bg-primary/15 text-primary'
              : 'text-muted-foreground hover:text-foreground'
          "
          @click="activeRole = tag.value"
        >
          {{ tag.label }}
        </button>
      </div>
    </div>

    <ScrollArea class="h-full min-h-0 flex-1 overflow-hidden">
      <div class="grid grid-cols-5 gap-1.5 p-2">
        <button
          v-for="champ in filtered"
          :key="champ.id"
          type="button"
          class="flex flex-col items-center gap-1 rounded-xl p-1 text-center transition-colors hover:bg-muted/50 focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
          :class="selectedId === champ.id && 'bg-primary/10 ring-1 ring-primary/30'"
          :title="champ.name"
          @click="emit('select', champ.id)"
        >
          <img
            :src="getChampionIconUrl(champ.id)"
            alt=""
            class="size-9 rounded-lg ring-1 ring-border/50"
            loading="lazy"
          />
          <span class="w-full truncate text-xs leading-tight text-muted-foreground">{{ champ.name }}</span>
        </button>
      </div>
      <p v-if="!filtered.length" class="px-3 py-8 text-center text-xs text-muted-foreground">没有匹配英雄</p>
    </ScrollArea>
  </aside>
</template>

<script setup lang="ts">
import { Search } from 'lucide-vue-next'
import { getAllChampions, getChampionIconUrl, isStandardDDragonChampion } from '@/lib'
import type { DDragonChampion } from '@/lib/dataApi'

const props = defineProps<{
  selectedId?: number | null
}>()

const emit = defineEmits<{
  select: [championId: number]
}>()

const dataStore = useDataStore()
const gameVersion = computed(() => dataStore.gameVersion)
const champions = ref<Array<{ id: number; name: string; tags: string[] }>>([])
const query = ref('')
const activeRole = ref('all')

const roleFilters = [
  { value: 'all', label: '全部' },
  { value: 'Fighter', label: '战' },
  { value: 'Tank', label: '坦' },
  { value: 'Mage', label: '法' },
  { value: 'Assassin', label: '刺' },
  { value: 'Marksman', label: '射' },
  { value: 'Support', label: '辅' }
] as const

const loadChampions = async (version?: string) => {
  if (!version) return
  const data = await getAllChampions(version)
  const list = Object.values(data.data || {}) as DDragonChampion[]
  champions.value = list
    .filter(isStandardDDragonChampion)
    .map((c) => ({
      id: Number(c.key),
      name: c.name,
      tags: c.tags || []
    }))
    .sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'))
}

onMounted(() => void loadChampions(gameVersion.value))
watch(gameVersion, (v) => void loadChampions(v))

const filtered = computed(() => {
  const q = query.value.trim().toLowerCase()
  return champions.value.filter((c) => {
    if (activeRole.value !== 'all' && !c.tags.includes(activeRole.value)) return false
    if (!q) return true
    return c.name.toLowerCase().includes(q) || String(c.id).includes(q)
  })
})
</script>
