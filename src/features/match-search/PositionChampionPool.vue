<template>
  <Card>
    <CardHeader>
      <CardTitle class="text-base">英雄池分析</CardTitle>
      <CardDescription>该位置常用英雄 · Top 5</CardDescription>
    </CardHeader>
    <CardContent>
      <div v-if="championStats.length > 0" class="space-y-3">
        <div
          v-for="(champ, idx) in championStats"
          :key="champ.championId"
          class="flex items-center gap-3 p-3 rounded-lg bg-muted/30 hover:bg-muted/50 transition"
        >
          <div class="flex items-center justify-center w-8 h-8 rounded bg-primary/10 text-primary font-bold text-sm">
            {{ idx + 1 }}
          </div>
          <div class="flex-1">
            <div class="flex items-center justify-between mb-1">
              <span class="font-semibold">{{ champ.championName || `英雄 ${champ.championId}` }}</span>
              <span class="text-sm text-muted-foreground">{{ champ.games }} 场</span>
            </div>
            <div class="flex items-center gap-4 text-xs">
              <span :class="champ.winRate >= 50 ? 'text-green-600' : 'text-red-600'">
                胜率 {{ champ.winRate.toFixed(1) }}%
              </span>
              <span class="text-muted-foreground">{{ champ.wins }}胜 {{ champ.games - champ.wins }}负</span>
            </div>
          </div>
          <div class="w-16 h-2 bg-muted rounded-full overflow-hidden">
            <div
              class="h-full bg-primary transition-all"
              :style="{ width: `${(champ.games / totalGames) * 100}%` }"
            ></div>
          </div>
        </div>
      </div>
      <div v-else class="text-center py-8 text-muted-foreground text-sm">
        <p>暂无英雄数据</p>
        <p class="text-xs mt-1">需要更多对局数据才能分析英雄池</p>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
interface ChampionStat {
  championId: number
  championName?: string
  games: number
  wins: number
  winRate: number
}

interface Props {
  positionData: PositionStats
}

const props = defineProps<Props>()

// TODO: 从后端获取英雄数据，目前使用模拟数据
const championStats = computed<ChampionStat[]>(() => {
  // 这里需要后端支持，返回该位置的英雄统计
  // 暂时返回空数组
  return []
})

const totalGames = computed(() => {
  return championStats.value.reduce((sum, champ) => sum + champ.games, 0)
})
</script>
