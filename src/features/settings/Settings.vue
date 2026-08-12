<template>
  <div class="flex w-full flex-col gap-4 pb-6">
    <Card class="shrink-0 gap-0 py-0">
      <CardContent class="flex flex-wrap items-start justify-between gap-3 p-4">
        <div class="min-w-0">
          <h1 class="text-xl font-medium leading-tight">设置</h1>
          <p class="mt-1 text-sm text-muted-foreground">自定义系统主题与游戏助手</p>
        </div>
        <div class="flex max-w-full gap-0.5 overflow-x-auto rounded-full surface-inset p-0.5">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            type="button"
            class="inline-flex shrink-0 items-center gap-1.5 rounded-full px-3 py-1.5 text-sm font-medium transition-colors"
            :class="activeTab === tab.id ? 'bg-primary/15 text-primary' : 'text-muted-foreground hover:text-foreground'"
            @click="selectTab(tab.id)"
          >
            <component :is="tab.icon" class="size-3.5 shrink-0" />
            {{ tab.label }}
          </button>
        </div>
      </CardContent>
    </Card>

    <!-- 外观设置 -->
    <div v-if="activeTab === 'appearance'" class="w-full space-y-4">
      <Card class="gap-0 py-0">
        <CardContent class="space-y-6 px-4 py-4 sm:px-5">
          <ThemeCustomizer />
          <div class="space-y-1 border-t border-border/50 pt-4">
            <h3 class="text-sm font-medium">字体声明</h3>
            <p class="text-xs leading-relaxed text-muted-foreground">
              界面字体使用 HarmonyOS Sans Fonts。Copyright 2021 Huawei Device Co., Ltd.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>

    <!-- 游戏设置 -->
    <div v-else-if="activeTab === 'game'" class="w-full space-y-4">
      <Card class="gap-0 overflow-hidden py-0">
        <div class="grid gap-0 border-b border-border/50 lg:grid-cols-2 lg:divide-x lg:divide-border/50">
          <section class="space-y-3 px-4 py-4 sm:px-5">
            <div class="space-y-0.5">
              <h2 class="flex items-center gap-2 text-lg font-medium leading-tight">
                <MessageSquareText class="size-4 text-muted-foreground" />
                个人签名
              </h2>
              <p class="mt-0.5 text-xs text-muted-foreground">自定义你的个性签名</p>
            </div>
            <SummonerNoteEditor embedded />
          </section>

          <section class="space-y-3 border-t border-border/50 px-4 py-4 sm:px-5 lg:border-t-0">
            <div class="space-y-0.5">
              <h2 class="flex items-center gap-2 text-lg font-medium leading-tight">
                <Trophy class="size-4 text-muted-foreground" />
                段位设置
              </h2>
              <p class="mt-0.5 text-xs text-muted-foreground">自定义你的段位信息</p>
            </div>
            <SummonerRankEditor embedded />
          </section>
        </div>

        <section class="space-y-3 px-4 py-4 sm:px-5">
          <div class="space-y-0.5">
            <h2 class="flex items-center gap-2 text-lg font-medium leading-tight">
              <Users class="size-4 text-muted-foreground" />
              生涯背景
            </h2>
            <p class="mt-0.5 text-xs text-muted-foreground">选择英雄并设置皮肤为生涯背景</p>
          </div>
          <ProfileBackgroundManager embedded />
        </section>
      </Card>
    </div>

    <!-- 辅助功能 -->
    <div v-else-if="activeTab === 'automation'" class="w-full space-y-4">
      <Card class="gap-0 py-0">
        <CardContent class="flex flex-wrap items-start justify-between gap-3 p-4">
          <div class="min-w-0 space-y-0.5">
            <h2 class="text-lg font-medium leading-tight">辅助功能</h2>
            <p class="text-xs text-muted-foreground">接受对局、按序选人与禁人</p>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <span class="rounded-full surface-inset px-2.5 py-1 text-xs tabular-nums text-muted-foreground">
              已启用 {{ enabledFunctionsCount }} 项
            </span>
            <Button variant="outline" size="sm" class="h-8" :disabled="!isAnyFunctionEnabled" @click="handleDisableAll">
              <X class="size-3.5" />
              全部关闭
            </Button>
          </div>
        </CardContent>
      </Card>

      <AssistFunctionsPanel
        :accept="autoFunctions.acceptMatch"
        :select="autoFunctions.selectChampion"
        :ban="autoFunctions.banChampion"
        @select-add="autoFunctionStore.addChampionSelect"
        @select-remove="autoFunctionStore.removeChampionSelect"
        @select-clear="autoFunctionStore.clearChampionSelect"
        @select-reorder="autoFunctionStore.reorderChampionSelect"
        @ban-add="autoFunctionStore.addChampionBan"
        @ban-remove="autoFunctionStore.removeChampionBan"
        @ban-clear="autoFunctionStore.clearChampionBan"
        @ban-reorder="autoFunctionStore.reorderChampionBan"
      />

      <button
        type="button"
        class="flex w-full items-start gap-3 rounded-xl surface-inset px-4 py-3 text-left transition-colors hover:bg-muted/40"
        @click="selectTab('runes')"
      >
        <Sparkles class="mt-0.5 size-4 shrink-0 text-primary" />
        <span class="min-w-0 space-y-0.5">
          <span class="block text-sm font-medium">符文辅助已移至「符文配置」</span>
          <span class="block text-xs text-muted-foreground">支持智能匹配、OP.GG 推荐与自定义配置</span>
        </span>
      </button>
    </div>

    <!-- 符文配置 -->
    <div v-else-if="activeTab === 'runes'" class="space-y-4">
      <RuneSettingsTab />
    </div>

    <!-- 快捷键设置 -->
    <div v-else-if="activeTab === 'shortcuts'" class="w-full space-y-4">
      <Card class="gap-0 py-0">
        <CardHeader class="gap-1 px-4 py-3 sm:px-5">
          <CardTitle class="text-lg font-medium leading-tight">快捷键设置</CardTitle>
          <p class="mt-0.5 text-xs text-muted-foreground">自定义快捷键</p>
        </CardHeader>
        <CardContent class="px-4 pb-4 text-sm text-muted-foreground sm:px-5">敬请期待…</CardContent>
      </Card>
    </div>

    <SupportUs />
  </div>
