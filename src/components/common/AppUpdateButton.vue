<script setup lang="ts">
import { Download, LoaderCircle, RefreshCw } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'
import { useAppUpdater } from '@/shared/composables/app/useAppUpdater'

const updater = useAppUpdater()

const title = computed(() => {
  if (updater.phase.value === 'checking') return '正在检查更新'
  if (updater.phase.value === 'downloading') {
    return updater.progress.value === null ? '正在下载更新' : `正在下载更新 ${updater.progress.value}%`
  }
  if (updater.phase.value === 'installing') return '正在安装更新'
  if (updater.availableVersion.value) {
    return `发现新版本 v${updater.availableVersion.value}，点击安装`
  }
  if (updater.phase.value === 'error') return '检查更新失败，点击重试'
  return '检查更新'
})

const handleClick = () => {
  if (updater.availableVersion.value) {
    void updater.downloadAndInstall()
    return
  }
  void updater.checkForUpdates()
}
</script>

<template>
  <FloatIconButton
    v-if="updater.isSupported.value"
    class="p-2"
    :title="title"
    :disabled="updater.isBusy.value"
    @click="handleClick"
  >
    <LoaderCircle v-if="updater.isBusy.value" class="animate-spin" :size="16" />
    <Download v-else-if="updater.availableVersion.value" :size="16" />
    <RefreshCw v-else :size="16" />

    <span
      v-if="updater.availableVersion.value && !updater.isBusy.value"
      class="bg-primary absolute top-1.5 right-1.5 size-1.5 rounded-full"
      aria-hidden="true"
    />
  </FloatIconButton>
</template>
