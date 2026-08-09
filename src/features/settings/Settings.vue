<template>
  <div class="min-h-screen bg-gradient-to-br from-background via-muted/60 to-background/80 py-6 px-4">
    <div class="max-w-6xl mx-auto space-y-8">
      <div class="flex items-center gap-4 mb-4">
        <span class="inline-flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary shadow">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
            <path
              d="M12 3v2m0 14v2m9-9h-2M5 12H3m15.364-6.364l-1.414 1.414M6.05 17.95l-1.414 1.414m12.728 0l-1.414-1.414M6.05 6.05L4.636 4.636"
              stroke-linecap="round"
              stroke-linejoin="round"
            />
          </svg>
        </span>
        <div>
          <h1 class="text-3xl font-extrabold text-foreground tracking-tight">客户端设置</h1>
          <p class="text-muted-foreground text-base mt-1">自定义您的应用程序外观、行为和自动化功能</p>
        </div>
      </div>

      <Tabs v-model="activeTab" class="w-full">
        <TabsList class="w-full flex gap-2 bg-transparent p-0 border-0 shadow-none">
          <TabsTrigger
            v-for="tab in tabs"
            :key="tab.id"
            :value="tab.id"
            class="flex-1 min-w-fit flex items-center justify-center gap-2 rounded-lg py-2 px-4 text-base font-medium transition-all duration-200 border border-transparent hover:border-primary/40 hover:bg-primary/10 hover:text-primary focus-visible:ring-2 focus-visible:ring-primary/40 data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=active]:shadow-md data-[state=active]:border-primary"
          >
            <component :is="tab.icon" class="h-4 w-4" />
            {{ tab.label }}
          </TabsTrigger>
        </TabsList>

        <!-- 外观设置 -->
        <TabsContent value="appearance" class="space-y-8">
          <Card
            class="p-8 rounded-2xl shadow-xl bg-gradient-to-br from-white/80 to-muted/60 dark:from-background/80 dark:to-muted/40 border border-border"
          >
            <div class="space-y-6">
              <div>
                <h2 class="text-xl font-bold text-primary">主题定制</h2>
                <p class="text-sm text-muted-foreground">自定义应用程序的外观主题</p>
              </div>
              <div class="border-t border-dashed border-border pt-6">
                <ThemeCustomizer />
              </div>
              <div class="border-t border-dashed border-border pt-6 space-y-1">
                <h3 class="text-sm font-medium text-foreground">字体声明</h3>
                <p class="text-xs text-muted-foreground leading-relaxed">
                  界面字体使用 HarmonyOS Sans Fonts。Copyright 2021 Huawei Device Co., Ltd.
                </p>
              </div>
            </div>
          </Card>
        </TabsContent>

        <!-- 游戏设置 -->
        <TabsContent value="game" class="space-y-8">
          <Card
            class="p-8 rounded-2xl shadow-xl bg-gradient-to-br from-white/80 to-muted/60 dark:from-background/80 dark:to-muted/40 border border-border"
          >
            <div class="space-y-6">
              <div>
                <h2 class="text-xl font-bold text-primary">战绩搜索</h2>
                <p class="text-sm text-muted-foreground">
                  模式与场数在仪表盘选择并自动记住；此处仅控制搜索页是否跟随后台模式过滤
                </p>
              </div>
              <div class="border-t border-dashed border-border pt-6 space-y-4">
                <div class="flex items-center justify-between gap-4">
                  <div class="space-y-1">
                    <div class="text-sm font-medium text-foreground">查询后自动应用模式过滤</div>
                    <div class="text-xs text-muted-foreground">开启后按仪表盘上次模式过滤搜索结果</div>
                  </div>
                  <Switch
                    :model-value="applyDefaultFilterOnSearch"
                    @update:model-value="(v: boolean) => (applyDefaultFilterOnSearch = v)"
                  />
                </div>
              </div>
            </div>
          </Card>
        </TabsContent>

        <!-- 自动化功能 -->
        <TabsContent value="automation" class="space-y-8">
          <Card
            class="p-8 rounded-2xl shadow-xl bg-gradient-to-br from-white/80 to-muted/60 dark:from-background/80 dark:to-muted/40 border border-border"
          >
            <div class="space-y-6">
              <div class="flex items-center justify-between">
                <div>
                  <h2 class="text-xl font-bold text-primary">自动化功能配置</h2>
                  <p class="text-sm text-muted-foreground">配置各种自动化游戏功能和执行延迟</p>
                </div>
                <div class="flex items-center gap-4">
                  <Badge
                    variant="secondary"
                    class="px-4 py-2 text-sm font-medium bg-muted/50 hover:bg-muted transition-colors"
                  >
                    已启用: {{ enabledFunctionsCount }} 项
                  </Badge>
                  <Button
                    variant="destructive"
                    size="default"
                    @click="handleDisableAll"
                    :disabled="!isAnyFunctionEnabled"
                    class="gap-2 transition-all hover:bg-destructive/90 disabled:opacity-50"
                  >
                    <X class="h-4 w-4" />
                    全部禁用
                  </Button>
                </div>
              </div>

              <div class="border-t border-dashed border-border pt-6">
                <div class="space-y-6">
                  <!-- 功能状态摘要 -->
                  <div v-if="isAnyFunctionEnabled" class="p-6 bg-muted/30 rounded-lg border border-border">
                    <h3 class="text-lg font-semibold text-foreground mb-4">已启用功能</h3>
                    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                      <div
                        v-for="func in enabledFunctions"
                        :key="func.key"
                        class="flex items-center gap-3 p-3 bg-card rounded-lg border border-border"
                      >
                        <div class="h-2 w-2 bg-green-500 rounded-full animate-pulse"></div>
                        <span class="text-sm font-medium text-foreground">{{ func.name }}</span>
                      </div>
                    </div>
                  </div>

                  <!-- 功能卡片 -->
                  <div class="grid grid-cols-1 xl:grid-cols-2 gap-6">
                    <!-- 自动接受对局 -->
                    <div class="card-hover-effect rounded-xl bg-card">
                      <FunctionCard
                        title="自动接受对局"
                        description="自动接受匹配到的对局，避免错过游戏机会"
                        v-model:enabled="autoFunctions.acceptMatch.enabled"
                        v-model:delay="autoFunctions.acceptMatch.delay"
                      />
                    </div>

                    <!-- 自动选择英雄 -->
                    <div class="card-hover-effect rounded-xl bg-card">
                      <ChampionFunctionCard
                        title="自动选择英雄"
                        description="在选择阶段自动选择指定英雄，快速完成英雄选择"
                        v-model:enabled="autoFunctions.selectChampion.enabled"
                        v-model:delay="autoFunctions.selectChampion.delay"
                        :champion-list="autoFunctions.selectChampion.championList"
                        :show-risk-warning="true"
                        risk-warning-text="⚠️ 请设置适当的延迟"
                        @champion-add="autoFunctionStore.addChampionSelect"
                        @champion-remove="autoFunctionStore.removeChampionSelect"
                        @champion-reorder="autoFunctionStore.reorderChampionSelect"
                        @champion-clear="autoFunctionStore.clearChampionSelect"
                      />
                    </div>

                    <!-- 自动禁用英雄 -->
                    <div class="card-hover-effect rounded-xl bg-card">
                      <ChampionFunctionCard
                        title="自动禁用英雄"
                        description="在禁用阶段自动禁用指定英雄，防止对手选择"
                        v-model:enabled="autoFunctions.banChampion.enabled"
                        v-model:delay="autoFunctions.banChampion.delay"
                        :champion-list="autoFunctions.banChampion.championList"
                        :show-risk-warning="true"
                        risk-warning-text="⚠️ 请设置适当的延迟"
                        @champion-add="autoFunctionStore.addChampionBan"
                        @champion-remove="autoFunctionStore.removeChampionBan"
                        @champion-reorder="autoFunctionStore.reorderChampionBan"
                        @champion-clear="autoFunctionStore.clearChampionBan"
                      />
                    </div>
                  </div>
                </div>
              </div>

              <!-- 符文配置导航提示 -->
              <div class="p-4 rounded-lg bg-primary/5 border border-primary/20">
                <div class="flex items-start gap-3">
                  <Sparkles class="h-5 w-5 text-primary mt-0.5 flex-shrink-0" />
                  <div class="flex-1">
                    <div class="text-sm font-medium text-primary">寻找符文自动配置？</div>
                    <div class="text-xs text-muted-foreground mt-1">
                      完整的符文自动应用和自定义配置功能已移至
                      <span class="font-semibold text-primary">"符文配置"</span> 标签页， 支持智能匹配、OP.GG
                      推荐和自定义符文配置。
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </Card>
        </TabsContent>

        <!-- 智能分析 -->
        <TabsContent value="analysis" class="space-y-8">
          <AnalysisSettingsTab />
        </TabsContent>

        <!-- 符文配置 -->
        <TabsContent value="runes" class="space-y-8">
          <RuneSettingsTab />
        </TabsContent>

        <!-- 快捷键设置 -->
        <TabsContent value="shortcuts" class="space-y-8">
          <Card
            class="p-8 rounded-2xl shadow-xl bg-gradient-to-br from-white/80 to-muted/60 dark:from-background/80 dark:to-muted/40 border border-border"
          >
            <div class="space-y-6">
              <div>
                <h2 class="text-xl font-bold text-primary">快捷键设置</h2>
                <p class="text-sm text-muted-foreground">自定义您的快捷键</p>
              </div>
              <div class="border-t border-dashed border-border pt-6">
                <!-- 这里后续可以放快捷键设置组件 -->
                <div class="text-muted-foreground">敬请期待...</div>
              </div>
            </div>
          </Card>
        </TabsContent>
      </Tabs>

      <!-- 捐赠/支持我们 -->
      <SupportUs />
    </div>
  </div>