</template>

<script setup lang="ts">
import SupportUs from '@/components/common/SupportUs.vue'
import AssistFunctionsPanel from '@/features/auto-function/components/AssistFunctionsPanel.vue'
import SummonerNoteEditor from '@/features/game-helper/components/SummonerNoteEditor.vue'
import SummonerRankEditor from '@/features/game-helper/components/SummonerRankEditor.vue'
import ProfileBackgroundManager from '@/features/game-helper/components/ProfileBackgroundManager.vue'
import RuneSettingsTab from './components/rune/RuneSettingsTab.vue'
import { Palette, Gamepad2, Zap, Keyboard, X, Sparkles, MessageSquareText, Trophy, Users } from 'lucide-vue-next'

const autoFunctionStore = useAutoFunctionStore()
const activityLogger = useActivityLogger()

const autoFunctions = computed(() => autoFunctionStore.autoFunctions)
const enabledFunctionsCount = computed(() => autoFunctionStore.enabledFunctionsCount)
const isAnyFunctionEnabled = computed(() => autoFunctionStore.isAnyFunctionEnabled)

const handleDisableAll = () => {
  autoFunctionStore.disableAllFunctions()
  activityLogger.log.info('已关闭所有辅助功能', 'settings')
}

// 「智能分析」暂缓：AI BYOK 未就绪前不展示 Tab；组件仍保留在 components/analysis/
const tabs = [
  { id: 'appearance', label: '外观设置', icon: Palette },
  { id: 'game', label: '游戏设置', icon: Gamepad2 },
  { id: 'automation', label: '辅助功能', icon: Zap },
  { id: 'runes', label: '符文配置', icon: Sparkles },
  { id: 'shortcuts', label: '快捷键设置', icon: Keyboard }
] as const

type TabId = (typeof tabs)[number]['id']

const route = useRoute()
const router = useRouter()
const tabIds = new Set<string>(tabs.map((t) => t.id))
const activeTab = ref<TabId>(tabIds.has(String(route.query.tab)) ? (route.query.tab as TabId) : 'appearance')

const selectTab = (id: TabId) => {
  activeTab.value = id
  void router.replace({ query: { ...route.query, tab: id } })
}

watch(
  () => route.query.tab,
  (newTab) => {
    if (typeof newTab === 'string' && tabIds.has(newTab)) {
      activeTab.value = newTab as TabId
    }
  }
)
</script>
