<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Sword class="w-5 h-5" />
        推荐装备
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-4">
      <!-- 起始装备 -->
      <div v-if="items.startItems?.length" class="space-y-3">
        <h4 class="font-medium mb-2 flex items-center gap-2">
          <Package class="w-4 h-4" />
          起始装备
        </h4>
        <div
          v-for="(item, index) in items.startItems.slice(0, 3)"
          :key="index"
          class="flex items-center gap-4 p-3 bg-muted rounded-lg mb-1"
        >
          <!-- 图标区，固定宽度，居中 -->
          <div class="flex gap-1 justify-center min-w-[96px]">
            <div v-for="itemId in item.ids" :key="itemId">
              <Tooltip>
                <TooltipTrigger as-child>
                  <img
                    :src="getItemIconUrl(itemId, gameVersion)"
                    :alt="getItemName(itemId)"
                    class="w-8 h-8 rounded border border-green-500 hover:ring-2 hover:ring-primary transition"
                    @error="onItemImageError"
                  />
                </TooltipTrigger>
                <TooltipContent
                  v-if="allItems[itemId]"
                  class="z-50 min-w-[220px] max-w-xs p-3 rounded-lg shadow-xl bg-white/90 dark:bg-neutral-800/95 border border-neutral-200 dark:border-neutral-700 text-xs text-left transition"
                >
                  <div class="font-bold text-base mb-1 text-primary dark:text-blue-300">
                    {{ allItems[itemId].name }}
                  </div>
                  <div class="mb-2 text-neutral-700 dark:text-neutral-200" v-html="allItems[itemId].description"></div>
                  <div class="text-xs text-muted-foreground mt-1">
                    <span>售价：{{ allItems[itemId].gold?.total }}</span>
                    <span class="ml-2">合成价：{{ allItems[itemId].gold?.base }}</span>
                  </div>
                </TooltipContent>
              </Tooltip>
            </div>
          </div>
          <!-- 文本区，始终左对齐 -->
          <div class="flex-1">
            <div class="font-medium">
              <span class="text-base">起始装备</span>
              <span class="text-lg font-bold text-green-600 ml-1">{{ getStartItemSchemeNumber(index) }}</span>
            </div>
            <div class="text-xs text-muted-foreground mt-1">
              {{ item.ids.map(getItemName).join(' ') }}
            </div>
            <div class="text-sm text-muted-foreground mt-1">
              胜率: {{ formatPercentage(item.win / item.play) }} | 选取率: {{ formatPercentage(item.pickRate) }}
            </div>
            <div class="text-xs text-muted-foreground mt-1">{{ item.play }} 局游戏</div>
          </div>
        </div>
      </div>

      <!-- 核心装备 -->
      <div v-if="items.coreItems?.length" class="space-y-3">
        <h4 class="font-medium mb-2 flex items-center gap-2">
          <Target class="w-4 h-4" />
          核心装备
        </h4>
        <div
          v-for="(item, index) in items.coreItems.slice(0, 3)"
          :key="index"
          class="flex items-center gap-4 p-3 bg-muted rounded-lg mb-1"
        >
          <!-- 图标区，固定宽度，居中 -->
          <div class="flex gap-1 justify-center min-w-[96px]">
            <div v-for="itemId in item.ids" :key="itemId">
              <Tooltip>
                <TooltipTrigger as-child>
                  <img
                    :src="getItemIconUrl(itemId, gameVersion)"
                    :alt="getItemName(itemId)"
                    class="w-8 h-8 rounded border border-purple-500 hover:ring-2 hover:ring-primary transition"
                    @error="onItemImageError"
                  />
                </TooltipTrigger>
                <TooltipContent
                  v-if="allItems[itemId]"
                  class="z-50 min-w-[220px] max-w-xs p-3 rounded-lg shadow-xl bg-white/90 dark:bg-neutral-800/95 border border-neutral-200 dark:border-neutral-700 text-xs text-left transition"
                >
                  <div class="font-bold text-base mb-1 text-primary dark:text-blue-300">
                    {{ allItems[itemId].name }}
                  </div>
                  <div class="mb-2 text-neutral-700 dark:text-neutral-200" v-html="allItems[itemId].description"></div>
                  <div class="text-xs text-muted-foreground mt-1">
                    <span>售价：{{ allItems[itemId].gold?.total }}</span>
                    <span class="ml-2">合成价：{{ allItems[itemId].gold?.base }}</span>
                  </div>
                </TooltipContent>
              </Tooltip>
            </div>
          </div>
          <!-- 文本区，始终左对齐 -->
          <div class="flex-1">
            <div class="font-medium">
              <span class="text-base">核心装备</span>
              <span class="text-lg font-bold text-purple-600 ml-1">{{ getCoreItemSchemeNumber(index) }}</span>
            </div>
            <div class="text-xs text-muted-foreground mt-1">
              {{ item.ids.map(getItemName).join(' ') }}
            </div>
            <div class="text-sm text-muted-foreground mt-1">
              胜率: {{ formatPercentage(item.win / item.play) }} | 选取率: {{ formatPercentage(item.pickRate) }}
            </div>
            <div class="text-xs text-muted-foreground mt-1">{{ item.play }} 局游戏</div>
          </div>
        </div>
      </div>

      <!-- 备选装备 -->
      <div v-if="items.lastItems?.length" class="space-y-3">
        <h4 class="font-medium mb-2 flex items-center gap-2">
          <Wand2 class="w-4 h-4" />
          备选装备
        </h4>
        <div class="flex flex-row flex-wrap gap-2 p-2 bg-muted rounded-lg">
          <div v-for="(item, index) in filteredLastItems" :key="index" class="flex gap-1 justify-center">
            <div v-for="itemId in item.ids" :key="itemId">
              <Tooltip>
                <TooltipTrigger as-child>
                  <img
                    :src="getItemIconUrl(itemId, gameVersion)"
                    :alt="getItemName(itemId)"
                    class="w-8 h-8 rounded-full border border-yellow-500 hover:ring-2 hover:ring-primary transition"
                    @error="onItemImageError"
                  />
                </TooltipTrigger>
                <TooltipContent
                  v-if="allItems[itemId]"
                  class="z-50 min-w-[220px] max-w-xs p-3 rounded-lg shadow-xl bg-white/90 dark:bg-neutral-800/95 border border-neutral-200 dark:border-neutral-700 text-xs text-left transition"
                >
                  <div class="font-bold text-base mb-1 text-primary dark:text-blue-300">
                    {{ allItems[itemId].name }}
                  </div>
                  <div class="mb-2 text-neutral-700 dark:text-neutral-200" v-html="allItems[itemId].description"></div>
                  <div class="text-xs text-muted-foreground mt-1">
                    <span>售价：{{ allItems[itemId].gold?.total }}</span>
                    <span class="ml-2">合成价：{{ allItems[itemId].gold?.base }}</span>
                  </div>
                </TooltipContent>
              </Tooltip>
            </div>
          </div>
        </div>
      </div>

      <!-- 鞋子推荐 -->
      <div v-if="items.boots?.length" class="space-y-3">
        <h4 class="font-medium mb-2 flex items-center gap-2">
          <Footprints class="w-4 h-4" />
          鞋子推荐
        </h4>
        <div
          v-for="(boot, index) in items.boots.slice(0, 3)"
          :key="index"
          class="flex items-center gap-4 p-3 bg-muted rounded-lg mb-1"
        >
          <!-- 图标区，固定宽度，居中 -->
          <div class="flex gap-1 justify-center min-w-[96px]">
            <div v-for="itemId in boot.ids" :key="itemId">
              <Tooltip>
                <TooltipTrigger as-child>
                  <img
                    :src="getItemIconUrl(itemId, gameVersion)"
                    :alt="getItemName(itemId)"
                    class="w-8 h-8 rounded border border-blue-500 hover:ring-2 hover:ring-primary transition"
                    @error="onItemImageError"
                  />
                </TooltipTrigger>
                <TooltipContent
                  v-if="allItems[itemId]"
                  class="z-50 min-w-[220px] max-w-xs p-3 rounded-lg shadow-xl bg-white/90 dark:bg-neutral-800/95 border border-neutral-200 dark:border-neutral-700 text-xs text-left transition"
                >
                  <div class="font-bold text-base mb-1 text-primary dark:text-blue-300">
                    {{ allItems[itemId].name }}
                  </div>
                  <div class="mb-2 text-neutral-700 dark:text-neutral-200" v-html="allItems[itemId].description"></div>
                  <div class="text-xs text-muted-foreground mt-1">
                    <span>售价：{{ allItems[itemId].gold?.total }}</span>
                    <span class="ml-2">合成价：{{ allItems[itemId].gold?.base }}</span>
                  </div>
                </TooltipContent>
              </Tooltip>
            </div>
          </div>
          <!-- 文本区，始终左对齐 -->
          <div class="flex-1">
            <div class="font-medium">
              <span class="text-base">鞋子选择</span>
              <span class="text-lg font-bold text-blue-600 ml-1">{{ getBootSchemeNumber(index) }}</span>
            </div>
            <div class="text-xs text-muted-foreground mt-1">
              {{ boot.ids.map(getItemName).join(' ') }}
            </div>
            <div class="text-sm text-muted-foreground mt-1">
              胜率: {{ formatPercentage(boot.win / boot.play) }} | 选取率: {{ formatPercentage(boot.pickRate) }}
            </div>
            <div class="text-xs text-muted-foreground mt-1">{{ boot.play }} 局游戏</div>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { Sword, Package, Target, Footprints, Wand2 } from 'lucide-vue-next'
