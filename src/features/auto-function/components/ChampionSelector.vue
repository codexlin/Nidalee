<template>
  <div class="space-y-3">
    <div class="relative">
      <Search class="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground" />
      <Input v-model="searchText" class="h-9 pl-8 text-sm" placeholder="搜索英雄名称或别名…" aria-label="搜索英雄" />
      <button
        v-if="searchText"
        type="button"
        class="absolute top-1/2 right-2 flex size-7 -translate-y-1/2 items-center justify-center rounded-lg text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
        title="清除搜索"
        @click="searchText = ''"
      >
        <X class="size-3.5" />
      </button>
    </div>

    <div
      v-if="!showInitialLoading && !errorMessage"
      class="flex items-center justify-between px-0.5 text-xs text-muted-foreground"
    >
      <span class="tabular-nums">{{ filteredChampions.length }} / {{ champions.length }} 个英雄</span>
      <span v-if="searchText" class="truncate text-primary">搜索：{{ searchText }}</span>
    </div>

    <div class="overflow-hidden rounded-xl surface-inset">
      <ScrollArea class="h-[min(520px,calc(85vh-220px))] w-full">
        <div class="p-3">
          <div v-if="showInitialLoading" class="flex flex-col items-center justify-center gap-2 py-16 text-center">
            <div class="size-8 animate-spin rounded-full border-2 border-primary border-t-transparent" />
            <p class="text-sm text-muted-foreground">加载英雄列表…</p>
          </div>

          <div v-else-if="errorMessage" class="flex flex-col items-center justify-center gap-3 py-16 text-center">
            <p class="text-sm font-medium text-destructive">加载失败</p>
            <p class="max-w-md text-xs text-muted-foreground">{{ errorMessage }}</p>
            <Button size="sm" class="h-8" @click="() => refetch()">重新加载</Button>
          </div>

          <template v-else>
            <div class="grid grid-cols-[repeat(auto-fill,minmax(4.5rem,1fr))] gap-1.5">
              <button
                v-for="champion in filteredChampions"
                :key="champion.id"
                type="button"
                class="flex flex-col items-center gap-0.5 rounded-lg p-1 text-center transition-colors hover:bg-muted/50 focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
                :title="champion.name"
                @click="emit('select', champion)"
              >
                <img
                  :src="getChampionIconUrl(champion.id)"
                  :alt="champion.name"
                  class="size-12 rounded-lg object-cover ring-1 ring-border/50"
                  loading="lazy"
                  draggable="false"
                />
                <span class="w-full truncate text-[11px] leading-tight text-muted-foreground">{{ champion.name }}</span>
              </button>
            </div>

            <div
              v-if="!filteredChampions.length"
              class="flex flex-col items-center justify-center gap-1 py-16 text-center"
            >
              <p class="text-sm font-medium">没有找到匹配的英雄</p>
              <p class="text-xs text-muted-foreground">试试名称或英文别名</p>
            </div>
          </template>
        </div>
      </ScrollArea>
    </div>
  </div>
</template>

<script setup lang="ts">
import { getChampionIconUrl, isStandardChampion } from '@/lib'
import { Search, X } from 'lucide-vue-next'

const emit = defineEmits<{
  select: [champion: ChampionInfo]
}>()

const searchText = ref('')
const { data: championsData, isLoading, isPending, error, refetch } = useChampions()

const champions = computed(() =>
  (championsData.value ?? [])
    .filter(isStandardChampion)
    .slice()
    .sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'))
)

/** 仅无缓存时显示全屏 loading；有缓存时即时展示 */
const showInitialLoading = computed(() => (isPending.value || isLoading.value) && champions.value.length === 0)

const errorMessage = computed(() => {
  if (champions.value.length) return null
  const err = error.value
  return err ? (err instanceof Error ? err.message : String(err)) : null
})

const filteredChampions = computed(() => {
  const q = searchText.value.trim().toLowerCase()
  if (!q) return champions.value
  return champions.value.filter((c) => c.name.toLowerCase().includes(q) || c.alias.toLowerCase().includes(q))
})
</script>
