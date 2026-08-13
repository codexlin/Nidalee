<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { Keyboard, RotateCcw } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { DEFAULT_OVERLAY_SHORTCUT, eventToAccelerator, formatAccelerator } from '@/shared/utils/accelerator'

const settingsStore = useSettingsStore()
const recording = ref(false)
const display = computed(() => formatAccelerator(settingsStore.augmentOverlayShortcut || DEFAULT_OVERLAY_SHORTCUT))

const stopRecording = () => {
  recording.value = false
}

const startRecording = () => {
  recording.value = true
}

const applyShortcut = async (raw: string) => {
  const formatted = formatAccelerator(raw)
  try {
    const saved = await invoke<string>('set_augment_overlay_shortcut', { shortcut: formatted })
    settingsStore.setAugmentOverlayShortcut(formatAccelerator(saved) || formatted)
    toast.success(`已设置快捷键 ${settingsStore.augmentOverlayShortcut}`)
  } catch (error) {
    toast.error('快捷键设置失败', {
      description: error instanceof Error ? error.message : String(error)
    })
  }
}

const onKeydown = (event: KeyboardEvent) => {
  if (!recording.value) return
  if (event.key === 'Escape') {
    event.preventDefault()
    stopRecording()
    return
  }
  const accel = eventToAccelerator(event)
  if (!accel) return
  event.preventDefault()
  event.stopPropagation()
  stopRecording()
  void applyShortcut(accel)
}

const resetShortcut = () => {
  stopRecording()
  void applyShortcut(DEFAULT_OVERLAY_SHORTCUT)
}

onMounted(() => window.addEventListener('keydown', onKeydown, true))
onUnmounted(() => {
  window.removeEventListener('keydown', onKeydown, true)
  stopRecording()
})
</script>

<template>
  <Card class="gap-0 py-0">
    <CardHeader class="gap-1 px-4 py-3 sm:px-5">
      <CardTitle class="flex items-center gap-2 text-lg font-medium leading-tight">
        <Keyboard class="size-4 text-muted-foreground" />
        快捷键设置
      </CardTitle>
      <p class="mt-0.5 text-xs text-muted-foreground">海克斯大乱斗对局中显示或隐藏推荐侧栏</p>
    </CardHeader>
    <CardContent class="flex flex-col gap-3 px-4 pb-4 sm:px-5">
      <div class="flex items-center justify-between gap-4 rounded-xl surface-inset px-3 py-3">
        <div class="min-w-0">
          <div class="text-sm font-medium">海克斯推荐侧栏</div>
          <div class="mt-0.5 text-xs text-muted-foreground">显示或隐藏推荐。关掉后按快捷键即可再打开。</div>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            class="h-8 min-w-24 font-medium tabular-nums"
            @click="startRecording"
          >
            {{ recording ? '按下快捷键…' : display }}
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            class="h-8 px-2"
            title="恢复默认 Insert"
            aria-label="恢复默认 Insert"
            @click="resetShortcut"
          >
            <RotateCcw data-icon="inline-start" />
          </Button>
        </div>
      </div>
      <p v-if="recording" class="text-xs text-muted-foreground">按下新的快捷键，Esc 取消</p>
    </CardContent>
  </Card>
</template>
