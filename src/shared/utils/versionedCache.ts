/**
 * 按游戏版本持久化的轻量 localStorage 缓存。
 * 版本变了即失效，不用 TTL。
 */

interface VersionedPayload<T> {
  version: string
  data: T
}

export function readVersionedCache<T>(storageKey: string, version: string): T | null {
  if (!version) return null
  try {
    const raw = localStorage.getItem(storageKey)
    if (!raw) return null
    const parsed = JSON.parse(raw) as VersionedPayload<T>
    if (!parsed || parsed.version !== version || parsed.data === null || parsed.data === undefined) {
      return null
    }
    return parsed.data
  } catch {
    return null
  }
}

export function writeVersionedCache<T>(storageKey: string, version: string, data: T): void {
  if (!version) return
  try {
    const payload: VersionedPayload<T> = { version, data }
    localStorage.setItem(storageKey, JSON.stringify(payload))
  } catch (error) {
    console.warn(`[versionedCache] 写入失败 ${storageKey}:`, error)
  }
}

export function clearVersionedCache(storageKey: string): void {
  try {
    localStorage.removeItem(storageKey)
  } catch {
    // ignore
  }
}
