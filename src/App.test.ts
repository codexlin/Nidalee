import { createSSRApp, h } from 'vue'
import { renderToString } from 'vue/server-renderer'
import { describe, expect, it, vi } from 'vitest'
import App from './App.vue'

const role = vi.hoisted(() => ({ overlay: false }))
const mountedMain = vi.hoisted(() => vi.fn())
vi.mock('./shared/utils/overlayWindow', () => ({ isOverlayWindow: () => role.overlay }))
vi.mock('./MainWindow.vue', () => ({
  default: {
    setup() {
      mountedMain()
      return () => h('main', 'main window')
    }
  }
}))

describe('window startup', () => {
  it.each([true, false])('mounts the correct shell when overlay=%s', async (overlay) => {
    role.overlay = overlay
    const app = createSSRApp(App)
    app.component('RouterView', { render: () => h('aside', 'overlay route') })
    const html = await renderToString(app)
    expect(html).toContain(overlay ? 'overlay route' : 'main window')
    expect(mountedMain).toHaveBeenCalledTimes(overlay ? 0 : 1)
  })
})
