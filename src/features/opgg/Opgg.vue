<template>
  <div class="min-h-screen">
    <div class="max-w-5xl mx-auto space-y-8">
      <!-- 页面标题 -->
      <div class="text-center space-y-2">
        <h1 class="text-4xl font-extrabold text-primary drop-shadow-sm">🎯 OP.GG 英雄构建推荐</h1>
        <p class="text-lg text-muted-foreground">获取最新的英雄出装、符文和克制关系数据</p>
      </div>

      <!-- 配置面板 -->
      <div
        class="rounded-xl shadow-lg bg-white/80 dark:bg-neutral-900/80 border border-neutral-200 dark:border-neutral-800 p-4"
      >
        <OpggConfigPanel
          :config="opggData.config.value"
          :regions="opggData.regions"
          :modes="opggData.modes"
          :tiers="opggData.tiers"
          :positions="opggData.positions"
          @update:config="handleConfigUpdate"
        />
      </div>

      <!-- 操作按钮 -->
      <div class="flex flex-wrap gap-4 justify-end">
        <OpggActionButtons
          :loading="opggData.state.value.loading"
          :champion-id="opggData.config.value.championId"
          :champion-build="opggData.state.value.championBuild"
          @load-build="opggData.loadChampionBuild"
          @apply-best-runes="handleApplyBestRunes"
        />
      </div>

      <!-- 错误提示 -->
      <Card v-if="opggData.state.value.error" class="border-destructive bg-destructive/10 rounded-xl shadow-md">
        <CardContent class="pt-6">
          <div class="flex items-center gap-3">
            <AlertCircle class="h-4 w-4 text-destructive" />
            <div>
              <h3 class="font-semibold text-destructive">出现错误</h3>
              <p class="text-sm text-destructive/80">{{ opggData.state.value.error }}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- 成功消息 -->
      <Card
        v-if="opggData.state.value.message && !opggData.state.value.message.includes('✨')"
        class="border-blue-200 bg-blue-50/80 dark:border-blue-800 dark:bg-blue-950/80 rounded-xl shadow-md"
      >
        <CardContent>
          <div class="flex items-center gap-3">
            <Info class="h-4 w-4 text-blue-600 dark:text-blue-400" />
            <div>
              <h3 class="font-semibold text-blue-800 dark:text-blue-200">操作提示</h3>
              <p class="text-sm text-blue-700 dark:text-blue-300 whitespace-pre-line">
                {{ opggData.state.value.message }}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- 成功应用符文消息 -->
      <Card
        v-if="opggData.state.value.message && opggData.state.value.message.includes('✨')"
        class="border-green-200 bg-green-50/80 dark:border-green-800 dark:bg-green-950/80 rounded-xl shadow-md"
      >
        <CardContent class="pt-6">
          <div class="flex items-center gap-3">
            <CheckCircle class="h-4 w-4 text-green-600 dark:text-green-400" />
            <div>
              <h3 class="font-semibold text-green-800 dark:text-green-200">符文应用成功</h3>
              <p class="text-sm text-green-700 dark:text-green-300 whitespace-pre-line">
                {{ opggData.state.value.message }}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      <!-- 英雄构建数据显示 -->
      <div v-if="opggData.state.value.championBuild" class="space-y-8">
        <!-- 英雄基本信息 -->
        <div
          class="rounded-xl shadow-lg bg-white/90 dark:bg-neutral-900/90 border border-neutral-200 dark:border-neutral-800 p-4"
        >
          <ChampionSummaryCard :summary="opggData.state.value.championBuild.summary" />
        </div>

        <!-- 推荐符文 -->
        <div
          class="rounded-xl shadow-lg bg-white/90 dark:bg-neutral-900/90 border border-neutral-200 dark:border-neutral-800 p-4"
        >
          <RunesCard :perks="opggData.state.value.championBuild.perks" @apply-runes="handleApplySpecificRunes" />
        </div>

        <!-- 召唤师技能 -->
        <div
          v-if="opggData.state.value.championBuild.summonerSpells?.length"
          class="rounded-xl shadow-lg bg-white/90 dark:bg-neutral-900/90 border border-neutral-200 dark:border-neutral-800 p-4"
        >
          <SummonerSpellsCard :spells="opggData.state.value.championBuild.summonerSpells" />
        </div>

        <!-- 推荐装备 -->
        <div
          v-if="opggData.state.value.championBuild.items"
          class="rounded-xl shadow-lg bg-white/90 dark:bg-neutral-900/90 border border-neutral-200 dark:border-neutral-800 p-4"
        >
          <ItemsCard :items="opggData.state.value.championBuild.items" />
        </div>

        <!-- 技能加点 -->
        <div
          v-if="opggData.state.value.championBuild.championSkills"
          class="rounded-xl shadow-lg bg-white/90 dark:bg-neutral-900/90 border border-neutral-200 dark:border-neutral-800 p-4"
        >
          <SkillsCard :skills="opggData.state.value.championBuild.championSkills" />
        </div>

        <!-- 克制关系 -->
        <div
          v-if="opggData.state.value.championBuild.counters"
          class="rounded-xl shadow-lg bg-white/90 dark:bg-neutral-900/90 border border-neutral-200 dark:border-neutral-800 p-4"
        >
          <CountersCard :counters="opggData.state.value.championBuild.counters" />
        </div>
      </div>

      <!-- 层级列表数据显示 -->
      <Card v-if="opggData.state.value.tierList" class="bg-muted/20 rounded-xl shadow-md">
        <CardHeader>
          <CardTitle class="text-sm">🔍 调试信息 - 英雄层级列表</CardTitle>
          <CardDescription>数据版本: {{ opggData.state.value.tierList.meta?.version }}</CardDescription>
        </CardHeader>
        <CardContent>
          <ScrollArea class="h-96">
            <pre class="text-xs text-muted-foreground">{{
              JSON.stringify(opggData.state.value.tierList, null, 2)
            }}</pre>
          </ScrollArea>
        </CardContent>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { AlertCircle, Info, CheckCircle } from 'lucide-vue-next'