</template>

<script setup lang="ts">
import SupportUs from '@/components/common/SupportUs.vue'
import FunctionCard from '@/features/auto-function/components/FunctionCard.vue'
import ChampionFunctionCard from '@/features/auto-function/components/ChampionFunctionCard.vue'
import RuneSettingsTab from './components/rune/RuneSettingsTab.vue'
import AnalysisSettingsTab from './components/analysis/AnalysisSettingsTab.vue'
import { Palette, Gamepad2, Zap, Keyboard, X, Sparkles, BarChart3 } from 'lucide-vue-next'

const settingsStore = useSettingsStore()
const { applyDefaultFilterOnSearch } = storeToRefs(settingsStore)

// 自动化功能相关
const autoFunctionStore = useAutoFunctionStore()
const activityLogger = useActivityLogger()

// 计算属性 - 使用 toRef 确保响应式
const autoFunctions = computed(() => {
  const functions = autoFunctionStore.autoFunctions
  console.log('[SettingsView] autoFunctions computed:', functions)
  return functions
})

const enabledFunctionsCount = computed(() => autoFunctionStore.enabledFunctionsCount)
const isAnyFunctionEnabled = computed(() => autoFunctionStore.isAnyFunctionEnabled)
const enabledFunctions = computed(() => autoFunctionStore.enabledFunctions)

