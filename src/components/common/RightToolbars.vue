<template>
  <div class="px-4">
    <div class="flex items-center gap-2">
      <NotificationHoverCard title="系统活动" side="bottom" align="end" />

      <FloatIconButton title="刷新数据" @click="refreshData">
        <RefreshCw :size="17" />
      </FloatIconButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import { RefreshCw } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'

const activityLogger = useActivityLogger()
const connectionStore = useConnectionStore()

const { isConnected } = storeToRefs(connectionStore)
const { updateSummonerAndMatches } = useSummonerAndMatchUpdater()

const refreshData = async () => {
  console.log('刷新数据')
  try {
    if (isConnected.value) {
      updateSummonerAndMatches()
    }
  } catch (error) {
    console.error('刷新数据失败:', error)
    activityLogger.logError.apiError('数据刷新失败')
  }
}
</script>

<style></style>
