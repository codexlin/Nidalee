<template>
  <Card class="overflow-hidden">
    <CardHeader class="pb-3">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <Trophy class="h-5 w-5 text-primary" />
          <CardTitle class="text-lg">位置统计</CardTitle>
        </div>
        <Badge variant="outline">{{ positionStats.length }} 个位置</Badge>
      </div>
      <CardDescription>
        主要位置: <span class="font-semibold text-primary">{{ mainPosition }}</span>
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-3">
      <div v-for="pos in positionStats" :key="pos.position" class="space-y-2">
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-2">
            <div
              :class="[
                'h-8 w-8 rounded-md flex items-center justify-center text-xs font-bold',
                pos.position === mainPosition ? 'bg-primary text-primary-foreground' : 'bg-muted text-muted-foreground'
              ]"
            >
              {{ getPositionIcon(pos.position) }}
            </div>
            <div class="flex flex-col">
              <span class="text-sm font-medium">{{ pos.position }}</span>
              <span class="text-xs text-muted-foreground">{{ pos.games }} 场对局</span>
            </div>
          </div>
          <div class="flex items-center gap-4">
            <div class="text-right">
              <div class="text-sm font-semibold">{{ pos.wins }}胜 {{ pos.games - pos.wins }}负</div>
              <div
                :class="[
                  'text-xs font-medium',
                  pos.winRate >= 55
                    ? 'text-green-600 dark:text-green-400'
                    : pos.winRate >= 50
                      ? 'text-blue-600 dark:text-blue-400'
                      : 'text-red-600 dark:text-red-400'
                ]"
              >
                胜率 {{ pos.winRate.toFixed(1) }}%
              </div>
            </div>
            <Button variant="outline" size="sm" @click="() => emit('view-details', pos)">
              <TrendingUp class="h-4 w-4 mr-1" />
              详情
            </Button>
          </div>
        </div>
        <div class="grid grid-cols-3 gap-2 text-xs">
          <div class="bg-muted/50 rounded px-2 py-1">
            <div class="text-muted-foreground">KDA</div>
            <div class="font-semibold">{{ pos.stats.avgKda.toFixed(2) }}</div>
          </div>
          <div class="bg-muted/50 rounded px-2 py-1">
            <div class="text-muted-foreground">K/D/A</div>
            <div class="font-semibold">
              {{ pos.stats.avgKills.toFixed(1) }} / {{ pos.stats.avgDeaths.toFixed(1) }} /
              {{ pos.stats.avgAssists.toFixed(1) }}
            </div>
          </div>
          <div class="bg-muted/50 rounded px-2 py-1">
            <div class="text-muted-foreground">CS/min</div>
            <div class="font-semibold">{{ pos.stats.cspm.toFixed(1) }}</div>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { Trophy, TrendingUp } from 'lucide-vue-next'

interface Props {
  positionStats: PositionStats[]
  mainPosition: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'view-details': [pos: PositionStats]
}>()

const getPositionIcon = (position: string): string => {
  const icons: Record<string, string> = {
    上单: '上',
    打野: '野',
    中单: '中',
    ADC: '下',
    辅助: '辅',
    灵活: '灵'
  }
  return icons[position] || '?'
}
</script>

  return icons[position] || '?'
}
</script>
