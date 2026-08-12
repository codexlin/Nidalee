<script setup lang="ts">
import type { AcceptableValue } from 'reka-ui'
import { CheckCircle2, Settings, Sparkles, Zap } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { APP_ROUTES } from '@/router/appRoutes'
import { useBuildPresetStore } from '@/shared/stores/features/buildPresetStore'
import { AUTO_RUNE_OPGG_TIER_OPTIONS, getOpggTierLabel, isOpggTier } from '@/shared/utils/opggTier'

const presetStore = useBuildPresetStore()
const router = useRouter()
const autoBuild = computed(() => presetStore.autoBuild)
const presetCount = computed(() => presetStore.presetCount)
const opggTierLabel = computed(() => getOpggTierLabel(autoBuild.value.opggTier))
const tierOptions = AUTO_RUNE_OPGG_TIER_OPTIONS

onMounted(async () => {
  if (presetStore.isLoaded) return
  try {
    await presetStore.loadFromStore()
  } catch (error) {
    toast.error('构建方案加载失败', {
      description: error instanceof Error ? error.message : String(error)
    })
  }
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

const handleTierChange = (tier: AcceptableValue) => {
  if (isOpggTier(tier)) void savePolicy({ opggTier: tier })
}

const openBuildPresets = () => router.push({ name: APP_ROUTES.buildCenter.name, query: { tab: 'saved' } })
</script>

<template>
  <div class="flex flex-col gap-4">
    <Card class="gap-0 overflow-hidden py-0">
      <CardHeader class="gap-1 border-b border-border/50 px-4 py-4 sm:px-5">
        <CardTitle class="flex items-center gap-2 text-lg font-medium leading-tight">
          <Zap class="size-4 text-muted-foreground" />
          自动应用
        </CardTitle>
        <CardDescription class="text-xs">
          锁定英雄后，优先使用匹配的个人方案；没有个人方案时自动采用在线推荐。
        </CardDescription>
      </CardHeader>

      <CardContent class="flex flex-col gap-4 px-4 py-4 sm:px-5">
        <div class="flex items-center justify-between gap-4 rounded-xl surface-inset px-3 py-3">
          <div class="min-w-0">
            <div class="text-sm font-medium">启用自动构建</div>
            <div class="mt-0.5 text-xs text-muted-foreground">支持单双排、灵活排位、匹配峡谷和极地大乱斗。</div>
          </div>
          <Switch
            :model-value="autoBuild.enabled"
            aria-label="启用自动构建"
            @update:model-value="void savePolicy({ enabled: $event })"
          />
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="flex flex-col gap-1.5">
            <Label class="text-sm font-medium">在线推荐参考段位</Label>
            <Select :model-value="autoBuild.opggTier" @update:model-value="handleTierChange">
              <SelectTrigger class="h-9 w-full text-sm">
                <SelectValue placeholder="选择段位" />
              </SelectTrigger>
              <SelectContent>
                <SelectGroup>
                  <SelectItem v-for="tier in tierOptions" :key="tier.value" :value="tier.value">
                    {{ tier.label }}{{ tier.value === 'diamond_plus' ? '（推荐）' : '' }}
                  </SelectItem>
                </SelectGroup>
              </SelectContent>
            </Select>
            <p class="text-xs text-muted-foreground">仅在没有匹配的个人方案时使用。</p>
          </div>

          <div class="flex items-center justify-between gap-4 rounded-xl surface-inset px-3 py-3">
            <div class="min-w-0">
              <div class="text-sm font-medium">显示应用结果</div>
              <div class="mt-0.5 text-xs text-muted-foreground">符文应用成功或失败后显示通知。</div>
            </div>
            <Switch
              :model-value="autoBuild.showToast"
              aria-label="显示应用结果"
              @update:model-value="void savePolicy({ showToast: $event })"
            />
          </div>
        </div>

        <Alert v-if="autoBuild.enabled">
          <CheckCircle2 />
          <AlertTitle>自动构建已启用</AlertTitle>
          <AlertDescription>
            个人方案优先 · 在线推荐参考 {{ opggTierLabel }}。不支持的游戏模式不会改动客户端符文。
          </AlertDescription>
        </Alert>
      </CardContent>
    </Card>

    <Card class="gap-0 overflow-hidden py-0">
      <CardContent class="flex flex-wrap items-center justify-between gap-4 px-4 py-4 sm:px-5">
        <div class="min-w-0">
          <h2 class="flex items-center gap-2 text-lg font-medium leading-tight">
            <Settings class="size-4 text-muted-foreground" />
            我的构建方案
          </h2>
          <p class="mt-0.5 text-xs text-muted-foreground">高级玩家可为指定英雄和游戏场景保存固定符文，覆盖在线推荐。</p>
        </div>
        <Button variant="outline" @click="openBuildPresets">
          <Sparkles data-icon="inline-start" />
          管理 {{ presetCount }} 个方案
        </Button>
      </CardContent>
    </Card>
  </div>
</template>
