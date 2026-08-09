<template>
  <div class="flex flex-col gap-4">
    <SummonerSearchBox v-model:summoner-name="searchText" @on-search="onSearch" />

    <!-- Tab切换：基础统计 vs 位置分组 -->
    <Tabs v-if="currentResult" default-value="basic" class="w-full">
      <TabsList class="grid w-full grid-cols-2">
        <TabsTrigger value="basic">基础统计</TabsTrigger>
        <TabsTrigger value="positions">
          位置分组
          <Badge v-if="searchPositionAnalysis?.positionStats.length" variant="secondary" class="ml-2">
            {{ searchPositionAnalysis.positionStats.length }}
          </Badge>
        </TabsTrigger>
      </TabsList>

      <TabsContent value="basic" class="space-y-4">
        <div v-if="names.length" class="flex gap-2 flex-wrap">
          <Badge
            v-for="(name, idx) in names"
            :key="name"
            :class="[
              'cursor-pointer select-none transition',
              idx === cunrrentIndex ? 'bg-primary text-primary-foreground shadow' : 'bg-muted text-muted-foreground'
            ]"
            @click="cunrrentIndex = idx"
          >
            {{ name }}
          </Badge>
        </div>
        <SummonerCard :summoner-info="currentResult?.summonerInfo" />
        <GameStats
          :is-connected="isConnected"
          :match-history-loading="loading"
          :match-statistics="filteredCurrentMatches || currentResult?.matches"
        />
      </TabsContent>

      <TabsContent value="positions" class="space-y-4">
        <Alert v-if="!searchPositionAnalysis" variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>暂无位置数据</AlertTitle>
          <AlertDescription>本次搜索未附带位置分析，请重新查询召唤师。</AlertDescription>
        </Alert>

        <div v-else class="space-y-4">
          <SummonerCard :summoner-info="currentResult?.summonerInfo" />

          <PositionStatsCard
            :position-stats="searchPositionAnalysis.positionStats"
            :main-position="searchPositionAnalysis.mainPosition"
            @view-details="handlePositionDetails"
          />

          <Card>
            <CardHeader>
              <CardTitle class="text-base">总览数据</CardTitle>
              <CardDescription>
                所有位置合计 · {{ searchPositionAnalysis.overallStats.totalGames }} 场对局
              </CardDescription>
            </CardHeader>
            <CardContent>
              <GameStats
                :is-connected="isConnected"
                :match-history-loading="false"
                :match-statistics="searchPositionAnalysis.overallStats"
              />
            </CardContent>
          </Card>
        </div>
      </TabsContent>
    </Tabs>

    <PositionDetailsDialog
      v-if="selectedPosition"
      :open="showPositionDetails"
      :position-data="selectedPosition"
      @close="closePositionDetails"
    />
  </div>
</template>

<script lang="ts" setup>
import { appContextKey, type AppContext } from '@/types'
import { AlertCircle } from 'lucide-vue-next'
import PositionStatsCard from './PositionStatsCard.vue'
import PositionDetailsDialog from './PositionDetailsDialog.vue'

const { isConnected } = inject(appContextKey) as AppContext

const { onSearch, cunrrentIndex, names, searchText, loading, currentResult, filteredCurrentMatches } =
  useSearchMatches()

const searchPositionAnalysis = computed(() => currentResult.value?.positionAnalysis ?? null)
const selectedPosition = ref<PositionStats | null>(null)
const showPositionDetails = ref(false)

const handlePositionDetails = (pos: PositionStats) => {
  selectedPosition.value = pos
  showPositionDetails.value = true
}

const closePositionDetails = () => {
  showPositionDetails.value = false
  setTimeout(() => {
    selectedPosition.value = null
  }, 300)
}
</script>
