<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-3">
        <Avatar class="w-12 h-12">
          <AvatarImage
            v-if="summary.championId"
            :src="getChampionIconUrl(summary.championId)"
            :alt="getChampionName(summary.championId)"
            @error="onChampionImageError"
          />
          <AvatarFallback>{{ getChampionName(summary.championId || 0).charAt(0) }}</AvatarFallback>
        </Avatar>
        <div>
          <div class="text-xl font-bold">
            {{ getChampionName(summary.championId || 0) }}
          </div>
          <div class="text-sm text-muted-foreground">{{ summary.position || '未知' }} 位置</div>
        </div>
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
        <div class="text-center p-4 bg-muted rounded-lg">
          <div class="text-sm text-muted-foreground mb-1">胜率</div>
          <div class="text-2xl font-bold text-foreground">
            {{ formatPercentage(summary.winRate) }}
          </div>
        </div>
        <div class="text-center p-4 bg-muted rounded-lg">
          <div class="text-sm text-muted-foreground mb-1">选取率</div>
          <div class="text-2xl font-bold text-foreground">
            {{ formatPercentage(summary.pickRate) }}
          </div>
        </div>
        <div class="text-center p-4 bg-muted rounded-lg">
          <div class="text-sm text-muted-foreground mb-1">禁用率</div>
          <div class="text-2xl font-bold text-foreground">
            {{ formatPercentage(summary.banRate) }}
          </div>
        </div>
        <div class="text-center p-4 bg-muted rounded-lg">
          <div class="text-sm text-muted-foreground mb-1">KDA</div>
          <div class="text-2xl font-bold text-foreground">
            {{ formatKda(summary.kda) }}
          </div>
        </div>
      </div>
      <div class="mt-4 text-center">
        <Badge variant="outline">排名: #{{ summary.rank || 'N/A' }}</Badge>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { getChampionIconUrl, getChampionName } from '@/lib'

// 使用后端生成的类型
interface Props {
  summary: OpggChampionSummary
}

defineProps<Props>()

const createFormatter = <T, R = string>(formatter: (value: T) => R, fallback: R = 'N/A' as R) => {
  return (value: T | null | undefined): R => {
    if (value === null || value === undefined) return fallback
    return formatter(value)
  }
}

// 格式化百分比
const formatPercentage = createFormatter((value: number) => `${(value * 100).toFixed(1)}%`, 'N/A')

// 格式化KDA
const formatKda = createFormatter((value: number) => value.toFixed(2), 'N/A')

// 图标错误处理
const onChampionImageError = (event: Event) => {
  const img = event.target as HTMLImageElement
  img.src = '/src/assets/logo.png'
}
</script>
