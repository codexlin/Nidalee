<template>
  <div class="w-full h-full relative">
    <!-- 使用 Transition 实现平滑切换 -->
    <Transition name="fade" mode="out-in">
      <!-- Main Analysis View -->
      <div
        v-if="shouldShowAnalysis && hasMyTeamData"
        key="analysis"
        class="mx-auto h-[calc(100dvh-8.5rem)] min-h-150 w-full max-w-full overflow-hidden"
      >
        <div class="flex h-full min-h-0 flex-col gap-1">
          <!-- Ally Team -->
          <section
            class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-xl border border-blue-500/15 bg-blue-500/[0.02]"
          >
            <AnalysisHeader team-type="ally" :phase="currentPhase" :team-count="myTeam.length" />
            <div class="min-h-0 flex-1 p-1.5">
              <TeamAnalysisCard
                :players="myTeam"
                team-type="ally"
                :local-player-cell-id="localPlayerCellId"
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
            <AnalysisHeader team-type="enemy" :phase="currentPhase" :team-count="enemyTeam.length" />
            <div class="min-h-0 flex-1 p-1.5">
              <TeamAnalysisCard
                :players="enemyTeam"
                team-type="enemy"
                :local-player-cell-id="localPlayerCellId"
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
const { currentPhase, myTeam, enemyTeam, localPlayerCellId, shouldShowAnalysis, hasMyTeamData } =
  storeToRefs(matchAnalysisStore)

// 注释：敌方英雄选择现在由 team-analysis-data 事件自动更新
// 不再需要手动监听 gameStore.champSelectSession

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
