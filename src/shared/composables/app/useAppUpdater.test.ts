import type { CheckOptions, DownloadEvent, DownloadOptions, Update } from '@tauri-apps/plugin-updater'
import { describe, expect, it, vi } from 'vitest'
import { createAppUpdater } from './useAppUpdater'

const createUpdate = (
  overrides: Partial<{
    downloadAndInstall: (onEvent?: (event: DownloadEvent) => void, options?: DownloadOptions) => Promise<void>
    close: () => Promise<void>
  }> = {}
) =>
  ({
    rid: 1,
    currentVersion: '2.1.6',
    version: '3.0.0',
    body: '## 更新内容\n- 修复实时分析',
    date: '2026-08-17T00:00:00Z',
    rawJson: {},
    downloadAndInstall: vi.fn().mockResolvedValue(undefined),
    close: vi.fn().mockResolvedValue(undefined),
    ...overrides
  }) as unknown as Update

const createHarness = (checkResult: Promise<Update | null>) => {
  const notify = {
    current: vi.fn(),
    installed: vi.fn(),
    failed: vi.fn()
  }
  const relaunch = vi.fn().mockResolvedValue(undefined)
  const check = vi.fn((_options?: CheckOptions) => checkResult)

  const updater = createAppUpdater({
    isTauri: () => true,
    isDevelopment: () => false,
    check,
    relaunch,
    notify
  })

  return { updater, check, relaunch, notify }
}

describe('useAppUpdater', () => {
  it('coalesces concurrent update checks', async () => {
    let resolveCheck: (update: Update | null) => void = () => {}
    const pending = new Promise<Update | null>((resolve) => {
      resolveCheck = resolve
    })
    const { updater, check } = createHarness(pending)

    const first = updater.checkForUpdates({ silent: true })
    const second = updater.checkForUpdates({ silent: true })

    expect(first).toBe(second)
    expect(check).toHaveBeenCalledTimes(1)

    resolveCheck(null)
    await first
    expect(updater.phase.value).toBe('idle')
  })

  it('exposes an available version and its release notes', async () => {
    const update = createUpdate()
    const { updater } = createHarness(Promise.resolve(update))

    await updater.checkForUpdates({ silent: true })

    expect(updater.phase.value).toBe('available')
    expect(updater.availableVersion.value).toBe('3.0.0')
    expect(updater.availableNotes.value).toBe('## 更新内容\n- 修复实时分析')
    expect(updater.availableDate.value).toBe('2026-08-17T00:00:00Z')
  })

  it('tracks download progress, installs, and relaunches', async () => {
    const downloadAndInstall = vi.fn(async (onEvent?: (event: DownloadEvent) => void, _options?: DownloadOptions) => {
      onEvent?.({ event: 'Started', data: { contentLength: 100 } })
      onEvent?.({ event: 'Progress', data: { chunkLength: 40 } })
      onEvent?.({ event: 'Progress', data: { chunkLength: 60 } })
      onEvent?.({ event: 'Finished' })
    })
    const update = createUpdate({ downloadAndInstall })
    const { updater, relaunch, notify } = createHarness(Promise.resolve(update))

    await updater.checkForUpdates({ silent: true })
    await updater.downloadAndInstall()

    expect(downloadAndInstall).toHaveBeenCalledTimes(1)
    expect(updater.progress.value).toBe(100)
    expect(updater.phase.value).toBe('installing')
    expect(notify.installed).toHaveBeenCalledWith('3.0.0')
    expect(relaunch).toHaveBeenCalledTimes(1)
  })

  it('keeps the available update retryable after an install failure', async () => {
    const update = createUpdate({
      downloadAndInstall: vi.fn().mockRejectedValue(new Error('network unavailable'))
    })
    const { updater, notify } = createHarness(Promise.resolve(update))

    await updater.checkForUpdates({ silent: true })
    await updater.downloadAndInstall()

    expect(updater.phase.value).toBe('available')
    expect(updater.availableVersion.value).toBe('3.0.0')
    expect(notify.failed).toHaveBeenCalledWith('network unavailable')
  })
})
