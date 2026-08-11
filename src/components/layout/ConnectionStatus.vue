<template>
  <div
    class="titlebar-status no-drag"
    :class="{ 'is-offline': !isConnected }"
    :title="statusTitle"
  >
    <span :class="['status-dot', isConnected ? 'is-online' : 'is-offline']" />
    <span class="status-name">
      {{ isConnected ? summonerInfo?.displayName || '未知召唤师' : '未连接' }}
    </span>
    <GameLauncher v-if="!isConnected" />
  </div>
</template>

<script setup lang="ts">
import GameLauncher from '@/components/common/GameLauncher.vue'

const dataStore = useDataStore()
const connectionStore = useConnectionStore()

const { summonerInfo } = storeToRefs(dataStore)
const { isConnected } = storeToRefs(connectionStore)

const statusTitle = computed(() => {
  if (!isConnected.value) return '客户端未连接'
  const level = summonerInfo.value?.summonerLevel
  return level ? `已连接 · 等级 ${level}` : '已连接'
})
</script>

<style scoped>
.titlebar-status {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  max-width: 320px;
  min-width: 0;
  height: 26px;
  padding: 0 8px;
  border-radius: 9999px;
  border: 1px solid color-mix(in oklch, var(--border) 40%, transparent);
  background: color-mix(in oklch, var(--muted) 28%, transparent);
  color: var(--muted-foreground);
}

.titlebar-status.is-offline {
  border-color: color-mix(in oklch, var(--destructive) 40%, transparent);
}

.titlebar-status :deep(button) {
  color: inherit;
}

.status-dot {
  width: 6px;
  height: 6px;
  border-radius: 9999px;
  flex-shrink: 0;
}

.status-dot.is-online {
  background: var(--color-emerald-500, #10b981);
}

.status-dot.is-offline {
  background: var(--destructive);
}

.status-name {
  min-width: 0;
  max-width: 7.5rem;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-weight: 500;
  line-height: 1;
  color: var(--foreground);
}

.no-drag {
  -webkit-app-region: no-drag;
  app-region: no-drag;
}
</style>
