<template>
  <div class="space-y-4">
    <Card class="gap-0 overflow-hidden py-0">
      <div class="space-y-0.5 border-b border-border/50 px-4 py-4 sm:px-5">
        <h2 class="flex items-center gap-2 text-lg font-medium leading-tight">
          <Zap class="size-4 text-muted-foreground" />
          自动应用
        </h2>
        <p class="text-xs text-muted-foreground">选人阶段按策略自动应用“我的方案”或推荐符文。</p>
      </div>

      <div class="space-y-4 px-4 py-4 sm:px-5">
        <div class="flex items-center justify-between gap-4 rounded-xl surface-inset px-3 py-3">
          <div class="min-w-0 space-y-0.5">
            <div class="text-sm font-medium">启用自动构建</div>
            <div class="text-xs text-muted-foreground">英雄锁定后自动解析并应用一套构建方案。</div>
          </div>
          <Switch :model-value="autoBuild.enabled" @update:model-value="handleAutoApplyChange" />
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-1.5">
            <Label class="text-sm font-medium">优先级策略</Label>
            <Select :model-value="autoBuild.strategy" @update:model-value="handleStrategyChange">
              <SelectTrigger class="h-9 w-full text-sm">
                <SelectValue placeholder="选择策略" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="smart">
                  <div class="flex items-center gap-2">
                    <Sparkles class="size-3.5" />
                    <div>
                      <div class="text-sm font-medium">智能模式（推荐）</div>
                      <div class="text-xs text-muted-foreground">优先我的方案，未命中时使用推荐</div>
                    </div>
                  </div>
                </SelectItem>
                <SelectItem value="recommended-only">
                  <div class="flex items-center gap-2">
                    <Globe class="size-3.5" />
                    <div>
                      <div class="text-sm font-medium">仅推荐方案</div>
                      <div class="text-xs text-muted-foreground">总是使用当前 OP.GG 推荐</div>
                    </div>
                  </div>
                </SelectItem>
                <SelectItem value="saved-only">
                  <div class="flex items-center gap-2">
                    <User class="size-3.5" />
                    <div>
                      <div class="text-sm font-medium">仅我的方案</div>
                      <div class="text-xs text-muted-foreground">未命中已保存方案时不自动应用</div>
                    </div>
                  </div>
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="space-y-1.5">
            <Label class="text-sm font-medium">OP.GG 段位参考</Label>
            <Select :model-value="autoBuild.opggTier" @update:model-value="handleTierChange">
              <SelectTrigger class="h-9 w-full text-sm">
                <SelectValue placeholder="选择段位" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem v-for="tier in tierOptions" :key="tier.value" :value="tier.value">
                  {{ tier.label }}{{ tier.value === 'diamond_plus' ? '（推荐）' : '' }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <div class="flex items-center justify-between gap-4 rounded-xl surface-inset px-3 py-3">
          <div class="min-w-0 space-y-0.5">
            <div class="text-sm font-medium">显示应用成功提示</div>
            <div class="text-xs text-muted-foreground">应用符文后显示 Toast 通知</div>
          </div>
          <Switch :model-value="autoBuild.showToast" @update:model-value="handleShowToastChange" />
        </div>

        <div
          v-if="autoBuild.enabled"
          class="flex items-start gap-2 rounded-xl border border-primary/20 bg-primary/5 px-3 py-2.5"
        >
          <CheckCircle2 class="mt-0.5 size-4 shrink-0 text-primary" />
          <div class="min-w-0 space-y-0.5">
            <div class="text-sm font-medium text-primary">自动构建已启用</div>
            <div class="text-xs text-muted-foreground">
              当前策略：{{ strategyLabel }} · OP.GG 段位：{{ opggTierLabel }}
            </div>
          </div>
        </div>
      </div>
    </Card>

    <Card class="gap-0 overflow-hidden py-0">
      <div class="flex flex-wrap items-center justify-between gap-4 px-4 py-4 sm:px-5">
        <div class="min-w-0 space-y-0.5">
          <h2 class="flex items-center gap-2 text-lg font-medium leading-tight">
            <Settings class="size-4 text-muted-foreground" />
            我的构建方案
          </h2>
          <p class="text-xs text-muted-foreground">方案创建、编辑、导入和应用统一在构建中心管理。</p>
        </div>
        <Button variant="outline" @click="openBuildPresets">管理 {{ presetCount }} 个方案</Button>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { useBuildPresetStore } from '@/shared/stores/features/buildPresetStore'
import { Zap, Sparkles, Globe, User, Settings, CheckCircle2 } from 'lucide-vue-next'
import type { AcceptableValue } from 'reka-ui'
import { toast } from 'vue-sonner'
import { AUTO_RUNE_OPGG_TIER_OPTIONS, getOpggTierLabel, isOpggTier } from '@/shared/utils/opggTier'

onMounted(async () => {
  if (!useBuildPresetStore().isLoaded) {
    try {
      await useBuildPresetStore().loadFromStore()
    } catch (error) {
      toast.error('构建方案加载失败', {
        description: error instanceof Error ? error.message : String(error)
      })
    }
  }
})

const presetStore = useBuildPresetStore()
const router = useRouter()
const autoBuild = computed(() => presetStore.autoBuild)
const presetCount = computed(() => presetStore.presetCount)
const tierOptions = AUTO_RUNE_OPGG_TIER_OPTIONS
const opggTierLabel = computed(() => getOpggTierLabel(autoBuild.value.opggTier))

const strategyLabel = computed(() => {
  const labels = {
    smart: '智能模式',
    'recommended-only': '仅推荐方案',
    'saved-only': '仅我的方案'
  }
  return labels[autoBuild.value.strategy]
})

const savePolicy = async (updates: Parameters<typeof presetStore.updateAutoBuild>[0]) => {
  try {
    await presetStore.updateAutoBuild(updates)
  } catch (error) {
    toast.error('自动构建设置保存失败', {
      description: error instanceof Error ? error.message : String(error)
    })
  }
}

const handleAutoApplyChange = (enabled: boolean) => {
  void savePolicy({ enabled })
}

const handleStrategyChange = (strategy: AcceptableValue) => {
  if (typeof strategy !== 'string') return
  if (!['smart', 'recommended-only', 'saved-only'].includes(strategy)) return
  void savePolicy({ strategy: strategy as 'smart' | 'recommended-only' | 'saved-only' })
}

const handleTierChange = (tier: AcceptableValue) => {
  if (!isOpggTier(tier)) return
  void savePolicy({ opggTier: tier })
}

const handleShowToastChange = (showToast: boolean) => {
  void savePolicy({ showToast })
}

const openBuildPresets = () => router.push({ path: '/opgg', query: { tab: 'saved' } })
</script>
