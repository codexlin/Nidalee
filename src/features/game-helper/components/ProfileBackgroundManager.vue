<template>
  <component :is="embedded ? 'div' : Card" :class="embedded ? 'space-y-3' : 'gap-0 py-0'">
    <template v-if="!embedded">
      <CardHeader class="gap-1 px-4 py-3 sm:px-5">
        <CardTitle class="flex items-center gap-2 text-base font-medium">
          <Users class="size-4 shrink-0 text-muted-foreground" />
          生涯背景
        </CardTitle>
        <p class="text-xs text-muted-foreground">选择英雄并设置皮肤为生涯背景</p>
      </CardHeader>
    </template>

    <component :is="embedded ? 'div' : CardContent" :class="embedded ? '' : 'px-4 pb-4 sm:px-5'">
      <div v-if="!selectedChampion" class="space-y-3">
        <div class="relative">
          <Search
            class="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-muted-foreground"
          />
          <Input
            v-model="searchText"
            class="h-9 pl-8 text-sm"
            placeholder="搜索英雄名称或别名…"
            aria-label="搜索英雄"
          />
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

        <div class="overflow-hidden rounded-xl surface-inset">
          <ScrollArea class="h-[min(520px,calc(85vh-240px))] w-full">
            <div class="p-2">
              <div
                v-if="loadingChampions"
                class="grid grid-cols-[repeat(auto-fill,3.25rem)] justify-start gap-1"
              >
                <Skeleton v-for="i in 40" :key="i" class="size-[3.25rem] rounded-lg" />
              </div>

              <div
                v-else-if="championsError"
                class="flex flex-col items-center justify-center gap-3 py-16 text-center"
              >
                <div class="flex size-12 items-center justify-center rounded-xl bg-destructive/10">
                  <Search class="size-5 text-destructive" />
                </div>
                <div class="space-y-1">
                  <p class="text-base font-medium text-destructive">加载失败</p>
                  <p class="max-w-md text-sm text-muted-foreground">{{ championsError.message }}</p>
                </div>
                <Button size="sm" class="h-9" @click="() => reloadChampions()">重新加载</Button>
              </div>

              <template v-else>
                <div class="grid grid-cols-[repeat(auto-fill,3.25rem)] justify-start gap-1">
                  <button
                    v-for="champion in filteredChampions"
                    :key="champion.id"
                    type="button"
                    class="flex w-[3.25rem] flex-col items-center gap-0.5 rounded-lg p-0.5 text-center transition-colors hover:bg-muted/50 focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50"
                    :title="champion.name"
                    @click="handleChampionSelect(champion)"
                  >
                    <img
                      :src="getChampionIconUrl(champion.id)"
                      :alt="champion.name"
                      class="size-10 rounded-lg object-cover ring-1 ring-border/50"
                      loading="lazy"
                    />
                    <span class="w-full truncate text-[10px] leading-tight text-muted-foreground">{{
                      champion.name
                    }}</span>
                  </button>
                </div>

                <div
                  v-if="!filteredChampions.length"
                  class="flex flex-col items-center justify-center gap-2 py-16 text-center"
                >
                  <div class="flex size-12 items-center justify-center rounded-xl bg-muted/40">
                    <Search class="size-5 text-muted-foreground" />
                  </div>
                  <p class="text-base font-medium">没有找到匹配的英雄</p>
                  <p class="text-sm text-muted-foreground">试试名称或英文别名</p>
                </div>
              </template>
            </div>
          </ScrollArea>
        </div>
      </div>

      <div v-else class="space-y-4">
        <div class="flex flex-wrap items-center gap-3">
          <Button variant="outline" size="sm" class="h-8" @click="clearChampion">
            <ArrowLeft class="size-3.5" />
            返回列表
          </Button>
          <h2 class="text-lg font-medium">{{ selectedChampion.name }}</h2>
        </div>

        <div v-if="loadingSkins" class="space-y-3">
          <div class="grid grid-cols-1 gap-3 md:grid-cols-2">
            <Skeleton v-for="i in 6" :key="i" class="aspect-[1.7] w-full rounded-xl" />
          </div>
          <p class="text-center text-sm text-muted-foreground">正在加载皮肤数据…</p>
        </div>

        <div
          v-else-if="skinsError"
          class="flex flex-col items-center justify-center gap-3 py-16 text-center"
        >
          <div class="flex size-12 items-center justify-center rounded-xl bg-destructive/10">
            <X class="size-5 text-destructive" />
          </div>
          <div class="space-y-1">
            <p class="text-base font-medium text-destructive">加载失败</p>
            <p class="max-w-md text-sm text-muted-foreground">{{ skinsError.message }}</p>
          </div>
          <Button size="sm" class="h-9" @click="() => reloadSkins()">重新加载</Button>
        </div>

        <template v-else>
          <div class="columns-1 gap-10 md:columns-2">
            <div
              v-for="(skin, idx) in championSkins"
              :id="'skin-card-' + skin.id"
              :key="skin.id"
              :style="getCard3DStyle(idx)"
              class="group relative mb-10 break-inside-avoid cursor-pointer overflow-hidden rounded-3xl border-2 border-transparent bg-gradient-to-br from-background/80 to-background/60 shadow-2xl transition-all duration-300 hover:z-20 hover:scale-105 hover:border-primary/80 hover:shadow-2xl hover:tw-animate-tilt hover:tw-animate-glow transform-gpu group-hover:[transform:perspective(800px)_rotateX(8deg)_rotateY(8deg)_scale3d(1.04,1.04,1.04)]"
              :class="[
                { 'pointer-events-none opacity-70': applyingSkinId === skin.id },
                shakeSkinId === skin.id ? 'animate-shake' : ''
              ]"
              @click="applySkinBackground(skin)"
            >
              <img
                :src="getSkinImageUrl(skin)"
                :alt="skin.name"
                class="aspect-[1.7] w-full object-cover transition-transform duration-500 group-hover:scale-110"
              />
              <div
                class="absolute inset-0 z-10 bg-black/40 transition-colors duration-300 group-hover:bg-black/20"
              />

              <div
                v-if="applyingSkinId === skin.id"
                class="absolute inset-0 z-30 flex items-center justify-center rounded-3xl bg-primary/80"
              >
                <LoadingSpinner />
                <span class="ml-3 text-sm font-medium text-white">正在应用…</span>
              </div>

              <div class="absolute right-0 bottom-0 left-0 z-20 flex flex-col items-start px-4 py-2">
                <span class="text-base font-bold text-white drop-shadow-lg">{{ skin.name }}</span>
                <span
                  v-if="skin.isBase"
                  class="mt-1 rounded-full bg-primary/80 px-3 py-0.5 text-xs font-medium text-white shadow"
                >
                  经典
                </span>
              </div>
              <div
                class="pointer-events-none absolute inset-0 rounded-3xl transition-all duration-300 group-hover:ring-4 group-hover:ring-primary/40"
              />
            </div>
          </div>

          <div
            v-if="!championSkins.length"
            class="flex flex-col items-center justify-center gap-2 py-16 text-center"
          >
            <div class="flex size-12 items-center justify-center rounded-xl surface-inset">
              <Users class="size-5 text-muted-foreground" />
            </div>
            <p class="text-base font-medium">暂无皮肤数据</p>
            <p class="text-sm text-muted-foreground">该英雄暂时没有可用的皮肤数据</p>
          </div>
        </template>
      </div>
    </component>
  </component>
