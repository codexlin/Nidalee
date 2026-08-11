import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { clearVersionedCache, readVersionedCache, writeVersionedCache } from './versionedCache'

const KEY = 'nidalee-test-versioned-cache'

describe('versionedCache', () => {
  const store = new Map<string, string>()

  beforeEach(() => {
    store.clear()
    vi.stubGlobal('localStorage', {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => {
        store.set(k, v)
      },
      removeItem: (k: string) => {
        store.delete(k)
      }
    })
  })

  afterEach(() => {
    clearVersionedCache(KEY)
    vi.unstubAllGlobals()
  })

  it('hits only when version matches', () => {
    writeVersionedCache(KEY, '16.1.1', { ok: true })
    expect(readVersionedCache<{ ok: boolean }>(KEY, '16.1.1')).toEqual({ ok: true })
    expect(readVersionedCache(KEY, '16.2.1')).toBeNull()
  })

  it('returns null for empty version', () => {
    writeVersionedCache(KEY, '16.1.1', [1])
    expect(readVersionedCache(KEY, '')).toBeNull()
  })
})
