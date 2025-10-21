<template>
  <div class="space-y-6">
    <!-- 简单模式 -->
    <Card
      class="p-8 rounded-2xl shadow-xl bg-gradient-to-br from-white/80 to-muted/60 dark:from-background/80 dark:to-muted/40 border border-border"
    >
      <div class="space-y-6">
        <div>
          <h2 class="text-xl font-bold text-primary flex items-center gap-2">
            <Zap class="h-5 w-5" />
            简单模式（自动应用）
          </h2>
          <p class="text-sm text-muted-foreground">选人阶段自动应用符文，支持 OP.GG 推荐和用户自定义配置</p>
        </div>

        <div class="border-t border-dashed border-border pt-6 space-y-4">
          <!-- 启用开关 -->
          <div class="flex items-center justify-between gap-4">
            <div class="space-y-1">
              <div class="text-sm font-medium text-foreground">启用自动符文配置</div>
              <div class="text-xs text-muted-foreground">选择英雄后自动应用最佳符文配置</div>
            </div>
            <Switch v-model="autoApply.enabled" @update:model-value="handleAutoApplyChange" />
          </div>

          <!-- 优先级策略 -->
          <div class="space-y-2">
            <div class="text-sm font-medium text-foreground">优先级策略</div>
            <Select v-model:model-value="autoApply.strategy" @update:model-value="handleStrategyChange">
              <SelectTrigger class="w-full">
                <SelectValue placeholder="选择策略" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">
                  <div class="flex items-center gap-2">
                    <Sparkles class="h-4 w-4" />
                    <div>
                      <div class="font-medium">智能模式（推荐）</div>
                      <div class="text-xs text-muted-foreground">优先自定义，回退 OP.GG</div>
                    </div>
                  </div>
                </SelectItem>
                <SelectItem value="opgg">
                  <div class="flex items-center gap-2">
                    <Globe class="h-4 w-4" />
                    <div>
                      <div class="font-medium">仅 OP.GG</div>
                      <div class="text-xs text-muted-foreground">总是使用 OP.GG 推荐</div>
                    </div>
                  </div>
                </SelectItem>
                <SelectItem value="custom">
                  <div class="flex items-center gap-2">
                    <User class="h-4 w-4" />
                    <div>
                      <div class="font-medium">仅自定义</div>
                      <div class="text-xs text-muted-foreground">只使用用户自定义配置</div>
                    </div>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- OP.GG 段位参考 -->
          <div class="space-y-2">
            <div class="text-sm font-medium text-foreground">OP.GG 段位参考</div>
            <Select v-model:model-value="autoApply.opggTier" @update:model-value="handleTierChange">
              <SelectTrigger class="w-full">
                <SelectValue placeholder="选择段位" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="ALL">全部段位</SelectItem>
                <SelectItem value="PLATINUM+">铂金以上</SelectItem>
                <SelectItem value="DIAMOND+">钻石以上（推荐）</SelectItem>
                <SelectItem value="MASTER+">大师以上</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- 显示提示 -->
          <div class="flex items-center justify-between gap-4">
            <div class="space-y-1">
              <div class="text-sm font-medium text-foreground">显示应用成功提示</div>
              <div class="text-xs text-muted-foreground">应用符文后显示 Toast 通知</div>
            </div>
            <Switch v-model="autoApply.showToast" @update:model-value="handleShowToastChange" />
          </div>

          <!-- 状态显示 -->
          <div v-if="autoApply.enabled" class="mt-4 p-4 rounded-lg bg-primary/10 border border-primary/20">
            <div class="flex items-start gap-3">
              <CheckCircle2 class="h-5 w-5 text-primary mt-0.5" />
              <div class="flex-1">
                <div class="text-sm font-medium text-primary">自动符文配置已启用</div>
                <div class="text-xs text-muted-foreground mt-1">
                  当前策略：{{ strategyLabel }} | OP.GG 段位：{{ autoApply.opggTier }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Card>

    <!-- 复杂模式 -->
    <Card
      class="p-8 rounded-2xl shadow-xl bg-gradient-to-br from-white/80 to-muted/60 dark:from-background/80 dark:to-muted/40 border border-border"
    >
      <div class="space-y-6">
        <div class="flex items-center justify-between">
          <div>
            <h2 class="text-xl font-bold text-primary flex items-center gap-2">
              <Settings class="h-5 w-5" />
              复杂模式（自定义符文配置）
            </h2>
            <p class="text-sm text-muted-foreground">为每个英雄和位置创建专属符文配置</p>
          </div>
          <div class="flex items-center gap-2">
            <Badge variant="outline" class="text-xs"> {{ configCount }} 个配置 </Badge>
          </div>
        </div>

        <div class="border-t border-dashed border-border pt-6">
          <RuneConfigList />
        </div>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { useUserRuneStore } from '@/shared/stores/features/userRuneStore'
import { Zap, Sparkles, Globe, User, Settings, CheckCircle2 } from 'lucide-vue-next'
import RuneConfigList from './RuneConfigList.vue'

// 确保 userRuneStore 在组件加载时初始化
onMounted(async () => {
  if (!useUserRuneStore().isLoaded) {
    await useUserRuneStore().loadFromStore()
  }
})

const userRuneStore = useUserRuneStore()

// 响应式状态
const autoApply = computed(() => userRuneStore.autoApply)
const configCount = computed(() => userRuneStore.configCount)

// 策略标签映射
const strategyLabel = computed(() => {
  const labels = {
    auto: '智能模式',
    opgg: '仅 OP.GG',
    custom: '仅自定义'
  }
  return labels[autoApply.value.strategy]
})

// 事件处理
const handleAutoApplyChange = async (enabled: boolean) => {
  await userRuneStore.updateAutoApply({ enabled })
}

const handleStrategyChange = async (strategy: string) => {
  await userRuneStore.updateAutoApply({ strategy: strategy as 'auto' | 'opgg' | 'custom' })
}

const handleTierChange = async (tier: string) => {
  await userRuneStore.updateAutoApply({ opggTier: tier })
}

const handleShowToastChange = async (showToast: boolean) => {
  await userRuneStore.updateAutoApply({ showToast })
}
</script>
