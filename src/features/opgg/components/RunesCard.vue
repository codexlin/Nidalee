<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Sparkles class="w-5 h-5" />
        推荐符文
      </CardTitle>
      <CardDescription>选择最适合的符文配置应用到游戏中</CardDescription>
    </CardHeader>
    <CardContent>
      <div v-if="perks && perks.length > 0" class="space-y-4">
        <div
          v-for="(rune, index) in perks.slice(0, 3)"
          :key="index"
          class="p-4 border rounded-lg hover:border-primary/50 transition-colors"
        >
          <div class="flex items-center justify-between mb-4">
            <div class="font-medium">
              <span class="text-base">符文方案</span>
              <span class="text-lg font-bold text-primary ml-1">{{ getRuneSchemeNumber(index) }}</span>
            </div>
            <Button @click="handleApplyRunes(index)" size="sm" variant="outline" class="flex items-center gap-2">
              <Wand2 class="w-4 h-4" />
              应用此符文
            </Button>
          </div>

          <!-- 符文图标显示 -->
          <div class="flex flex-wrap items-center gap-4">
            <!-- 主系符文 -->
            <div class="flex items-center gap-2">
              <Badge variant="secondary">主系</Badge>
              <div class="flex gap-1">
                <Tooltip v-for="runeId in rune.perks.slice(0, 4)" :key="runeId">
                  <TooltipTrigger as-child>
                    <img
                      :src="getRuneIconUrl(runeId)"
                      :alt="`符文 ${runeId}`"
                      class="w-8 h-8 rounded border hover:border-primary transition-colors cursor-help"
                      @error="onRuneImageError"
                    />
                  </TooltipTrigger>
                  <TooltipContent>
                    <p class="text-xs">符文 ID: {{ runeId }}</p>
                  </TooltipContent>
                </Tooltip>
              </div>
            </div>

            <!-- 副系符文 -->
            <div class="flex items-center gap-2">
              <Badge variant="outline">副系</Badge>
              <div class="flex gap-1">
                <Tooltip v-for="runeId in rune.perks.slice(4, 6)" :key="runeId">
                  <TooltipTrigger as-child>
                    <img
                      :src="getRuneIconUrl(runeId)"
                      :alt="`符文 ${runeId}`"
                      class="w-8 h-8 rounded border hover:border-primary transition-colors cursor-help"
                      @error="onRuneImageError"
                    />
                  </TooltipTrigger>
                  <TooltipContent>
                    <p class="text-xs">符文 ID: {{ runeId }}</p>
                  </TooltipContent>
                </Tooltip>
              </div>
            </div>

            <!-- 属性符文 -->
            <div v-if="rune.perks.length > 6" class="flex items-center gap-2">
              <Badge variant="secondary">属性</Badge>
              <div class="flex gap-1">
                <Tooltip v-for="runeId in rune.perks.slice(6, 9)" :key="runeId">
                  <TooltipTrigger as-child>
                    <img
                      :src="getRuneIconUrl(runeId)"
                      :alt="`符文 ${runeId}`"
                      class="w-6 h-6 rounded border hover:border-primary transition-colors cursor-help"
                      @error="onRuneImageError"
                    />
                  </TooltipTrigger>
                  <TooltipContent>
                    <p class="text-xs">属性 ID: {{ runeId }}</p>
                  </TooltipContent>
                </Tooltip>
              </div>
            </div>
          </div>

          <!-- 符文统计信息 -->
          <div class="mt-3 flex items-center gap-4 text-sm text-muted-foreground">
            <div class="flex items-center gap-1">
              <TrendingUp class="w-4 h-4" />
              <span
                >胜率:
                <span class="font-medium text-foreground">{{ formatPercentage(rune.win / rune.play) }}</span></span
              >
            </div>
            <div class="flex items-center gap-1">
              <BarChart3 class="w-4 h-4" />
              <span
                >选取率: <span class="font-medium text-foreground">{{ formatPercentage(rune.pickRate) }}</span></span
              >
            </div>
            <div class="flex items-center gap-1">
              <Target class="w-4 h-4" />
              <span
                ><span class="font-medium text-foreground">{{ rune.play }}</span> 局游戏</span
              >
            </div>
          </div>
        </div>
      </div>
      <div v-else class="text-center py-8 text-muted-foreground">
        <Sparkles class="w-12 h-12 mx-auto mb-2 opacity-50" />
        <p>暂无符文数据</p>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { Sparkles, Wand2, TrendingUp, BarChart3, Target } from 'lucide-vue-next'
import { getPerkIconUrlByCommunityDragon } from '@/lib'

// 使用后端生成的类型
interface Props {
  perks: OpggPerk[]
}

const props = defineProps<Props>()

const emit = defineEmits<{
  'apply-runes': [index: number]
}>()

// 获取Community Dragon符文数据
const { data: communityDragonPerks } = useCommunityDragonPerksQuery()

// 格式化百分比
const formatPercentage = (value: number): string => {
  return (value * 100).toFixed(1) + '%'
}

// 获取符文图标URL - 使用Community Dragon API
const getRuneIconUrl = (runeId: number): string => {
  if (!communityDragonPerks.value) {
    return ''
  }
  return getPerkIconUrlByCommunityDragon(runeId, communityDragonPerks.value)
}

// 符文图标错误处理
const onRuneImageError = (event: Event) => {
  const img = event.target as HTMLImageElement
  // 直接使用默认图标
  img.src =
    'data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMzIiIGhlaWdodD0iMzIiIHZpZXdCb3g9IjAgMCAzMiAzMiIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHJlY3Qgd2lkdGg9IjMyIiBoZWlnaHQ9IjMyIiByeD0iNCIgZmlsbD0iIzY2NjY2NiIvPgo8dGV4dCB4PSIxNiIgeT0iMjAiIGZvbnQtZmFtaWx5PSJBcmlhbCIgZm9udC1zaXplPSIxMiIgZmlsbD0id2hpdGUiIHRleHQtYW5jaG9yPSJtaWRkbGUiPj88L3RleHQ+Cjwvc3ZnPgo='
}

// 处理符文应用
const handleApplyRunes = (index: number) => {
  emit('apply-runes', index)
}

// 获取符文方案编号
const getRuneSchemeNumber = (index: number): string => {
  switch (index) {
    case 0:
      return '一'
    case 1:
      return '二'
    case 2:
      return '三'
    default:
      return `${index + 1}`
  }
}
</script>
