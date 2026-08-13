import { invoke } from '@tauri-apps/api/core'
import { listen, type Event } from '@tauri-apps/api/event'
import { onMounted, onUnmounted, shallowRef } from 'vue'
import { DEFAULT_OVERLAY_SHORTCUT, formatAccelerator } from '@/shared/utils/accelerator'
import type { HextechGuideAugment, HextechGuideTrio } from '@/shared/hextech/guideAugment'

export function useAugmentOverlay() {
  const visible = shallowRef(false)
  const recommendedAugments = shallowRef<HextechGuideAugment[]>([])
  const recommendedTrios = shallowRef<HextechGuideTrio[]>([])
  const championName = shallowRef<string | null>(null)
  const winratePending = shallowRef(false)
  const toggleShortcut = shallowRef(DEFAULT_OVERLAY_SHORTCUT)

  const cleanup: Array<() => void> = []

  function applyPayload(payload: AugmentDetectedPayload | null) {
    if (!payload?.success) return
    championName.value = payload.championName
    recommendedAugments.value = payload.recommendedAugments
    recommendedTrios.value = payload.recommendedTrios
    winratePending.value = payload.winratePending
  }

  function clearGuide() {
    visible.value = false
    championName.value = null
    recommendedAugments.value = []
    recommendedTrios.value = []
    winratePending.value = false
  }

  onMounted(async () => {
    document.documentElement.classList.add('overlay-shell', 'dark', 'theme-zinc')

    cleanup.push(
      await listen('augment-detected', (event: Event<AugmentDetectedPayload>) => applyPayload(event.payload)),
      await listen('augment-cleared', clearGuide),
      await listen<boolean>('augment-overlay-visibility', (event) => {
        visible.value = event.payload
      }),
      await listen<string>('augment-overlay-shortcut', (event) => {
        toggleShortcut.value = formatAccelerator(event.payload)
      })
    )

    const [snapshot, currentVisible, shortcut] = await Promise.all([
      invoke<AugmentDetectedPayload | null>('get_augment_overlay_snapshot'),
      invoke<boolean>('get_augment_overlay_visible'),
      invoke<string>('get_augment_overlay_shortcut')
    ])
    applyPayload(snapshot)
    visible.value = currentVisible
    toggleShortcut.value = formatAccelerator(shortcut)
  })

  onUnmounted(() => {
    document.documentElement.classList.remove('overlay-shell')
    cleanup.splice(0).forEach((unlisten) => unlisten())
  })

  async function hide() {
    await invoke('hide_augment_side_panel')
  }

  return {
    visible,
    recommendedAugments,
    recommendedTrios,
    championName,
    winratePending,
    toggleShortcut,
    hide
  }
}