// 禁用所有功能
const handleDisableAll = async () => {
  console.log('Disabling all functions')
  autoFunctionStore.disableAllFunctions()
  activityLogger.log.info('已禁用所有自动功能', 'settings')
}

// Tabs 配置
const tabs = [
  {
    id: 'appearance',
    label: '外观设置',
    icon: Palette
  },
  {
    id: 'game',
    label: '游戏设置',
    icon: Gamepad2
  },
  {
    id: 'automation',
    label: '自动化功能',
    icon: Zap
  },
  {
    id: 'analysis',
    label: '智能分析',
    icon: BarChart3
  },
  {
    id: 'runes',
    label: '符文配置',
    icon: Sparkles
  },
  {
    id: 'shortcuts',
    label: '快捷键设置',
    icon: Keyboard
  }
]

// 从 URL 参数获取初始 tab
const route = useRoute()
const activeTab = ref((route.query.tab as string) || 'appearance')

// 监听路由变化
watch(
  () => route.query.tab,
  (newTab) => {
    if (newTab && typeof newTab === 'string') {
      activeTab.value = newTab
    }
  }
)
</script>

<style scoped>
.card-hover-effect {
  transition: all 0.2s ease-in-out;
}

.card-hover-effect:hover {
  transform: translateY(-2px);
  box-shadow:
    0 8px 25px -5px rgba(0, 0, 0, 0.1),
    0 10px 10px -5px rgba(0, 0, 0, 0.04);
}
</style>
