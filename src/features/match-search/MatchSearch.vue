<template>
  <div class="flex flex-col gap-4">
    <SummonerSearchBox v-model:summoner-name="searchText" @on-search="onSearch" />

    <!-- Tab切换：基础统计 vs 位置分组 -->
    <Tabs v-if="currentRestult" default-value="basic" class="w-full">
      <TabsList class="grid w-full grid-cols-2">
        <TabsTrigger value="basic">基础统计</TabsTrigger>
        <TabsTrigger value="positions" @click="loadPositionAnalysis">
          位置分组
          <Badge v-if="positionAnalysis" variant="secondary" class="ml-2">
            {{ positionAnalysis.positionStats.length }}
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
        <SummonerCard :summoner-info="currentRestult?.summonerInfo" />
        <GameStats
          :is-connected="isConnected"
          :match-history-loading="loading"
          :match-statistics="filteredCurrentMatches || currentRestult?.matches"
        />
      </TabsContent>

      <TabsContent value="positions" class="space-y-4">
        <div v-if="positionLoading" class="flex items-center justify-center py-8">
          <div class="text-center space-y-2">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto"></div>
            <p class="text-sm text-muted-foreground">加载位置分析中...</p>
          </div>
        </div>

        <Alert v-else-if="positionError" variant="destructive">
          <AlertCircle class="h-4 w-4" />
          <AlertTitle>加载失败</AlertTitle>
          <AlertDescription>{{ positionError }}</AlertDescription>
        </Alert>

        <div v-else-if="positionAnalysis" class="space-y-4">
          <SummonerCard :summoner-info="currentRestult?.summonerInfo" />

          <!-- 位置统计卡片 -->
          <PositionStatsCard
            :position-stats="positionAnalysis.positionStats"
            :main-position="positionAnalysis.mainPosition"
            @view-details="handlePositionDetails"
          />

          <!-- 总览数据 -->
          <Card>
            <CardHeader>
              <CardTitle class="text-base">总览数据</CardTitle>
              <CardDescription>所有位置合计 · {{ positionAnalysis.overallStats.totalGames }} 场对局</CardDescription>
            </CardHeader>
            <CardContent>
              <GameStats
                :is-connected="isConnected"
                :match-history-loading="false"
                :match-statistics="positionAnalysis.overallStats"
              />
            </CardContent>
          </Card>
        </div>

        <div v-else class="text-center py-8 text-muted-foreground">
          <p>暂无位置分析数据</p>
        </div>
      </TabsContent>
    </Tabs>

    <!-- 位置详情对话框 -->
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

const { onSearch, cunrrentIndex, names, searchText, loading, currentRestult, filteredCurrentMatches } =
  useSearchMatches()

// 位置分析相关
const {
  loading: positionLoading,
  error: positionError,
  positionAnalysis,
  selectedPosition,
  fetchPositionAnalysis,
  selectPosition,
  clearSelectedPosition
} = usePositionAnalysis()

const showPositionDetails = ref(false)

// 加载位置分析数据
const loadPositionAnalysis = async () => {
  if (!positionAnalysis.value && !positionLoading.value) {
    // 默认加载所有排位赛数据（包括单排420和灵活组排440）
    await fetchPositionAnalysis(30, null)
  }
}

// 查看位置详情
const handlePositionDetails = (pos: PositionStats) => {
  selectPosition(pos)
  showPositionDetails.value = true
}

// 关闭位置详情
const closePositionDetails = () => {
  showPositionDetails.value = false
  setTimeout(() => {
    clearSelectedPosition()
  }, 300)
}
</script>
