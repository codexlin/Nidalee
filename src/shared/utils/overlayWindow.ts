import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

export const OVERLAY_SIDE_PANEL_LABEL = 'augment-side-panel'
export const OVERLAY_SIDE_PANEL_ROUTE = '/augment-side-panel'

declare global {
  interface Window {
    __NIDALEE_OVERLAY__?: boolean
    __NIDALEE_OVERLAY_ROUTE__?: string
  }
}

export function isOverlayWindow(): boolean {
  if (typeof window !== 'undefined' && window.__NIDALEE_OVERLAY__ === true) {
    return true
  }
  if (!isTauri()) {
    return false
  }
  try {
    const label = getCurrentWebviewWindow().label
    return label === OVERLAY_SIDE_PANEL_LABEL
  } catch {
    return false
  }
}

export function overlayRoute(): string {
  if (typeof window !== 'undefined' && window.__NIDALEE_OVERLAY_ROUTE__) {
    return window.__NIDALEE_OVERLAY_ROUTE__
  }
  return OVERLAY_SIDE_PANEL_ROUTE
}

export function markOverlayDocument(): void {
  if (typeof document === 'undefined') return
  const html = document.documentElement
  html.classList.add('overlay-shell', 'dark', 'theme-zinc')
}
