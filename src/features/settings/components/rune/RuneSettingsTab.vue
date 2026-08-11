<template>
  <div class="space-y-4">
    <Card class="gap-0 overflow-hidden py-0">
      <div class="space-y-0.5 border-b border-border/50 px-4 py-4 sm:px-5">
        <h2 class="flex items-center gap-2 text-lg font-medium leading-tight">
          <Zap class="size-4 text-muted-foreground" />
          简单模式
        </h2>
        <p class="text-xs text-muted-foreground">选人阶段自动应用符文，支持 OP.GG 与自定义配置</p>
      </div>

      <div class="space-y-4 px-4 py-4 sm:px-5">
        <div class="flex items-center justify-between gap-4 rounded-xl surface-inset px-3 py-3">
          <div class="min-w-0 space-y-0.5">
            <div class="text-sm font-medium">启用自动符文配置</div>
            <div class="text-xs text-muted-foreground">选择英雄后自动应用最佳符文</div>
          </div>
          <Switch v-model="autoApply.enabled" @update:model-value="handleAutoApplyChange" />
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-1.5">
            <Label class="text-sm font-medium">优先级策略</Label>
            <Select v-model:model-value="autoApply.strategy" @update:model-value="handleStrategyChange">
              <SelectTrigger class="h-9 w-full text-sm">
                <SelectValue placeholder="选择策略" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="auto">
                  <div class="flex items-center gap-2">
                    <Sparkles class="size-3.5" />
                    <div>
                      <div class="text-sm font-medium">智能模式（推荐）</div>
                      <div class="text-xs text-muted-foreground">优先自定义，回退 OP.GG</div>
                    </div>
                  </div>
                </SelectItem>
                <SelectItem value="opgg">
                  <div class="flex items-center gap-2">
                    <Globe class="size-3.5" />
                    <div>
                      <div class="text-sm font-medium">仅 OP.GG</div>
                      <div class="text-xs text-muted-foreground">总是使用 OP.GG 推荐</div>
                    </div>
                  </div>
                </SelectItem>
                <SelectItem value="custom">
                  <div class="flex items-center gap-2">
                    <User class="size-3.5" />
                    <div>
                      <div class="text-sm font-medium">仅自定义</div>
                      <div class="text-xs text-muted-foreground">只使用用户自定义配置</div>
                    </div>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-1.5">
            <Label class="text-sm font-medium">OP.GG 段位参考</Label>
            <Select v-model:model-value="autoApply.opggTier" @update:model-value="handleTierChange">
              <SelectTrigger class="h-9 w-full text-sm">
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
        </div>

        <div class="flex items-center justify-between gap-4 rounded-xl surface-inset px-3 py-3">
          <div class="min-w-0 space-y-0.5">
            <div class="text-sm font-medium">显示应用成功提示</div>
            <div class="text-xs text-muted-foreground">应用符文后显示 Toast 通知</div>
          </div>
          <Switch v-model="autoApply.showToast" @update:model-value="handleShowToastChange" />
        </div>

        <div
          v-if="autoApply.enabled"
          class="flex items-start gap-2 rounded-xl border border-primary/20 bg-primary/5 px-3 py-2.5"
        >
          <CheckCircle2 class="mt-0.5 size-4 shrink-0 text-primary" />
          <div class="min-w-0 space-y-0.5">
            <div class="text-sm font-medium text-primary">自动符文配置已启用</div>
            <div class="text-xs text-muted-foreground">
              当前策略：{{ strategyLabel }} · OP.GG 段位：{{ autoApply.opggTier }}
            </div>
          </div>
        </div>
      </div>
    </Card>

    <Card class="gap-0 overflow-hidden py-0">
      <div class="flex flex-wrap items-start justify-between gap-3 border-b border-border/50 px-4 py-4 sm:px-5">
        <div class="min-w-0 space-y-0.5">
          <h2 class="flex items-center gap-2 text-lg font-medium leading-tight">
            <Settings class="size-4 text-muted-foreground" />
            复杂模式
          </h2>
          <p class="text-xs text-muted-foreground">为每个英雄和位置创建专属符文配置</p>
        </div>
        <span class="rounded-full surface-inset px-2.5 py-1 text-xs tabular-nums text-muted-foreground">
          {{ configCount }} 个配置
        </span>
      </div>
      <div class="px-4 py-4 sm:px-5">
        <RuneConfigList />
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { useUserRuneStore } from '@/shared/stores/features/userRuneStore'
import { Zap, Sparkles, Globe, User, Settings, CheckCircle2 } from 'lucide-vue-next'
import type { AcceptableValue } from 'reka-ui'
import RuneConfigList from './RuneConfigList.vue'

onMounted(async () => {
  if (!useUserRuneStore().isLoaded) {
    await useUserRuneStore().loadFromStore()
  }
})

const userRuneStore = useUserRuneStore()
const autoApply = computed(() => userRuneStore.autoApply)
const configCount = computed(() => userRuneStore.configCount)

const strategyLabel = computed(() => {
  const labels = {
    auto: '智能模式',
    opgg: '仅 OP.GG',
    custom: '仅自定义'
  }
  return labels[autoApply.value.strategy]
})

const handleAutoApplyChange = async (enabled: boolean) => {
  await userRuneStore.updateAutoApply({ enabled })
}

const handleStrategyChange = async (strategy: AcceptableValue) => {
  if (typeof strategy !== 'string') return
  await userRuneStore.updateAutoApply({ strategy: strategy as 'auto' | 'opgg' | 'custom' })
}

const handleTierChange = async (tier: AcceptableValue) => {
  if (typeof tier !== 'string') return
  await userRuneStore.updateAutoApply({ opggTier: tier })
}

const handleShowToastChange = async (showToast: boolean) => {
  await userRuneStore.updateAutoApply({ showToast })
}
</script>