import { useOpggData, type OpggConfig } from './composables/useOpggData'
import { useOpggRunes } from './composables/useOpggRunes'

// 获取数据存储
// const dataStore = useDataStore()

// 使用 composables
const opggData = useOpggData()
const opggRunes = useOpggRunes()

// 处理配置更新
const handleConfigUpdate = (newConfig: OpggConfig) => {
  // 更新配置
  Object.assign(opggData.config.value, newConfig)
}

// 处理符文应用
const handleApplyBestRunes = async () => {
  try {
    await opggRunes.applyBestRunes(opggData.config.value.championId, opggData.config.value)
    if (opggRunes.applySuccess.value) {
      console.log('符文应用成功:', opggRunes.applySuccess.value)
    }
    if (opggRunes.applyError.value) {
      console.log('符文应用失败:', opggRunes.applyError.value)
    }
  } catch (error) {
    console.error('应用符文时发生错误:', error)
  }
}

const handleApplySpecificRunes = async (runeIndex: number) => {
  try {
    await opggRunes.applySpecificRunes(runeIndex, opggData.config.value.championId, opggData.config.value)
    if (opggRunes.applySuccess.value) {
      console.log('符文应用成功:', opggRunes.applySuccess.value)
    }
    if (opggRunes.applyError.value) {
      console.log('符文应用失败:', opggRunes.applyError.value)
    }
  } catch (error) {
    console.error('应用特定符文时发生错误:', error)
  }
}

const route = useRoute()

const loadChampionFromRoute = async (championIdRaw: string | number | (string | null)[] | undefined) => {
  let championId: number | undefined
  if (Array.isArray(championIdRaw)) {
    championId = Number(championIdRaw[0])
  } else {
    championId = Number(championIdRaw)
  }
  if (!championId || isNaN(championId)) return
  opggData.config.value.championId = championId
  await opggData.loadChampionBuild()
}

watch(
  () => route.query.championId,
  (newId) => {
    if (newId) {
      void loadChampionFromRoute(newId)
    }
  },
  { immediate: true }
)
</script>
