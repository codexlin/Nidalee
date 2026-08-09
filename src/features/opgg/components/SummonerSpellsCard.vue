<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Shield class="w-5 h-5" />
        推荐召唤师技能
      </CardTitle>
    </CardHeader>
    <CardContent>
      <div class="space-y-3">
        <div
          v-for="(spell, index) in spells.slice(0, 3)"
          :key="index"
          class="flex items-center justify-between p-3 bg-muted rounded-lg"
        >
          <div class="flex items-center gap-3 flex-1">
            <!-- 召唤师技能图标 -->
            <div class="flex gap-1">
              <img
                v-for="spellId in spell.ids"
                :key="spellId"
                :src="getSpellMeta(spellId).icon"
                :alt="getSpellMeta(spellId).label"
                class="w-8 h-8 rounded border border-yellow-500"
                @error="onSpellImageError"
              />
            </div>
            <div class="flex-1">
              <div class="font-medium">
                {{ spell.ids.map((id: number) => getSpellMeta(id).label).join(' + ') }}
              </div>
              <div class="text-sm text-muted-foreground">
                胜率: {{ formatPercentage(spell.win / spell.play) }} | 选取率: {{ formatPercentage(spell.pickRate) }}
              </div>
              <div class="text-xs text-muted-foreground">{{ spell.play }} 局游戏</div>
            </div>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { Shield } from 'lucide-vue-next'
import { getSpellMeta } from '@/lib'

// 使用后端生成的类型
interface Props {
  spells: OpggSummonerSpell[]
}

defineProps<Props>()

// 保证技能目录已加载（App 初始化也会拉；此处兜底）
useSummonerSpells()

// 格式化百分比
const formatPercentage = (value: number): string => {
  return (value * 100).toFixed(1) + '%'
}

const onSpellImageError = (event: Event) => {
  const img = event.target as HTMLImageElement
  img.style.visibility = 'hidden'
}
</script>
