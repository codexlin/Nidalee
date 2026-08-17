<script setup lang="ts">
import { Bell, CheckCircle2, Download, RefreshCw } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover'
import { Progress } from '@/components/ui/progress'
import { Separator } from '@/components/ui/separator'
import { Spinner } from '@/components/ui/spinner'
import { useAppUpdater } from '@/shared/composables/app/useAppUpdater'

const updater = useAppUpdater()
const isOpen = shallowRef(false)

const title = computed(() => {
  if (updater.phase.value === 'checking') return '正在检查更新'
  if (updater.phase.value === 'downloading') return '正在下载更新'
  if (updater.phase.value === 'installing') return '正在安装更新'
  if (updater.availableVersion.value) return `发现新版本 v${updater.availableVersion.value}`
  if (updater.phase.value === 'error') return '更新检查失败'
  return '应用更新'
})

const releaseDate = computed(() => {
  const value = updater.availableDate.value
  if (!value) return null
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return null
  return new Intl.DateTimeFormat('zh-CN', { year: 'numeric', month: 'short', day: 'numeric' }).format(date)
})

const releaseNotes = computed(() => {
  const body = updater.availableNotes.value
  if (!body) return []

  return body
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line) =>
      line
        .replace(/^#{1,6}\s+/, '')
        .replace(/^[-*]\s+/, '• ')
        .replace(/\[([^\]]+)]\([^)]+\)/g, '$1')
    )
})

const statusDescription = computed(() => {
  if (updater.phase.value === 'checking') return '正在连接更新服务…'
  if (updater.phase.value === 'downloading') {
    return updater.progress.value === null ? '正在下载新版本…' : `已下载 ${updater.progress.value}%`
  }
  if (updater.phase.value === 'installing') return '安装完成后应用将自动重新启动。'
  if (updater.phase.value === 'error') return updater.lastError.value || '暂时无法检查更新，请稍后重试。'
  return '当前没有待安装的更新。'
})

function checkForUpdates() {
  void updater.checkForUpdates()
}

function installUpdate() {
  void updater.downloadAndInstall()
}
</script>

<template>
  <Popover v-if="updater.isSupported.value" v-model:open="isOpen">
    <PopoverTrigger as-child>
      <FloatIconButton class="p-2" :title="title" aria-label="应用更新">
        <Bell :size="16" />
        <span
          v-if="updater.availableVersion.value && !updater.isBusy.value"
          class="absolute top-1 right-1 size-2 rounded-full bg-primary ring-2 ring-background"
          aria-hidden="true"
        />
      </FloatIconButton>
    </PopoverTrigger>

    <PopoverContent align="end" side="bottom" class="surface-overlay flex w-90 flex-col gap-3 !border-none p-4">
      <div class="flex items-start justify-between gap-3">
        <div class="flex min-w-0 flex-col gap-1">
          <h3 class="font-medium text-foreground">应用更新</h3>
          <p class="text-xs text-muted-foreground">版本检查与安装</p>
        </div>
        <Badge v-if="updater.availableVersion.value" variant="secondary">新版本</Badge>
      </div>

      <Separator />

      <template v-if="updater.availableVersion.value">
        <div class="flex items-center justify-between gap-3">
          <div class="min-w-0">
            <p class="font-medium text-foreground">Nidalee v{{ updater.availableVersion.value }}</p>
            <p v-if="releaseDate" class="mt-0.5 text-xs text-muted-foreground">发布于 {{ releaseDate }}</p>
          </div>
          <Download class="size-5 shrink-0 text-primary" aria-hidden="true" />
        </div>

        <div v-if="releaseNotes.length" class="max-h-52 overflow-y-auto rounded-lg surface-inset px-3 py-2.5">
          <p
            v-for="(line, index) in releaseNotes"
            :key="`${index}-${line}`"
            class="text-xs leading-5 text-muted-foreground"
          >
            {{ line }}
          </p>
        </div>
        <p v-else class="text-sm text-muted-foreground">新版本已经准备好，可以直接下载并安装。</p>

        <div v-if="updater.phase.value === 'downloading'" class="flex flex-col gap-2">
          <Progress :model-value="updater.progress.value ?? 0" />
          <p class="text-xs text-muted-foreground">{{ statusDescription }}</p>
        </div>

        <Button class="w-full" :disabled="updater.isBusy.value" @click="installUpdate">
          <Spinner v-if="updater.isBusy.value" data-icon="inline-start" />
          <Download v-else data-icon="inline-start" />
          {{ updater.isBusy.value ? statusDescription : '下载并安装' }}
        </Button>
      </template>

      <template v-else>
        <div class="flex items-start gap-3 rounded-lg surface-inset p-3">
          <Spinner v-if="updater.isBusy.value" class="mt-0.5" />
          <CheckCircle2 v-else class="mt-0.5 size-4 shrink-0 text-muted-foreground" aria-hidden="true" />
          <div class="min-w-0">
            <p class="text-sm font-medium text-foreground">
              {{ updater.phase.value === 'error' ? '检查更新失败' : '已启用自动检查' }}
            </p>
            <p class="mt-0.5 break-words text-xs leading-5 text-muted-foreground">{{ statusDescription }}</p>
          </div>
        </div>

        <Button variant="outline" class="w-full" :disabled="updater.isBusy.value" @click="checkForUpdates">
          <Spinner v-if="updater.isBusy.value" data-icon="inline-start" />
          <RefreshCw v-else data-icon="inline-start" />
          {{ updater.isBusy.value ? '正在检查' : '检查更新' }}
        </Button>
      </template>
    </PopoverContent>
  </Popover>
</template>
