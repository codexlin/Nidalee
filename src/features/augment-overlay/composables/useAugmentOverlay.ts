import { listen, type Event } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { onMounted, onUnmounted, shallowRef } from 'vue'
import { DEFAULT_OVERLAY_SHORTCUT, formatAccelerator } from '@/shared/utils/accelerator'
import { OVERLAY_CARD_ROUTE } from '@/shared/utils/overlayWindow'

function isCardOverlay() {
  return typeof window !== 'undefined' && window.__NIDALEE_OVERLAY_ROUTE__ === OVERLAY_CARD_ROUTE
}
import type { HextechGuideAugment, HextechGuideTrio } from '@/shared/hextech/guideAugment'

export type OverlayGuideAugment = HextechGuideAugment
export type OverlayGuideTrio = HextechGuideTrio

export type OverlayOffer = {
  id: number | null
  name: string
  rarity: string
  rarityDisplayName: string
  iconUrl: string
  detectedSlot: number
  missing: boolean
  winRate: number | null
  pickRate: number | null
  games: number | null
  recommended: boolean
}

export type AugmentOverlayPayload = {
  success: boolean
  gamePhase: string
  championId: number | null
  championName: string | null
  augments?: OverlayOffer[]
  recommendedAugments: OverlayGuideAugment[]
  recommendedTrios: OverlayGuideTrio[]
  winratePending: boolean
}

export function useAugmentOverlay() {
  const visible = shallowRef(false)
  const currentOffers = shallowRef<OverlayOffer[]>([])
  const recommendedAugments = shallowRef<OverlayGuideAugment[]>([])
  const recommendedTrios = shallowRef<OverlayGuideTrio[]>([])
  const championName = shallowRef<string | null>(null)
  const winratePending = shallowRef(false)
  const toggleShortcut = shallowRef(DEFAULT_OVERLAY_SHORTCUT)

  let unlistenDetected: (() => void) | undefined
  let unlistenCleared: (() => void) | undefined
  let unlistenVisibility: (() => void) | undefined
  let unlistenShortcut: (() => void) | undefined

  onMounted(async () => {
    document.documentElement.classList.add('overlay-shell', 'dark', 'theme-zinc')

    const applyPayload = (payload: AugmentOverlayPayload | null | undefined) => {
      if (!payload?.success) return
      championName.value = payload.championName ?? null
      currentOffers.value = payload.augments ?? []
      recommendedAugments.value = payload.recommendedAugments ?? []
      recommendedTrios.value = payload.recommendedTrios ?? []
      winratePending.value = payload.winratePending
      if (isCardOverlay()) {
        visible.value = (payload.augments ?? []).some((item) => !item.missing)
      }
    }

    unlistenDetected = await listen('augment-detected', (event: Event<AugmentOverlayPayload>) => {
      applyPayload(event.payload)
    })
    unlistenCleared = await listen('augment-cleared', () => {
      visible.value = false
      currentOffers.value = []
      recommendedAugments.value = []
      recommendedTrios.value = []
      championName.value = null
      winratePending.value = false
    })
    unlistenVisibility = await listen<boolean>('augment-overlay-visibility', (event) => {
      if (!isCardOverlay()) visible.value = Boolean(event.payload)
    })
    unlistenShortcut = await listen<string>('augment-overlay-shortcut', (event) => {
      if (event.payload) toggleShortcut.value = formatAccelerator(event.payload)
    })

    try {
      const snapshot = await invoke<AugmentOverlayPayload | null>('get_augment_overlay_snapshot')
      applyPayload(snapshot)
    } catch {
      // 快照命令在旧进程上可能还不存在
    }
    if (!isCardOverlay()) {
      try {
        visible.value = await invoke<boolean>('get_augment_overlay_visible')
      } catch {
        // 旧进程可能还没有显隐命令
      }
    } else {
      visible.value = currentOffers.value.some((item) => !item.missing)
    }
    try {
      const shortcut = await invoke<string>('get_augment_overlay_shortcut')
      if (shortcut) toggleShortcut.value = formatAccelerator(shortcut)
    } catch {
      // 旧进程可能还没有快捷键命令
    }
  })

  onUnmounted(() => {
    document.documentElement.classList.remove('overlay-shell')
    unlistenDetected?.()
    unlistenCleared?.()
    unlistenVisibility?.()
    unlistenShortcut?.()
  })

  async function hideSidePanel() {
    visible.value = false
    await invoke('hide_augment_side_panel')
  }

  async function hide() {
    visible.value = false
    if (isCardOverlay()) {
      await invoke('hide_augment_card_overlay')
      return
    }
    await hideSidePanel()
  }

  return {
    visible,
    augments: currentOffers,
    currentOffers,
    recommendedAugments,
    recommendedTrios,
    championName,
    winratePending,
    toggleShortcut,
    hideSidePanel,
    hide
  }
}
