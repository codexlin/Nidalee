<template>
  <div class="flex items-center gap-1.5">
    <NotificationHoverCard title="系统活动" side="bottom" align="end" />

    <FloatIconButton class="p-2" title="刷新数据" @click="refreshData">
      <RefreshCw :size="16" />
    </FloatIconButton>

    <FloatIconButton class="p-2" title="给项目点个 Star" aria-label="GitHub Star" @click="openGithub">
      <Github :size="16" />
    </FloatIconButton>
  </div>
</template>

<script setup lang="ts">
import { Github, RefreshCw } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import FloatIconButton from '@/components/common/FloatIconButton.vue'

const activityLogger = useActivityLogger()
const connectionStore = useConnectionStore()

const { isConnected } = storeToRefs(connectionStore)
const { updateSummonerAndMatches } = useSummonerAndMatchUpdater()

const refreshData = async () => {
  try {
    if (isConnected.value) {
      updateSummonerAndMatches()
    }
  } catch (error) {
    console.error('刷新数据失败:', error)
    activityLogger.logError.apiError('数据刷新失败')
  }
}

const openGithub = () => {
  window.open('https://github.com/codexlin/Nidalee', '_blank', 'noopener,noreferrer')
  setTimeout(() => {
    toast.success('谢谢你的⭐！', {
      description: '开发者收到了你的鼓励，超级开心 🥳',
      duration: 3000
    })
  }, 200)
}
</script>
