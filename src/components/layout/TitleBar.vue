<template>
  <div data-tauri-drag-region class="titlebar">
    <div data-tauri-drag-region class="titlebar-col titlebar-left">
      <img src="@/assets/logo.svg" alt="logo" class="logo" />
      <span class="title">Nidalee</span>

      <span class="titlebar-divider" />

      <nav data-tauri-drag-region class="versions tabular-nums select-none">
        <span class="version-item app">
          <Package class="size-3" />
          <span>{{ appVersionLabel }}</span>
        </span>

        <span class="titlebar-divider" />

        <span class="version-item game">
          <Gamepad2 class="size-3" />
          <span>{{ lolGameVersion || '—' }}</span>
        </span>
      </nav>
    </div>

    <div data-tauri-drag-region class="titlebar-col titlebar-center">
      <span class="attribution">Made by <span class="animate-pulse">❤️</span> CodexLin</span>
    </div>

    <div data-tauri-drag-region class="titlebar-col titlebar-right">
      <ConnectionStatus />

      <div v-if="inTauri" class="titlebar-window-controls no-drag">
        <div class="titlebar-button" id="titlebar-minimize" @click="minimizeWindow">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <rect width="10" height="1" x="1" y="5.5" fill="currentColor" />
          </svg>
        </div>
      <div class="titlebar-button" id="titlebar-maximize" @click="toggleMaximize">
        <!-- 还原 -->
        <svg v-if="isMaximized" width="12" height="12" viewBox="0 0 12 12">
          <rect x="3" y="1.5" width="7.5" height="7.5" fill="none" stroke="currentColor" />
          <rect x="1.5" y="3" width="7.5" height="7.5" fill="var(--background)" stroke="currentColor" />
        </svg>
        <!-- 最大化 -->
        <svg v-else width="12" height="12" viewBox="0 0 12 12">
          <rect width="9" height="9" x="1.5" y="1.5" fill="none" stroke="currentColor" />
        </svg>
      </div>
        <div class="titlebar-button close" id="titlebar-close" @click="hideWindow">
          <svg width="12" height="12" viewBox="0 0 12 12">
            <path
              d="M2.4 1.399L1.399 2.4 5 6 1.399 9.6 2.4 10.6 6 7 9.6 10.6 10.6 9.6 7 6 10.6 2.4 9.6 1.399 6 5z"
              fill="currentColor"
            />
          </svg>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { isTauri } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { getCurrentWindow, type Window } from '@tauri-apps/api/window'
import { Gamepad2, Package } from 'lucide-vue-next'
import ConnectionStatus from '@/components/layout/ConnectionStatus.vue'

const inTauri = isTauri()
const appWindow: Window | null = inTauri ? getCurrentWindow() : null

const appVersion = ref('')
const appVersionLabel = computed(() => (appVersion.value ? `v${appVersion.value}` : 'v—'))
const isMaximized = ref(false)
const dataStore = useDataStore()
const lolGameVersion = computed(() => dataStore.gameVersion)

let unlistenResize: (() => void) | undefined

const syncMaximized = async () => {
  if (!appWindow) return
  try {
    isMaximized.value = await appWindow.isMaximized()
  } catch {
    isMaximized.value = false
  }
}

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = ''
  }

  await syncMaximized()
  if (appWindow) {
    try {
      unlistenResize = await appWindow.onResized(() => {
        void syncMaximized()
      })
    } catch {
      // ignore
    }
  }
})

onBeforeUnmount(() => {
  unlistenResize?.()
})

const minimizeWindow = () => {
  void appWindow?.minimize()
}

const toggleMaximize = async () => {
  if (!appWindow) return
  try {
    await appWindow.toggleMaximize()
  } catch (error) {
    console.error('切换最大化失败:', error)
  } finally {
    await syncMaximized()
  }
}

const hideWindow = () => {
  void appWindow?.hide()
}
</script>

<style scoped>
.titlebar {
  height: 40px;
  background: var(--background);
  user-select: none;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 50;
  -webkit-app-region: drag;
  app-region: drag;
}

.titlebar-col {
  display: flex;
  align-items: center;
  min-width: 0;
  height: 100%;
  -webkit-app-region: drag;
  app-region: drag;
}

.titlebar-left {
  justify-content: flex-start;
  padding-left: 12px;
  gap: 8px;
}

.titlebar-center {
  justify-content: center;
  padding: 0 12px;
}

.titlebar-right {
  justify-content: flex-end;
  gap: 10px;
}

.logo {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.title {
  font-size: 14px;
  font-weight: 700;
  color: var(--foreground);
  letter-spacing: -0.01em;
  flex-shrink: 0;
}

.titlebar-divider {
  display: inline-block;
  width: 1px;
  height: 12px;
  background: color-mix(in oklch, var(--border) 60%, transparent);
  flex-shrink: 0;
}

.versions {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  min-width: 0;
  height: 100%;
}

.version-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

.version-item.app {
  color: var(--muted-foreground);
  opacity: 0.85;
}

.version-item.game {
  color: var(--muted-foreground);
  font-weight: 500;
}

.version-item :deep(svg) {
  flex-shrink: 0;
  opacity: 0.65;
}

.attribution {
  font-size: 12px;
  color: color-mix(in oklch, var(--muted-foreground) 70%, transparent);
  letter-spacing: 0.01em;
  white-space: nowrap;
}

.titlebar-window-controls {
  display: flex;
  height: 100%;
}

.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 46px;
  height: 100%;
  color: var(--foreground);
  transition: background-color 120ms ease;
}

.titlebar-button:hover {
  background: var(--muted);
}

.titlebar-button.close:hover {
  background: #e81123;
  color: white;
}

.no-drag,
.no-drag * {
  -webkit-app-region: no-drag !important;
  app-region: no-drag !important;
}

@media (max-width: 900px) {
  .titlebar-center {
    display: none;
  }

  .titlebar {
    grid-template-columns: minmax(0, 1fr) auto;
  }
}
</style>
