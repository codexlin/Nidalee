<template>
  <div class="space-y-4">
    <h4 class="font-semibold flex items-center">
      <UserCheck class="h-4 w-4 mr-2 text-purple-500" />
      召唤师特征
    </h4>

    <!-- 调试：无数据时显示提示 -->
    <div v-if="!matchStatistics" class="text-center py-6 text-sm text-muted-foreground">
      暂无统计数据
    </div>
    <div v-else-if="!matchStatistics.traits || matchStatistics.traits.length === 0" class="text-center py-6 text-sm text-muted-foreground">
      <p>暂无特征数据</p>
      <p class="text-xs mt-1">需要更多对局数据来分析召唤师特征</p>
      <!-- 调试信息 -->
      <p class="text-xs text-muted-foreground/50 mt-2">
        调试: matchStatistics keys = {{ Object.keys(matchStatistics).join(', ') }}
      </p>
    </div>
    <div v-else class="flex flex-wrap gap-1">
      <template v-for="trait in traits" :key="trait.name">
        <button
          class="flex items-center gap-1 px-2 h-8 rounded-full border text-xs font-medium transition-all duration-150 focus:outline-none cursor-pointer select-none min-w-[96px] max-w-[30%] mb-[2px] whitespace-nowrap"
          :class="[
            selectedTrait?.name === trait.name
              ? trait.type === 'bad'
                ? 'border-red-500 bg-red-50 text-red-600 ring-1 ring-red-400'
                : 'border-primary bg-primary/10 text-primary ring-1 ring-primary'
              : trait.type === 'bad'
                ? 'border-red-200 bg-red-50 text-red-500 hover:border-red-400 hover:text-red-600'
                : 'border-border bg-muted text-muted-foreground hover:border-primary/40 hover:text-primary'
          ]"
          @click="selectTrait(trait)"
        >
          <component v-if="trait.icon" :is="trait.icon" class="h-4 w-4" />
          <span v-else class="h-4 w-4 inline-block"></span>
          <span>{{ trait.name }}</span>
          <span
            class="ml-1 flex items-center justify-center w-6 h-6 rounded-full text-[10px] font-bold transition-colors"
            :class="
              selectedTrait?.name === trait.name
                ? trait.type === 'bad'
                  ? 'bg-red-500/95 text-white/95 dark:bg-red-500/85 dark:text-white/85'
                  : 'bg-primary/90 text-primary-foreground/95 dark:bg-primary/70 dark:text-primary-foreground/85'
                : trait.type === 'bad'
                  ? 'bg-red-100 text-red-600 dark:bg-red-950 dark:text-red-300'
                  : 'bg-muted text-muted-foreground/90 dark:bg-muted dark:text-muted-foreground/80'
            "
          >
            {{ trait.score }}
          </span>
        </button>
      </template>
    </div>

    <div v-if="selectedTrait" class="bg-card border rounded-lg p-4 transition-all duration-200 hover:shadow-sm">
      <div class="flex items-start gap-3">
        <div class="p-2 bg-muted rounded-lg">
          <component :is="selectedTrait.icon" class="h-5 w-5" />
        </div>
        <div class="flex-1">
          <div class="flex items-center gap-2 mb-1">
            <h5 class="font-semibold">
              {{ selectedTrait.name }}
            </h5>
            <Badge
              :variant="selectedTrait.name === primaryTrait?.name ? 'default' : 'outline'"
              class="text-xs px-2 py-0.5 font-medium border"
              :class="
                selectedTrait.name === primaryTrait?.name
                  ? 'bg-primary/85 dark:bg-primary/70 text-primary-foreground/95 dark:text-primary-foreground/85 border-primary/30 dark:border-primary/25'
                  : 'bg-primary/10 dark:bg-primary/15 text-primary/90 dark:text-primary/85 border-border dark:border-border/60'
              "
            >
              {{ selectedTrait.name === primaryTrait?.name ? '主要特征' : '特征详情' }}
            </Badge>
          </div>
          <p class="text-sm text-muted-foreground mb-2">
            {{ selectedTrait.description }}
          </p>
          <p class="text-xs text-muted-foreground">
            {{ selectedTrait.detail }}
          </p>
        </div>
      </div>
    </div>

    <div v-if="traits.length > 1" class="text-xs text-muted-foreground text-center">
      共识别出 {{ traits.length }} 个特征，点击标签查看详情
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  AlertTriangle,
  Award,
  Coffee,
  Crown,
  Eye,
  Flame,
  Heart,
  Meh,
  Star,
  Swords,
  TrendingUp,
  UserCheck,
  Zap
} from 'lucide-vue-next'

interface UITrait {
  name: string
  description: string
  detail: string
  score: number
  variant: 'default' | 'secondary' | 'destructive' | 'outline'
  icon: any
  type: 'good' | 'bad'
}

const props = defineProps<{
  matchStatistics: PlayerMatchStats | null
}>()

// 选中的特征
const selectedTrait = ref<UITrait | null>(null)

// 选择特征
const selectTrait = (trait: UITrait) => {
  selectedTrait.value = trait
}

// ✨ 图标映射（根据特征名称）
const iconMap: Record<string, any> = {
  大神: Crown,
  稳定: Award,
  坑货: Meh,
  大爹: Flame,
  输出: Zap,
  送分: AlertTriangle,
  人头狗: Swords,
  辅助王: Heart,
  连胜王: TrendingUp,
  连败: Coffee,
  全能王: Crown,
  视野王: Eye,
  // 默认图标
  _default: Star
}

// ✨ 直接使用后端计算的 traits，只做UI适配
const convertBackendTraits = (): UITrait[] => {
  const backendTraits = props.matchStatistics?.traits || []

  return backendTraits.map((trait) => ({
    name: trait.name,
    description: trait.description,
    detail: `${trait.description}（评分：${trait.score}）`,
    score: trait.score,
    variant: trait.type === 'good' ? 'default' : 'destructive',
    icon: iconMap[trait.name] || iconMap['_default'],
    type: trait.type as 'good' | 'bad'
  }))
}

// ✅ 使用后端计算好的特征
const traits = computed(() => convertBackendTraits())

// 主要特征（得分最高的）
const primaryTrait = computed(() => traits.value[0])

// 初始化选中主要特征
watch(
  traits,
  (newTraits) => {
    if (newTraits.length > 0 && !selectedTrait.value) {
      selectedTrait.value = newTraits[0]
    }
  },
  { immediate: true }
)
</script>
