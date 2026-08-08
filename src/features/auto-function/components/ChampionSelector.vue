<template>
  <div class="space-y-6">
    <div class="relative">
      <Search class="absolute left-4 top-1/2 transform -translate-y-1/2 h-5 w-5 text-muted-foreground" />
      <Input
        v-model="searchText"
        placeholder="搜索英雄名称或别名..."
        class="text-foreground pl-12 h-12 text-base bg-background/50 border-border/50 focus:border-primary/50 focus:bg-background transition-all duration-200 shadow-sm focus:shadow-md"
      />
      <div v-if="searchText" class="absolute right-3 top-1/2 transform -translate-y-1/2">
        <button
          @click="searchText = ''"
          class="h-7 w-7 rounded-full bg-secondary hover:bg-destructive/10 hover:text-destructive flex items-center justify-center transition-all duration-200 border border-border hover:border-destructive/30 group"
          title="清除搜索"
        >
          <X class="h-4 w-4 text-muted-foreground group-hover:text-destructive transition-colors duration-200" />
        </button>
      </div>
    </div>

    <div v-if="!loading && !error" class="flex items-center justify-between text-sm text-muted-foreground px-1">
      <span>{{ filteredChampions.length }} / {{ champions.length }} 个英雄</span>
      <div class="flex items-center gap-4">
        <span v-if="searchText" class="text-primary font-medium">搜索: "{{ searchText }}"</span>
        <span class="text-xs opacity-75">按 ESC 键关闭</span>
      </div>
    </div>

    <ScrollArea class="h-[min(600px,calc(85vh-200px))] w-full rounded-lg border border-border bg-muted/20">
      <div class="p-6">
        <div v-if="loading" class="flex flex-col items-center justify-center py-16 text-center">
          <div class="h-10 w-10 animate-spin rounded-full border-3 border-primary border-t-transparent mb-6"></div>
          <p class="text-muted-foreground font-medium text-lg">正在加载英雄数据...</p>
          <p class="text-sm text-muted-foreground mt-2">请稍候片刻</p>
        </div>

        <div v-else-if="error" class="flex flex-col items-center justify-center py-16 text-center">
          <div
            class="h-16 w-16 rounded-full bg-destructive/10 flex items-center justify-center mb-6 border border-destructive/20"
          >
            <Search class="h-8 w-8 text-destructive" />
          </div>
          <p class="text-destructive font-semibold text-lg mb-2">加载失败</p>
          <p class="text-sm text-muted-foreground mb-6 max-w-md">{{ error }}</p>
          <button
            @click="loadChampions"
            class="px-6 py-3 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-all duration-200 text-sm font-medium shadow-sm hover:shadow-md"
          >
            重新加载
          </button>
        </div>

        <div v-else>
          <div class="grid grid-cols-[repeat(auto-fill,minmax(100px,1fr))] gap-3">
            <div
              v-for="champion in filteredChampions"
              :key="champion.id"
              class="flex flex-col items-center p-2 rounded-xl cursor-pointer transition-all duration-300 hover:bg-accent/50 active:scale-95 group border border-transparent hover:border-primary/30"
              @click="selectChampion(champion)"
            >
              <div class="relative transform-gpu transition-transform duration-300 hover:scale-110">
                <Avatar
                  class="h-16 w-16 border-2 border-transparent group-hover:border-primary transition-all duration-300 shadow-md group-hover:shadow-lg"
                >
                  <AvatarImage
                    :src="getChampionIconUrlByAlias(champion.alias)"
                    :alt="champion.name"
                    class="object-cover"
                  />
                  <AvatarFallback
                    class="text-sm font-semibold bg-gradient-to-br from-primary/10 to-primary/20 text-primary border border-primary/30"
                  >
                    {{ champion.name.slice(0, 2).toUpperCase() }}
                  </AvatarFallback>
                </Avatar>
                <div
                  class="absolute inset-0 rounded-full bg-primary/20 opacity-0 group-hover:opacity-100 transition-opacity duration-300 -z-10 blur-sm"
                ></div>
              </div>
              <div class="w-full text-center mt-2">
                <span
                  class="text-xs text-foreground font-medium leading-tight inline-block w-full overflow-hidden text-ellipsis whitespace-nowrap group-hover:text-primary transition-colors duration-200"
                  :title="champion.name"
                >
                  {{ champion.name }}
                </span>
              </div>
            </div>
          </div>

          <!-- no result -->
          <div
            v-if="!loading && !error && filteredChampions.length === 0"
            class="flex flex-col items-center justify-center py-20 text-center"
          >
            <div
              class="h-20 w-20 rounded-full bg-muted flex items-center justify-center mb-6 border-2 border-dashed border-muted-foreground/30"
            >
              <Search class="h-10 w-10 text-muted-foreground/50" />
            </div>
            <p class="text-muted-foreground font-semibold text-lg mb-2">没有找到匹配的英雄</p>
            <p class="text-sm text-muted-foreground">尝试使用不同的关键词搜索，如英雄名称或别名</p>
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
</template>

<script setup lang="ts">
import { getChampionIconUrlByAlias } from '@/lib'
import { fetchChampionSummary } from '@/lib/dataApi'
import { Search, X } from 'lucide-vue-next'

interface Emits {
  (e: 'select', champion: ChampionInfo): void
}

const emit = defineEmits<Emits>()

const searchText = ref('')
const champions = ref<ChampionInfo[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

const filteredChampions = computed(() => {
  if (!searchText.value.trim()) {
    return champions.value
  }

  const search = searchText.value.toLowerCase()
  return champions.value.filter(
    (champion) =>
      champion.name.toLowerCase().includes(search) || champion.alias.toLowerCase().includes(search)
  )
})

const selectChampion = (champion: ChampionInfo) => {
  emit('select', champion)
}

const loadChampions = async () => {
  try {
    loading.value = true
    error.value = null

    console.log('🚀 开始获取英雄数据...')
    const response = await fetchChampionSummary()

    if (response.success && response.data) {
      // 将 Community Dragon 的英雄数据转换为 ChampionInfo 格式
      const championList: ChampionInfo[] = response.data
        .filter((champion) => champion.id > 0) // 过滤掉 id 为 -1 的"无"选项
        .map((champion) => ({
          id: champion.id,
          name: champion.name,
          alias: champion.alias,
          squarePortraitPath: champion.squarePortraitPath,
          roles: champion.roles
        }))
        .filter((champion) => !champion.alias.includes('Ruby_'))
        .sort((a, b) => a.name.localeCompare(b.name))

      champions.value = championList
      console.log(`✅ 成功加载 ${championList.length} 个英雄`)
    } else {
      throw new Error(response.error || '获取英雄数据失败')
    }
  } catch (err) {
    console.error('❌ 加载英雄数据失败:', err)
    error.value = err instanceof Error ? err.message : '未知错误'
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  loadChampions()
})
</script>