import { getItemIconUrl, getAllItems } from '@/lib'
import type { DDragonItem } from '@/lib/dataApi'

// 使用后端生成的类型
interface Props {
  items: OpggItems
}

const props = defineProps<Props>()

// 获取数据存储
const dataStore = useDataStore()
const gameVersion = computed(() => dataStore.gameVersion)

// 拉取所有物品详细信息
const allItems = ref<Record<string, DDragonItem>>({})

onMounted(async () => {
  if (gameVersion.value) {
    const data = await getAllItems(gameVersion.value)
    // DDragon 返回格式为 { data: { "1001": {...}, ... } }
    allItems.value = data.data || {}
  }
})

// 获取装备名称（优先用 DDragon 详细信息）
const getItemName = (itemId: number): string => {
  return allItems.value?.[String(itemId)]?.name || `物品${itemId}`
}

// 格式化百分比
const formatPercentage = (value: number): string => {
  return (value * 100).toFixed(1) + '%'
}

// 装备图标错误处理
const onItemImageError = (event: Event) => {
  const img = event.target as HTMLImageElement
  img.src =
    'data:image/svg+xml;utf8,<svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg"><rect width="32" height="32" rx="6" fill="%23e5e7eb"/><text x="16" y="22" text-anchor="middle" font-size="20" fill="%239ca3af" font-family="Arial, Helvetica, sans-serif">?</text></svg>'
}

// 计算 coreItems 中所有 id 的集合
const coreItemIds = computed(() => (props.items.coreItems || []).map((item) => item.id))
// 过滤、排序后的备选装备
const filteredLastItems = computed(() => {
  return (props.items.lastItems || [])
    .filter((item) => !coreItemIds.value.includes(item.id))
    .sort((a, b) => b.pickRate - a.pickRate)
})

// 获取起始装备方案的编号
const getStartItemSchemeNumber = (index: number): string => {
  switch (index) {
    case 0:
      return '一'
    case 1:
      return '二'
    case 2:
      return '三'
    default:
      return ''
  }
}

// 获取核心装备方案的编号
const getCoreItemSchemeNumber = (index: number): string => {
  switch (index) {
    case 0:
      return '一'
    case 1:
      return '二'
    case 2:
      return '三'
    default:
      return ''
  }
}

// 获取鞋子选择方案的编号
const getBootSchemeNumber = (index: number): string => {
  switch (index) {
    case 0:
      return '一'
    case 1:
      return '二'
    case 2:
      return '三'
    default:
      return ''
  }
}
</script>
