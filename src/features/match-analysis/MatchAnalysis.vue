<template>
  <div class="w-full h-full relative">
    <!-- 使用 Transition 实现平滑切换 -->
    <Transition name="fade" mode="out-in">
      <!-- Main Analysis View -->
      <div
        v-if="shouldShowAnalysis && hasMyTeamData && isDataReady"
        key="analysis"
        class="mx-auto h-[calc(100dvh-8.5rem)] min-h-150 w-full max-w-full overflow-hidden"
      >
        <div class="flex h-full min-h-0 flex-col gap-1">
          <!-- Ally Team -->
          <section
            class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border border-blue-500/15 bg-blue-500/[0.02]"
          >
            <AnalysisHeader
              team-type="ally"
              :phase="currentPhase"
              :team-count="myTeamData?.players.length || 0"
              :has-data="hasMyTeamData"
              :loading="isLoading"
            />
            <div class="min-h-0 flex-1 p-1.5">
              <TeamAnalysisCard
                :team-data="myTeamData!"
                :team-stats="myTeamStats"
                team-type="ally"
                :is-player-retrying="isPlayerRetrying"
                @select-player="handlePlayerDetails"
                @retry-player="retryPlayer"
              />
            </div>
          </section>

          <!-- Enemy Team -->
          <section
            class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border border-red-500/15 bg-red-500/[0.02]"
          >
            <AnalysisHeader
              team-type="enemy"
              :phase="currentPhase"
              :team-count="enemyTeamData?.players.length || 0"
              :has-data="hasEnemyTeamData"
              :loading="isEnemyTeamLoading"
            />
            <div class="min-h-0 flex-1 p-1.5">
              <TeamAnalysisCard
                :team-data="enemyTeamData!"
                :team-stats="enemyTeamStats"
                team-type="enemy"
                :is-player-retrying="isPlayerRetrying"
                @select-player="handlePlayerDetails"
                @retry-player="retryPlayer"
              />
            </div>
          </section>
        </div>
      </div>

      <!-- Pre-Analysis Status Hub：包一层元素，避免 Transition 对组件根节点告警导致切换卡住 -->
      <div v-else key="status" class="w-full">
        <GameStatusHub />
      </div>
    </Transition>

    <SummonerDetailSheet
      v-model:open="showPlayerDetails"
      :selected-player="selectedPlayer"
      :current-result="currentResult"
      :loading="summonerLoading"
      @refresh="refreshSummoner"
    />
  </div>
</template>

<script setup lang="ts">
import { useMatchAnalysisStore } from './store'
import type { UIPlayerData } from '@/types/match-analysis'
import { useSummonerDetailSheet } from '@/features/dashboard/composables/useSummonerDetailSheet'
import SummonerDetailSheet from '@/features/dashboard/components/detail/SummonerDetailSheet.vue'
import { usePlayerAnalysisRetry } from './composables/usePlayerAnalysisRetry'

// Use Pinia Store
const matchAnalysisStore = useMatchAnalysisStore()
const {
  currentPhase,
  isLoading,
  isEnemyTeamLoading,
  myTeamData,
  myTeamStats,
  enemyTeamData,
  enemyTeamStats,
  shouldShowAnalysis,
  hasMyTeamData,
  hasEnemyTeamData
} = storeToRefs(matchAnalysisStore)

// 注释：敌方英雄选择现在由 team-analysis-data 事件自动更新
// 不再需要手动监听 gameStore.champSelectSession

// 🎨 平滑切换逻辑：添加短暂延迟，确保数据准备完毕再显示
const isDataReady = ref(false)

watch(
  () => shouldShowAnalysis.value && hasMyTeamData.value,
  (shouldShow, _previousValue, onCleanup) => {
    if (shouldShow) {
      // 数据加载完成后，延迟 150ms 再显示，避免闪烁
      isDataReady.value = false
      const readyTimer = setTimeout(() => {
        isDataReady.value = true
      }, 150)
      onCleanup(() => clearTimeout(readyTimer))
    } else {
      isDataReady.value = false
    }
  },
  { immediate: true }
)

const {
  isOpen: showPlayerDetails,
  selectedPlayer,
  currentResult,
  loading: summonerLoading,
  openByDisplayName,
  refresh: refreshSummoner
} = useSummonerDetailSheet()
const { isRetrying: isPlayerRetrying, retryPlayer } = usePlayerAnalysisRetry()

function handlePlayerDetails(player: UIPlayerData): void {
  void openByDisplayName(player.displayName)
}
</script>

<style scoped>
.min-h-screen {
  min-height: 100vh;
}

/* 淡入淡出过渡动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.fade-enter-to,
.fade-leave-from {
  opacity: 1;
}
</style>
