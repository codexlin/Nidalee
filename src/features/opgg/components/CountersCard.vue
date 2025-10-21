<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Users class="w-5 h-5" />
        克制关系
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="grid md:grid-cols-2 gap-6">
        <!-- 优势对局 -->
        <div>
          <h4 class="font-medium mb-3 flex items-center gap-2 text-green-600 dark:text-green-400">
            <TrendingUp class="w-4 h-4" />
            优势对局
          </h4>
          <div v-if="counters.strongAgainst?.length" class="space-y-2">
            <div
              v-for="(champion, index) in counters.strongAgainst.slice(0, 5)"
              :key="index"
              class="flex items-center justify-between p-3 bg-muted rounded-lg"
            >
              <div class="flex items-center gap-3">
                <Avatar class="w-8 h-8">
                  <AvatarImage
                    :src="getChampionIconUrl(champion.championId)"
                    :alt="getChampionName(champion.championId)"
                    @error="onChampionImageError"
                  />
                  <AvatarFallback>{{ getChampionName(champion.championId).charAt(0) }}</AvatarFallback>
                </Avatar>
                <span class="font-medium">{{ getChampionName(champion.championId) }}</span>
              </div>
              <Badge variant="default" class="text-green-600 dark:text-green-400">
                {{ formatPercentage(champion.winRate) }}
              </Badge>
            </div>
          </div>
        </div>

        <!-- 劣势对局 -->
        <div>
          <h4 class="font-medium mb-3 flex items-center gap-2 text-red-600 dark:text-red-400">
            <TrendingDown class="w-4 h-4" />
            劣势对局
          </h4>
          <div v-if="counters.weakAgainst?.length" class="space-y-2">
            <div
              v-for="(champion, index) in counters.weakAgainst.slice(0, 5)"
              :key="index"
              class="flex items-center justify-between p-3 bg-muted rounded-lg"
            >
              <div class="flex items-center gap-3">
                <Avatar class="w-8 h-8">
                  <AvatarImage
                    :src="getChampionIconUrl(champion.championId)"
                    :alt="getChampionName(champion.championId)"
                    @error="onChampionImageError"
                  />
                  <AvatarFallback>{{ getChampionName(champion.championId).charAt(0) }}</AvatarFallback>
                </Avatar>
                <span class="font-medium">{{ getChampionName(champion.championId) }}</span>
              </div>
              <Badge variant="destructive">
                {{ formatPercentage(champion.winRate) }}
              </Badge>
            </div>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { Users, TrendingUp, TrendingDown } from 'lucide-vue-next'
import { getChampionIconUrl, getChampionName } from '@/lib'

// 使用后端生成的类型
interface Props {
  counters: OpggCounters
}

defineProps<Props>()

// 格式化百分比
const formatPercentage = (value: number): string => {
  return (value * 100).toFixed(1) + '%'
}

// 英雄图标错误处理
const onChampionImageError = (event: Event) => {
  const img = event.target as HTMLImageElement
  img.src = '/src/assets/logo.png'
}
</script>