</template>

<script setup lang="ts">
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { useGameHelper } from '../composables/useGameHelper'
import { getChampionIconUrl, isStandardChampion } from '@/lib'
import type { CommunityDragonSkin } from '@/lib/dataApi'
import { ArrowLeft, Search, Users, X } from 'lucide-vue-next'
import { debounce } from 'radash'

withDefaults(
  defineProps<{
    embedded?: boolean
  }>(),
  { embedded: false }
)

const { setSummonerBackgroundSkin } = useGameHelper()
const searchText = ref('')
const debouncedSearchText = ref('')
const {
  data: championsData,
  isLoading: loadingChampions,
  error: championsError,
  refetch: reloadChampions
} = useChampions()
const champions = computed<ChampionInfo[]>(() =>
  championsData.value
    ? championsData.value.filter(isStandardChampion).sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'))
    : []
)
const selectedChampion = ref<ChampionInfo | null>(null)
const selectedChampionId = computed(() => selectedChampion.value?.id ?? null)
const {
  data: championDetails,
  isLoading: loadingSkins,
  error: skinsError,
  refetch: reloadSkins
} = useChampionDetails(selectedChampionId)
const championSkins = computed<CommunityDragonSkin[]>(() => championDetails.value?.skins ?? [])
const applyingSkinId = ref<number | null>(null)
const shakeSkinId = ref<number | null>(null)

const debouncedUpdateSearch = debounce({ delay: 300 }, (value: string) => {
  debouncedSearchText.value = value
})

watch(searchText, (newValue) => {
  debouncedUpdateSearch(newValue)
})

const filteredChampions = computed(() => {
  if (!debouncedSearchText.value.trim()) return champions.value
  const search = debouncedSearchText.value.toLowerCase()
  return champions.value.filter((c) => c.name.toLowerCase().includes(search) || c.alias.toLowerCase().includes(search))
})

const handleChampionSelect = (champion: ChampionInfo) => {
  selectedChampion.value = champion
}

const clearChampion = () => {
  selectedChampion.value = null
}

const getSkinImageUrl = (skin: CommunityDragonSkin): string => {
  if (!selectedChampion.value?.alias) return ''
  const skinNum = skin.id % 1000
  return `https://ddragon.leagueoflegends.com/cdn/img/champion/splash/${selectedChampion.value.alias}_${skinNum}.jpg`
}

const applySkinBackground = async (skin: CommunityDragonSkin) => {
  try {
    applyingSkinId.value = skin.id
    shakeSkinId.value = skin.id
    await setSummonerBackgroundSkin(skin.id, skin.name)
  } finally {
    setTimeout(() => {
      shakeSkinId.value = null
      applyingSkinId.value = null
    }, 600)
  }
}

const getCard3DStyle = (idx: number) => {
  const rotateX = ((idx % 4) - 1.5) * 6
  const rotateY = ((idx % 6) - 2.5) * 5
  return {
    willChange: 'transform, opacity',
    transform: `perspective(800px) rotateX(${rotateX}deg) rotateY(${rotateY}deg) scale3d(1,1,1)`
  }
}

const LoadingSpinner = defineComponent({
  name: 'LoadingSpinner',
  setup() {
    return () =>
      h('div', { class: 'flex items-center justify-center' }, [
        h('span', {
          class:
            'inline-block size-8 animate-spin rounded-full border-4 border-primary border-t-transparent'
        })
      ])
  }
})
</script>
