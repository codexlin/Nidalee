import { describe, expect, it } from 'vitest'
import { isOverlayWindow, overlayRoute, OVERLAY_SIDE_PANEL_ROUTE } from './overlayWindow'

describe('isOverlayWindow', () => {
  it('returns false outside the overlay webview', () => {
    expect(isOverlayWindow()).toBe(false)
  })
})

describe('overlayRoute', () => {
  it('defaults to the side panel route', () => {
    expect(overlayRoute()).toBe(OVERLAY_SIDE_PANEL_ROUTE)
  })
})
