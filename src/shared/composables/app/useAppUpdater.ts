import { isTauri } from '@tauri-apps/api/core'
import { relaunch } from '@tauri-apps/plugin-process'
import { check, type DownloadEvent, type Update } from '@tauri-apps/plugin-updater'
import { computed, readonly, ref, shallowRef } from 'vue'
import { toast } from 'vue-sonner'

export type AppUpdaterPhase = 'idle' | 'checking' | 'available' | 'downloading' | 'installing' | 'error'

interface CheckForUpdatesOptions {
  silent?: boolean
}

interface AppUpdaterNotifier {
  current(): void
  installed(version: string): void
  failed(message: string): void
}

interface AppUpdaterDependencies {
  isTauri(): boolean
  isDevelopment(): boolean
  check: typeof check
  relaunch: typeof relaunch
  notify: AppUpdaterNotifier
}

const defaultDependencies: AppUpdaterDependencies = {
  isTauri,
  isDevelopment: () => import.meta.env.DEV,
  check,
  relaunch,
  notify: {
    current() {
      toast.success('当前已是最新版本')
    },
    installed(version) {
      toast.success(`Nidalee v${version} 已安装`, {
        description: '应用即将重新启动。'
      })
    },
    failed(message) {
      toast.error('更新失败', { description: message })
    }
  }
}

const errorMessage = (error: unknown) => (error instanceof Error ? error.message : String(error))

export function createAppUpdater(dependencies: AppUpdaterDependencies = defaultDependencies) {
  const phase = ref<AppUpdaterPhase>('idle')
  const availableUpdate = shallowRef<Update | null>(null)
  const downloadedBytes = ref(0)
  const contentLength = ref<number | null>(null)
  const lastError = ref<string | null>(null)

  let checkFlight: Promise<boolean> | null = null
  let installFlight: Promise<void> | null = null

  const isSupported = computed(() => dependencies.isTauri() && !dependencies.isDevelopment())
  const availableVersion = computed(() => availableUpdate.value?.version ?? null)
  const availableNotes = computed(() => availableUpdate.value?.body?.trim() || null)
  const availableDate = computed(() => availableUpdate.value?.date ?? null)
  const isBusy = computed(() => ['checking', 'downloading', 'installing'].includes(phase.value))
  const progress = computed(() => {
    if (!contentLength.value || contentLength.value <= 0) return null
    return Math.min(100, Math.round((downloadedBytes.value / contentLength.value) * 100))
  })

  const checkForUpdates = (options: CheckForUpdatesOptions = {}) => {
    if (!isSupported.value) return Promise.resolve(false)
    if (checkFlight) return checkFlight

    checkFlight = (async () => {
      phase.value = 'checking'
      lastError.value = null

      try {
        const nextUpdate = await dependencies.check({ timeout: 20000 })

        if (!nextUpdate) {
          if (availableUpdate.value) await availableUpdate.value.close()
          availableUpdate.value = null
          phase.value = 'idle'
          if (!options.silent) dependencies.notify.current()
          return false
        }

        if (availableUpdate.value && availableUpdate.value.rid !== nextUpdate.rid) {
          await availableUpdate.value.close()
        }

        availableUpdate.value = nextUpdate
        phase.value = 'available'
        return true
      } catch (error) {
        const message = errorMessage(error)
        lastError.value = message
        phase.value = availableUpdate.value ? 'available' : 'error'
        if (!options.silent) dependencies.notify.failed(message)
        return false
      } finally {
        checkFlight = null
      }
    })()

    return checkFlight
  }

  const handleDownloadEvent = (event: DownloadEvent) => {
    if (event.event === 'Started') {
      downloadedBytes.value = 0
      contentLength.value = event.data.contentLength ?? null
      return
    }

    if (event.event === 'Progress') {
      downloadedBytes.value += event.data.chunkLength
      return
    }

    phase.value = 'installing'
  }

  const downloadAndInstall = () => {
    if (installFlight) return installFlight

    installFlight = (async () => {
      let update = availableUpdate.value
      if (!update) {
        const found = await checkForUpdates()
        if (!found) return
        update = availableUpdate.value
      }
      if (!update) return

      phase.value = 'downloading'
      downloadedBytes.value = 0
      contentLength.value = null
      lastError.value = null

      try {
        await update.downloadAndInstall(handleDownloadEvent, { timeout: 300000 })
        phase.value = 'installing'
        dependencies.notify.installed(update.version)
        await dependencies.relaunch()
      } catch (error) {
        const message = errorMessage(error)
        lastError.value = message
        phase.value = 'available'
        dependencies.notify.failed(message)
      } finally {
        installFlight = null
      }
    })()

    return installFlight
  }

  return {
    phase: readonly(phase),
    availableVersion,
    availableNotes,
    availableDate,
    downloadedBytes: readonly(downloadedBytes),
    contentLength: readonly(contentLength),
    lastError: readonly(lastError),
    isSupported,
    isBusy,
    progress,
    checkForUpdates,
    downloadAndInstall
  }
}

const appUpdater = createAppUpdater()

export function useAppUpdater() {
  return appUpdater
}
