<template>
  <Sheet v-model:open="open">
    <SheetContent class="w-[500px] sm:w-[700px] lg:w-[900px] xl:w-[1000px] overflow-y-auto p-0">
      <div
        class="sticky top-0 z-10 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-b border-border p-6"
      >
        <SheetHeader>
          <div class="flex items-center justify-between">
            <SheetTitle class="flex items-center gap-4 text-left">
              <div v-if="summonerResult" class="flex items-center gap-4">
                <!-- 使用查询到的召唤师信息 -->
                <div
                  class="w-14 h-14 rounded-full bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center ring-2 ring-primary/20"
                >
                  <span class="text-lg font-bold text-primary">{{
                    summonerResult.displayName?.charAt(0)?.toUpperCase() || '?'
                  }}</span>
                </div>
                <div>
                  <h3 class="text-xl font-bold text-foreground">{{ summonerResult.displayName || '未知召唤师' }}</h3>
                  <p class="text-sm text-muted-foreground">召唤师详情与战绩分析</p>
                </div>
              </div>
              <div v-else-if="summoner" class="flex items-center gap-4">
                <!-- 查询中或失败时显示原始玩家信息 -->
                <div
                  class="w-14 h-14 rounded-full bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center ring-2 ring-primary/20"
                >
                  <span class="text-lg font-bold text-primary">{{
                    summoner.displayName?.charAt(0)?.toUpperCase() || '?'
                  }}</span>
                </div>
                <div>
                  <h3 class="text-xl font-bold text-foreground">{{ summoner.displayName || '未知召唤师' }}</h3>
                  <p class="text-sm text-muted-foreground">召唤师详情与战绩分析</p>
                </div>
              </div>
            </SheetTitle>

            <SheetClose
              class="rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-secondary text-muted-foreground hover:text-foreground"
            >
              <X class="h-4 w-4" />
              <span class="sr-only">关闭</span>
            </SheetClose>
          </div>
        </SheetHeader>
      </div>

      <div class="p-6 pt-4 space-y-6">
        <!-- 加载状态 -->
        <div v-if="loading" class="flex items-center justify-center py-8">
          <div class="flex items-center gap-3">
            <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
            <span class="text-muted-foreground">正在查询召唤师战绩...</span>
          </div>
        </div>

        <!-- 战绩数据 -->
        <div v-else-if="summonerResult" class="space-y-6">
          <!-- 召唤师信息卡片 -->
          <SummonerCard v-if="summonerResult.summonerInfo" :summoner-info="summonerResult.summonerInfo" />

          <!-- 游戏统计 -->
          <GameStats
            v-if="summonerResult.matches"
            :is-connected="true"
            :match-history-loading="false"
            :match-statistics="summonerResult.matches"
            :ranked-stats="summonerResult.positionAnalysis?.rankedStats"
            :other-stats="summonerResult.positionAnalysis?.otherStats"
            :position-stats="summonerResult.positionAnalysis?.positionStats"
            :main-position="summonerResult.positionAnalysis?.mainPosition"
          />
        </div>

        <!-- 无数据状态 -->
        <div v-else class="flex items-center justify-center py-8">
          <div class="text-center">
            <Info class="h-12 w-12 text-muted-foreground mx-auto mb-4" />
            <h3 class="text-lg font-semibold mb-2 text-foreground">暂无战绩数据</h3>
            <p class="text-muted-foreground">未能获取到该召唤师的战绩信息</p>
          </div>
        </div>
      </div>
    </SheetContent>
  </Sheet>
</template>

<script setup lang="ts">
import { Info, X } from 'lucide-vue-next'

interface Props {
  open: boolean
  summoner?: SummonerInfo | { displayName?: string }
  summonerResult?: SummonerWithMatches // 查询结果数据
  loading?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false
})

const emit = defineEmits<{
  'update:open': [value: boolean]
  close: []
}>()

const open = computed({
  get: () => props.open,
  set: (value) => {
    emit('update:open', value)
    if (!value) {
      emit('close')
    }
  }
})
</script>
