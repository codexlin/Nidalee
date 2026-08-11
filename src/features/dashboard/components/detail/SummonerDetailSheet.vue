<template>
  <Sheet v-model:open="open">
    <SheetContent side="right" class="w-full sm:w-[min(1100px,90vw)] sm:max-w-none overflow-y-auto p-0 gap-0">
      <div
        class="sticky top-0 z-10 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-b border-border p-6 pr-12"
      >
        <SheetHeader class="space-y-0 text-left">
          <SheetTitle class="flex items-center gap-4 text-left">
            <div v-if="currentResult" class="flex items-center gap-4">
              <div class="w-14 h-14 rounded-full bg-muted flex items-center justify-center ring-1 ring-border">
                <span class="text-lg font-bold text-foreground">{{
                  currentResult.displayName?.charAt(0)?.toUpperCase() || '?'
                }}</span>
              </div>
              <div>
                <h3 class="text-lg font-bold text-foreground">{{ currentResult.displayName || '未知召唤师' }}</h3>
                <p class="text-sm text-muted-foreground">召唤师详情与战绩分析</p>
              </div>
            </div>
            <div v-else-if="selectedPlayer" class="flex items-center gap-4">
              <div class="w-14 h-14 rounded-full bg-muted flex items-center justify-center ring-1 ring-border">
                <span class="text-lg font-bold text-foreground">{{
                  selectedPlayer.displayName?.charAt(0)?.toUpperCase() || '?'
                }}</span>
              </div>
              <div>
                <h3 class="text-lg font-bold text-foreground">{{ selectedPlayer.displayName || '未知召唤师' }}</h3>
                <p class="text-sm text-muted-foreground">召唤师详情与战绩分析</p>
              </div>
            </div>
          </SheetTitle>
        </SheetHeader>
      </div>

      <div class="p-6 pt-4 space-y-6">
        <div v-if="loading" class="flex items-center justify-center py-8 gap-3">
          <Spinner class="size-5 text-primary" />
          <span class="text-sm text-muted-foreground">正在查询召唤师战绩…</span>
        </div>

        <div v-else-if="currentResult" class="space-y-6">
          <SummonerCard :summoner-info="currentResult.summonerInfo" />
          <GameStats :is-connected="true" :match-history-loading="false" :match-statistics="currentResult.matches" />
        </div>

        <div v-else class="flex items-center justify-center py-8">
          <div class="text-center">
            <Info class="h-10 w-10 text-muted-foreground mx-auto mb-3" />
            <h3 class="text-base font-semibold mb-1 text-foreground">暂无战绩数据</h3>
            <p class="text-sm text-muted-foreground">未能获取到该召唤师的战绩信息</p>
          </div>
        </div>
      </div>
    </SheetContent>
  </Sheet>
</template>

<script setup lang="ts">
import { Info } from 'lucide-vue-next'

defineProps<{
  selectedPlayer: { displayName: string } | null
  currentResult: SummonerWithMatches | null
  loading: boolean
}>()

const open = defineModel<boolean>('open', { required: true })
</script>
