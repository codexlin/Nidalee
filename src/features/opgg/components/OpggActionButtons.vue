<template>
  <div class="flex flex-wrap gap-3">
    <Button @click="onLoadBuild" :disabled="loading || !championId" variant="default" class="flex items-center gap-2">
      <Search class="w-4 h-4" />
      {{ loading ? '加载中...' : '获取详细数据' }}
    </Button>

    <!-- <Button @click="onLoadTierList" :disabled="loading" variant="secondary" class="flex items-center gap-2">
      <List class="w-4 h-4" />
      获取层级列表
    </Button> -->

    <Button
      @click="onApplyBestRunes"
      :disabled="!championBuild || loading"
      variant="default"
      class="flex items-center gap-2 bg-blue-600 hover:bg-blue-700"
    >
      <Zap class="w-4 h-4" />
      应用最佳符文
    </Button>

    <!-- <Button @click="onTestComponent" variant="secondary" class="flex items-center gap-2">
      <TestTube class="w-4 h-4" />
      测试组件功能
    </Button> -->
  </div>
</template>

<script setup lang="ts">
import { Search, Zap } from 'lucide-vue-next'

interface Props {
  loading: boolean
  championId: number
  championBuild: OpggChampionBuild | null
}

interface Emits {
  (e: 'load-build'): void
  (e: 'load-tier-list'): void
  (e: 'apply-best-runes'): void
  (e: 'test-component'): void
}

defineProps<Props>()
const emit = defineEmits<Emits>()

const onLoadBuild = () => emit('load-build')
const onApplyBestRunes = () => emit('apply-best-runes')
</script>
