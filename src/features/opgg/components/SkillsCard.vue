<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Target class="w-5 h-5" />
        推荐技能加点
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="p-3 bg-muted rounded-lg">
        <div class="font-medium mb-2">主升优先级: {{ skills.masteries.join(' > ') }}</div>
        <div class="text-sm text-muted-foreground mb-2">
          胜率: {{ formatPercentage(skills.win / skills.play) }} | 选取率: {{ formatPercentage(skills.pickRate) }}
        </div>
        <div class="text-sm font-medium mb-2">加点顺序:</div>
        <div class="flex flex-wrap gap-1">
          <Badge v-for="(skill, levelIndex) in skills.order" :key="levelIndex" variant="outline" class="text-xs">
            {{ levelIndex + 1 }}: {{ skill }}
          </Badge>
        </div>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { Target } from 'lucide-vue-next'

// 使用后端生成的类型
interface Props {
  skills: OpggSkills
}

defineProps<Props>()

// 格式化百分比
const formatPercentage = (value: number): string => {
  return (value * 100).toFixed(1) + '%'
}
</script>
