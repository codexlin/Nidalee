<template>
  <div
    class="border rounded-lg p-4 transition-all duration-200 hover:shadow-md"
    :class="[priorityBorderClass, 'bg-card']"
  >
    <!-- 标题栏 -->
    <div class="flex items-start justify-between gap-3 mb-3">
      <div class="flex items-start gap-3 flex-1">
        <!-- 分类图标 -->
        <div class="p-2 rounded-lg flex-shrink-0" :class="categoryBgClass">
          <component :is="categoryIcon" class="h-5 w-5" :class="categoryIconClass" />
        </div>

        <!-- 标题和问题 -->
        <div class="flex-1 min-w-0">
          <h4 class="font-semibold text-base mb-1 flex items-center gap-2">
            <span>{{ advice.title }}</span>
          </h4>
          <p class="text-sm text-muted-foreground mb-2">
            {{ advice.problem }}
          </p>

          <!-- 数据证据 -->
          <div class="flex items-center gap-2 text-xs text-muted-foreground bg-muted/50 rounded px-2 py-1 inline-flex">
            <BarChart3 class="h-3 w-3" />
            <span>{{ advice.evidence }}</span>
          </div>
        </div>
      </div>

      <!-- 优先级标识 -->
      <Badge :variant="priorityVariant" class="text-xs font-semibold flex-shrink-0">
        {{ priorityLabel }}
      </Badge>
    </div>

    <!-- 建议列表 -->
    <div class="space-y-2 mt-4">
      <div class="flex items-center gap-2 text-sm font-medium text-foreground mb-2">
        <ListChecks class="h-4 w-4 text-primary" />
        <span>具体建议：</span>
      </div>

      <ul class="space-y-2">
        <li
          v-for="(suggestion, index) in advice.suggestions"
          :key="index"
          class="flex items-start gap-2 text-sm text-foreground bg-muted/30 rounded-md px-3 py-2 hover:bg-muted/50 transition-colors"
        >
          <CheckCircle2 class="h-4 w-4 text-green-600 dark:text-green-400 flex-shrink-0 mt-0.5" />
          <span class="flex-1">{{ suggestion }}</span>
        </li>
      </ul>
    </div>

    <!-- 底部信息 -->
    <div
      v-if="advice.affectedRole || advice.targetPlayer"
      class="mt-3 pt-3 border-t flex items-center gap-4 text-xs text-muted-foreground"
    >
      <div v-if="advice.affectedRole" class="flex items-center gap-1">
        <Target class="h-3 w-3" />
        <span>影响位置：{{ advice.affectedRole }}</span>
      </div>
      <div v-if="advice.targetPlayer" class="flex items-center gap-1">
        <User class="h-3 w-3" />
        <span>目标：{{ advice.targetPlayer }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import {
  BarChart3,
  CheckCircle2,
  ListChecks,
  Target,
  User,
  Swords,
  Sprout,
  Users,
  Eye,
  MapPin,
  Brain,
  Gamepad2
} from 'lucide-vue-next'
import { Badge } from '@/components/ui/badge'
import type { GameAdvice } from '@/types/generated/GameAdvice'

interface Props {
  advice: GameAdvice
  perspective?: 'self-improvement' | 'targeting' | 'collaboration'
}

const props = withDefaults(defineProps<Props>(), {
  perspective: 'self-improvement'
})

// 优先级样式和标签
const priorityVariant = computed(() => {
  if (props.advice.priority >= 4) return 'destructive' // 高优先级：红色
  if (props.advice.priority >= 2) return 'default' // 中优先级：默认
  return 'secondary' // 低优先级：灰色
})

const priorityLabel = computed(() => {
  if (props.advice.priority >= 4) return '高优先级'
  if (props.advice.priority >= 2) return '中优先级'
  return '低优先级'
})

const priorityBorderClass = computed(() => {
  if (props.advice.priority >= 4) {
    return 'border-l-4 border-l-red-500 dark:border-l-red-400'
  }
  if (props.advice.priority >= 2) {
    return 'border-l-4 border-l-blue-500 dark:border-l-blue-400'
  }
  return 'border-l-4 border-l-gray-400 dark:border-l-gray-600'
})

// 分类图标和样式
const categoryIcon = computed(() => {
  const icons = {
    Laning: Swords,
    Farming: Sprout,
    Teamfight: Users,
    Vision: Eye,
    Positioning: MapPin,
    Decision: Brain,
    Champion: Gamepad2
  }
  return icons[props.advice.category] || Swords
})

const categoryBgClass = computed(() => {
  const classes = {
    Laning: 'bg-red-100 dark:bg-red-950',
    Farming: 'bg-green-100 dark:bg-green-950',
    Teamfight: 'bg-blue-100 dark:bg-blue-950',
    Vision: 'bg-purple-100 dark:bg-purple-950',
    Positioning: 'bg-yellow-100 dark:bg-yellow-950',
    Decision: 'bg-indigo-100 dark:bg-indigo-950',
    Champion: 'bg-pink-100 dark:bg-pink-950'
  }
  return classes[props.advice.category] || 'bg-muted'
})

const categoryIconClass = computed(() => {
  const classes = {
    Laning: 'text-red-600 dark:text-red-400',
    Farming: 'text-green-600 dark:text-green-400',
    Teamfight: 'text-blue-600 dark:text-blue-400',
    Vision: 'text-purple-600 dark:text-purple-400',
    Positioning: 'text-yellow-600 dark:text-yellow-400',
    Decision: 'text-indigo-600 dark:text-indigo-400',
    Champion: 'text-pink-600 dark:text-pink-400'
  }
  return classes[props.advice.category] || 'text-muted-foreground'
})
</script>
