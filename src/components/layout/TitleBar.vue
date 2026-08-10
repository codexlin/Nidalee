<template>
  <div data-tauri-drag-region class="titlebar">
    <div class="titlebar-left">
      <img src="@/assets/logo.svg" alt="logo" class="logo" />
      <span class="title">Nidalee</span>
      <span class="versions tabular-nums select-none">
        <span>{{ appVersionLabel }}</span>
        <span class="versions-sep">·</span>
        <span>游戏 {{ lolGameVersion || '—' }}</span>
        <span class="versions-sep">·</span>
        <span>Made by <span class="animate-pulse"> ❤️ </span> CodexLin</span>
      </span>
    </div>
    <div v-if="inTauri" class="titlebar-right">
      <div class="titlebar-button" id="titlebar-minimize" @click="minimizeWindow">
        <svg width="12" height="12" viewBox="0 0 12 12">
          <rect width="10" height="1" x="1" y="5.5" fill="currentColor" />
        </svg>
      </div>
      <div class="titlebar-button" id="titlebar-maximize" @click="toggleMaximize">
        <svg width="12" height="12" viewBox="0 0 12 12">
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
</template>

<script setup lang="ts">
import { isTauri } from '@tauri-apps/api/core'
import { getVersion } from '@tauri-apps/api/app'
import { getCurrentWindow, type Window } from '@tauri-apps/api/window'

const inTauri = isTauri()
const appWindow: Window | null = inTauri ? getCurrentWindow() : null

const appVersion = ref('')
const appVersionLabel = computed(() => (appVersion.value ? `v${appVersion.value}` : 'v—'))

const dataStore = useDataStore()
const lolGameVersion = computed(() => dataStore.gameVersion)

onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = ''
  }
})

const minimizeWindow = () => {
  void appWindow?.minimize()
}
const toggleMaximize = () => {
  void appWindow?.maximize()
}
const hideWindow = () => {
  void appWindow?.hide()
}
</script>

<style scoped>
.titlebar {
  height: 32px;
  background: var(--background);
  user-select: none;
  display: flex;
  justify-content: space-between;
  align-items: center;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 50;
}

.titlebar-left {
  display: flex;
  align-items: center;
  padding-left: 8px;
  gap: 8px;
  min-width: 0;
}

.logo {
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.title {
  font-size: 12px;
  color: var(--foreground);
  flex-shrink: 0;
}

.versions {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 11px;
  color: var(--muted-foreground);
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.versions-sep {
  opacity: 0.55;
}

.titlebar-right {
  display: flex;
  height: 100%;
  flex-shrink: 0;
}

.titlebar-button {
  display: inline-flex;
  justify-content: center;
  align-items: center;
  width: 46px;
  height: 100%;
  color: var(--foreground);
}

.titlebar-button:hover {
  background: var(--muted);
}

.titlebar-button.close:hover {
  background: #e81123;
  color: white;
}
</style>
