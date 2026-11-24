import type { RouterOptions } from 'vue-router'
import { beforeEach, expect, it, vi } from 'vitest'

const connection = vi.hoisted(() => ({ hasAuth: undefined as boolean | undefined }))
vi.mock('@/shared/stores/core/connectionStore', () => ({ useConnectionStore: () => connection }))
vi.mock('@/shared/utils/overlayWindow', () => ({ isOverlayWindow: () => false }))
vi.mock(import('vue-router'), async (importOriginal) => {
  const actual = await importOriginal()
  return {
    ...actual,
    createWebHistory: actual.createMemoryHistory,
    createRouter: (options: RouterOptions) =>
      actual.createRouter({
        ...options,
        routes: options.routes.map((route) => ({ path: route.path, name: route.name, component: {} }))
      })
  }
})

import router from './index'

beforeEach(async () => {
  connection.hasAuth = undefined
  await router.replace('/')
})

it('keeps a rejected device on the forbidden page across reconnect and direct navigation', async () => {
  connection.hasAuth = false
  await router.replace('/forbidden')
  await router.replace('/')
  expect(router.currentRoute.value.path).toBe('/forbidden')
  await router.push('/settings')
  expect(router.currentRoute.value.path).toBe('/forbidden')
})

it('handles rejection while connected and restores navigation after authorization', async () => {
  await router.replace('/match-search')
  expect(router.currentRoute.value.path).toBe('/match-search')
  connection.hasAuth = false
  const rejection = router.replace('/forbidden')
  const reconnect = router.replace('/')
  await Promise.all([rejection, reconnect])
  expect(router.currentRoute.value.path).toBe('/forbidden')
  connection.hasAuth = true
  await router.replace('/')
  expect(router.currentRoute.value.path).toBe('/')
})
