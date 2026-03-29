<template>
  <Dialog :open="visible" @update:open="$emit('update:visible', $event)">
    <DialogContent class="!max-w-[90vw] w-[85vw] h-[85vh] bg-background text-foreground">
      <DialogHeader>
        <DialogTitle>游戏详细信息</DialogTitle>
        <DialogDescription v-if="selectedGame">
          {{ getChampionName(selectedGame.championId) }} - {{ getQueueName(selectedGame.queueId) }} -
          {{ formatRelativeTime(selectedGame.gameCreation) }}
        </DialogDescription>
      </DialogHeader>

      <ScrollArea class="max-h-[60vh] border-none will-change-auto">
        <div v-if="loading" class="flex items-center justify-center py-12">
          <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
          <span class="ml-3 text-muted-foreground">正在加载游戏详细信息...</span>
        </div>

        <!-- 详细信息内容 -->
        <div v-else-if="gameDetailData" class="space-y-6 px-2.5 will-change-auto">
          <!-- 队伍信息 -->
          <div class="grid grid-cols-1 gap-6">
            <!-- 蓝队 -->
            <Card class="bg-blue-50/50 dark:bg-blue-950/30 px-2">
              <div
                class="p-2 flex items-center font-bold text-blue-700 dark:text-blue-200 border-b border-blue-200 dark:border-blue-800"
              >
                <span class="mr-2">蓝队 ({{ getTeamResult('100') }})</span>
                <span class="ml-auto text-xs font-normal flex items-center">
                  击杀: {{ gameDetailData?.blueTeamStats?.kills || 0 }} | 经济:
                  {{ formatNumber(gameDetailData?.blueTeamStats?.goldEarned || 0) }} | 伤害:
                  {{ formatNumber(gameDetailData?.blueTeamStats?.totalDamageDealtToChampions || 0) }}
                  | 视野: {{ gameDetailData?.blueTeamStats?.visionScore || 0 }} | BAN:
                  <span
                    v-for="ban in getTeamBans('100', gameDetailData?.teams)"
                    :key="ban.championId"
                    class="inline-block mx-0.5"
                  >
                    <img
                      :src="getChampionIconUrl(ban.championId)"
                      class="h-6 w-6 rounded"
                      :title="getChampionName(ban.championId)"
                    />
                  </span>
                </span>
              </div>
              <Table class="table-fixed w-full">
                <TableHeader>
                  <TableRow>
                    <TableHead v-for="column in columns" :key="column.key" :class="column.class">{{
                      column.label
                    }}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow v-for="participant in getTeamParticipants('100')" :key="participant.participantId">
                    <TableCell class="flex items-center space-x-2">
                      <img :src="getProfileIconUrl(participant.profileIconId)" class="h-8 w-8 rounded-full" />
                      <div class="flex-1 flex items-center justify-between min-w-0">
                        <span
                          class="font-medium truncate cursor-pointer hover:text-primary transition-colors"
                          @click="openSummonerDetails(participant)"
                          title="点击查看召唤师详情"
                          >{{ participant.summonerName }}</span
                        >
                        <div class="flex items-center gap-1">
                          <button
                            class="text-primary hover:text-primary/80 focus:outline-none"
                            @click="copyName(participant.summonerName)"
                            title="复制召唤师名"
                          >
                            <svg
                              xmlns="http://www.w3.org/2000/svg"
                              class="inline h-4 w-4"
                              fill="none"
                              viewBox="0 0 24 24"
                              stroke="currentColor"
                            >
                              <rect
                                x="9"
                                y="9"
                                width="13"
                                height="13"
                                rx="2"
                                stroke="currentColor"
                                stroke-width="2"
                                fill="none"
                              />
                              <rect
                                x="3"
                                y="3"
                                width="13"
                                height="13"
                                rx="2"
                                stroke="currentColor"
                                stroke-width="2"
                                fill="none"
                              />
                            </svg>
                          </button>
                          <img
                            v-if="participant.rankTier"
                            :src="getRankIconUrl(participant.rankTier)"
                            class="h-6 w-6"
                          />
                        </div>
                      </div>
                    </TableCell>
                    <TableCell class="relative text-center">
                      <div class="flex items-center gap-1 justify-center">
                        <div class="relative">
                          <img
                            :src="getChampionIconUrl(participant.championId)"
                            class="h-8 w-8"
                            :title="getChampionName(participant.championId)"
                          />
                          <span
                            class="absolute -bottom-1 -right-1 bg-gray-900/75 text-white text-[10px] min-w-[16px] h-4 flex items-center justify-center rounded"
                          >
                            {{ participant.stats?.champLevel || '?' }}
                          </span>
                        </div>
                        <span class="text-sm font-medium">{{ getChampionName(participant.championId) }}</span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div class="flex items-center justify-center gap-1 w-full">
                        <img
                          v-for="i in itemSlots"
                          :key="i"
                          :src="getItemIconUrl((participant.stats as any)?.[`item${i}`] || 0, gameVersion)"
                          class="h-6 w-6 rounded bg-gray-100 dark:bg-gray-800"
                          :style="{ opacity: (participant.stats as any)?.[`item${i}`] ? 1 : 0.3 }"
                          :alt="
                            (participant.stats as any)?.[`item${i}`]
                              ? `装备 ${(participant.stats as any)[`item${i}`]}`
                              : '空装备槽'
                          "
                        />
                      </div>
                    </TableCell>
                    <TableCell class="text-center">
                      {{ participant.stats?.kills }}/{{ participant.stats?.deaths }}/{{ participant.stats?.assists }}
                    </TableCell>
                    <TableCell class="text-center text-yellow-700 dark:text-yellow-300">
                      {{ formatNumber(participant.stats?.goldEarned || 0) }}
                    </TableCell>
                    <TableCell class="text-center text-blue-700 dark:text-blue-300">
                      {{ formatNumber(participant.stats?.totalDamageDealtToChampions || 0) }}
                    </TableCell>
                    <TableCell class="text-center font-bold text-lg text-indigo-700 dark:text-indigo-300">
                      {{ participant.score || '-' }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </Card>

            <!-- 红队 -->
            <Card class="bg-red-50/50 dark:bg-red-950/30 px-2">
              <div
                class="p-2 flex items-center font-bold text-red-700 dark:text-red-200 border-b border-red-200 dark:border-red-800"
              >
                <span class="mr-2">红队 ({{ getTeamResult('200') }})</span>
                <span class="ml-auto text-xs font-normal flex items-center">
                  击杀: {{ gameDetailData?.redTeamStats?.kills || 0 }} | 经济:
                  {{ formatNumber(gameDetailData?.redTeamStats?.goldEarned || 0) }} | 伤害:
                  {{ formatNumber(gameDetailData?.redTeamStats?.totalDamageDealtToChampions || 0) }}
                  | 视野: {{ gameDetailData?.redTeamStats?.visionScore || 0 }} | BAN:
                  <span
                    v-for="ban in getTeamBans('200', gameDetailData?.teams)"
                    :key="ban.championId"
                    class="inline-block mx-0.5"
                  >
                    <img
                      :src="getChampionIconUrl(ban.championId)"
                      class="h-6 w-6 rounded"
                      :title="getChampionName(ban.championId)"
                    />
                  </span>
                </span>
              </div>
              <Table class="table-fixed w-full">
                <TableHeader>
                  <TableRow>
                    <TableHead v-for="column in columns" :key="column.key" :class="column.class">{{
                      column.label
                    }}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow v-for="participant in getTeamParticipants('200')" :key="participant.participantId">
                    <TableCell class="flex items-center space-x-2">
                      <img :src="getProfileIconUrl(participant.profileIconId)" class="h-8 w-8 rounded-full" />
                      <div class="flex-1 flex items-center justify-between min-w-0">
                        <span
                          class="font-medium truncate cursor-pointer hover:text-primary transition-colors"
                          @click="openSummonerDetails(participant)"
                          title="点击查看召唤师详情"
                          >{{ participant.summonerName }}</span
                        >
                        <div class="flex items-center gap-1">
                          <button
                            class="text-primary hover:text-primary/80 focus:outline-none"
                            @click="copyName(participant.summonerName)"
                            title="复制召唤师名"
                          >
                            <svg
                              xmlns="http://www.w3.org/2000/svg"
                              class="inline h-4 w-4"
                              fill="none"
                              viewBox="0 0 24 24"
                              stroke="currentColor"
                            >
                              <rect
                                x="9"
                                y="9"
                                width="13"
                                height="13"
                                rx="2"
                                stroke="currentColor"
                                stroke-width="2"
                                fill="none"
                              />
                              <rect
                                x="3"
                                y="3"
                                width="13"
                                height="13"
                                rx="2"
                                stroke="currentColor"
                                stroke-width="2"
                                fill="none"
                              />
                            </svg>
                          </button>
                        </div>
                      </div>
                    </TableCell>
                    <TableCell class="relative text-center">
                      <div class="flex items-center gap-1 justify-center">
                        <div class="relative">
                          <img
                            :src="getChampionIconUrl(participant.championId)"
                            class="h-8 w-8"
                            :title="getChampionName(participant.championId)"
                          />
                          <span
                            class="absolute -bottom-1 -right-1 bg-gray-900/75 text-white text-[10px] min-w-[16px] h-4 flex items-center justify-center rounded"
                          >
                            {{ participant.stats?.champLevel || '?' }}
                          </span>
                        </div>
                        <span class="text-sm font-medium">
                          {{ getChampionName(participant.championId) }}
                        </span>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div class="flex items-center justify-center gap-1 w-full">
                        <img
                          v-for="i in itemSlots"
                          :key="i"
                          :src="getItemIconUrl((participant.stats as any)?.[`item${i}`] || 0, gameVersion)"
                          class="h-6 w-6 rounded bg-gray-100 dark:bg-gray-800"
                          :style="{
                            opacity: (participant.stats as any)?.[`item${i}`] ? 1 : 0.3
                          }"
                          :alt="
                            (participant.stats as any)?.[`item${i}`]
                              ? `装备 ${(participant.stats as any)[`item${i}`]}`
                              : '空装备槽'
                          "
                        />
                      </div>
                    </TableCell>
                    <TableCell class="text-center">
                      {{ participant.stats?.kills }}/{{ participant.stats?.deaths }}/{{ participant.stats?.assists }}
                    </TableCell>
                    <TableCell class="text-center text-yellow-700 dark:text-yellow-300">
                      {{ formatNumber(participant.stats?.goldEarned || 0) }}
                    </TableCell>
                    <TableCell class="text-center text-blue-700 dark:text-blue-300">
                      {{ formatNumber(participant.stats?.totalDamageDealtToChampions || 0) }}
                    </TableCell>
                    <TableCell class="text-center font-bold text-lg text-indigo-700 dark:text-indigo-300">
                      {{ participant.score || '-' }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </Card>
          </div>

          <!-- 单项最佳 -->
          <div class="flex gap-4 mt-4">
            <Card class="flex-1 p-4 text-center">
              <img :src="getChampionIconUrl(gameDetailData.bestPlayerChampionId as number)" class="h-10 w-10 mx-auto" />
              <div class="font-bold text-lg mt-2">{{ gameDetailData.maxDamage }}</div>
              <div class="text-xs text-muted-foreground">最高英雄伤害</div>
            </Card>
            <Card class="flex-1 p-4 text-center">
              <img :src="getChampionIconUrl(gameDetailData.maxTankChampionId as number)" class="h-10 w-10 mx-auto" />
              <div class="font-bold text-lg mt-2">{{ gameDetailData.maxTank }}</div>
              <div class="text-xs text-muted-foreground">最高承受伤害</div>
            </Card>
            <Card class="flex-1 p-4 text-center">
              <img :src="getChampionIconUrl(gameDetailData.maxStreakChampionId as number)" class="h-10 w-10 mx-auto" />
              <div class="font-bold text-lg mt-2">{{ gameDetailData.maxStreak }}</div>
              <div class="text-xs text-muted-foreground">最多连杀</div>
            </Card>
          </div>
          <!-- 基本游戏信息 -->
          <Card>
            <div class="px-4">
              <h4 class="font-semibold mb-3">基本信息</h4>
              <div class="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                <div>
                  <span class="text-muted-foreground">游戏ID:</span>
                  <p class="font-mono">{{ gameDetailData.gameId }}</p>
                </div>
                <div>
                  <span class="text-muted-foreground">游戏时长:</span>
                  <p>{{ formatDuration(gameDetailData.gameDuration || 0) }}</p>
                </div>
                <div>
                  <span class="text-muted-foreground">地图:</span>
                  <p>{{ getMapName(gameDetailData.mapId) }}</p>
                </div>
                <div>
                  <span class="text-muted-foreground">游戏模式:</span>
                  <p>{{ formatGameMode(gameDetailData.gameMode || '') }}</p>
                </div>
                <div>
                  <span class="text-muted-foreground">队列类型:</span>
                  <p>{{ getQueueName(gameDetailData.queueId || 0) }}</p>
                </div>
                <div>
                  <span class="text-muted-foreground">游戏版本:</span>
                  <p class="text-xs font-mono">{{ gameDetailData.gameVersion }}</p>
                </div>
              </div>
            </div>
          </Card>
        </div>
      </ScrollArea>
    </DialogContent>

    <!-- 召唤师详情 -->
    <Sheet v-model:open="isDetailsOpen">
      <SheetContent class="w-[500px] sm:w-[700px] lg:w-[900px] xl:w-[1000px] overflow-y-auto p-0">
        <div
          class="sticky top-0 z-10 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60 border-b border-border p-6"
        >
          <SheetHeader>
            <div class="flex items-center justify-between">
              <SheetTitle class="flex items-center gap-4 text-left">
                <div v-if="currentRestult" class="flex items-center gap-4">
                  <!-- 使用查询到的召唤师信息 -->
                  <div
                    class="w-14 h-14 rounded-full bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center ring-2 ring-primary/20"
                  >
                    <span class="text-lg font-bold text-primary">{{
                      currentRestult.displayName?.charAt(0)?.toUpperCase() || '?'
                    }}</span>
                  </div>
                  <div>
                    <h3 class="text-xl font-bold text-foreground">{{ currentRestult.displayName || '未知召唤师' }}</h3>
                    <p class="text-sm text-muted-foreground">召唤师详情与战绩分析</p>
                  </div>
                </div>
                <div v-else-if="selectedPlayer" class="flex items-center gap-4">
                  <!-- 查询中或失败时显示原始玩家信息 -->
                  <div
                    class="w-14 h-14 rounded-full bg-gradient-to-br from-primary/20 to-primary/10 flex items-center justify-center ring-2 ring-primary/20"
                  >
                    <span class="text-lg font-bold text-primary">{{
                      selectedPlayer.displayName?.charAt(0)?.toUpperCase() || '?'
                    }}</span>
                  </div>
                  <div>
                    <h3 class="text-xl font-bold text-foreground">{{ selectedPlayer.displayName || '未知召唤师' }}</h3>
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
          <div v-if="searchLoading" class="flex items-center justify-center py-8">
            <div class="flex items-center gap-3">
              <div class="animate-spin rounded-full h-6 w-6 border-b-2 border-primary"></div>
              <span class="text-muted-foreground">正在查询召唤师战绩...</span>
            </div>
          </div>

          <!-- 战绩数据 -->
          <div v-else-if="currentRestult" class="space-y-6">
            <!-- 召唤师信息卡片 -->
            <SummonerCard :summoner-info="currentRestult.summonerInfo" />

            <!-- 游戏统计 -->
            <GameStats :is-connected="true" :match-history-loading="false" :match-statistics="currentRestult.matches" />
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
  </Dialog>
</template>

<script setup lang="ts">
import {
  getChampionIconUrl,
  getChampionName,
  getItemIconUrl,
  getMapName,
  getProfileIconUrl,
  getQueueName,
  getRankIconUrl
} from '@/lib'
import { invoke } from '@tauri-apps/api/core'
import { useClipboard } from '@vueuse/core'
import { toast } from 'vue-sonner'
import { Info, X } from 'lucide-vue-next'

const props = defineProps<{
  selectedGame: RecentGame | null
}>()

const visible = defineModel<boolean>('visible')

const activityLogger = useActivityLogger()
const { formatGameMode, formatRelativeTime } = useFormatters()

const loading = ref(false)
const gameDetailData = ref<GameDetail | null>(null)
const dataStore = useDataStore()
const gameVersion = computed(() => dataStore.gameVersion)

const isDetailsOpen = ref(false)
const selectedPlayer = ref<any>(null)

const { fetchSummonerInfo, currentRestult, loading: searchLoading } = useSearchMatches()

// 监听游戏数据变化
watch(
  () => props.selectedGame,
  async (newGame) => {
    if (newGame) {
      loading.value = true
      try {
        const result = await invoke<GameDetail>('get_game_detail', {
          gameId: newGame.gameId
        })
        gameDetailData.value = result
        console.log('gameDetailData', gameDetailData.value)
      } catch (err) {
        console.error('获取游戏详细信息失败:', err)
        activityLogger.logError.apiError(`获取游戏详细信息失败: ${err}`)
      } finally {
        loading.value = false
      }
    }
  }
)

const openSummonerDetails = async (participant: any) => {
  selectedPlayer.value = {
    displayName: participant.summonerName
  }
  isDetailsOpen.value = true
  if (
    participant.summonerName &&
    participant.summonerName !== '未知玩家' &&
    participant.summonerName !== '未知召唤师'
  ) {
    await fetchSummonerInfo([participant.summonerName])
  }
}

const columns = [
  { key: 'summoner', label: '召唤师', class: 'w-[22%]' },
  { key: 'champion', label: '英雄', class: 'w-[15%] text-center' },
  { key: 'items', label: '装备', class: 'w-[25%] text-center' },
  { key: 'kda', label: 'KDA', class: 'w-[10%] text-center' },
  { key: 'gold', label: '经济', class: 'w-[10%] text-center' },
  { key: 'damage', label: '伤害', class: 'w-[10%] text-center' },
  { key: 'score', label: '评分', class: 'w-[8%] text-center' }
]

// 装备槽位
const itemSlots = [0, 1, 2, 3, 4, 5, 6]

const formatDuration = (seconds: number) => {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}分 ${s}秒`
}
const formatNumber = (num: number) => num.toLocaleString()

const getTeamResult = (teamId: string) => {
  if (!gameDetailData.value) return '未知'
  const team = gameDetailData.value.teams.find((t) => t.teamId && t.teamId.toString() === teamId)
  if (!team || !team.win) {
    return '未知'
  }
  return team.win === 'Win' ? '胜利' : '失败'
}

const getTeamBans = (teamId: string, teams: any[]) => {
  if (!teams) return []
  const team = teams.find((t) => t.teamId && t.teamId.toString() === teamId)
  return team?.bans || []
}

const getTeamParticipants = (teamId: string) => {
  if (!gameDetailData.value?.participants) return []
  return gameDetailData.value.participants.filter((p) => p.teamId.toString() === teamId)
}

const clipboard = useClipboard()

function copyName(name: string) {
  clipboard.copy(name)
  toast.success('已复制召唤师名到剪贴板')
}
</script>
<style scoped>
/* 确保在内容不足时，ScrollArea 不会显示滚动条 */
[data-radix-scroll-area-viewport] {
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE 10+ */
}

[data-radix-scroll-area-viewport]::-webkit-scrollbar {
  display: none; /* Chrome, Safari, Opera*/
}
</style>
