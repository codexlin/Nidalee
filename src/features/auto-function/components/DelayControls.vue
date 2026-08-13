<template>
  <div :class="cn('flex flex-wrap items-center gap-x-3 gap-y-2', props.class)">
    <Label class="shrink-0 text-sm font-medium">延迟</Label>
    <Slider
      :model-value="delayModel"
      :max="10000"
      :min="1000"
      :step="100"
      class="min-w-32 flex-1"
      @update:model-value="(val: number[] | undefined) => (delayModel = val || [0])"
    />
    <div class="flex items-center gap-1.5 tabular-nums">
      <Input
        :model-value="secondsDisplay"
        type="number"
        :min="1"
        :max="10"
        :step="0.1"
        class="h-8 w-16 text-center text-sm"
        @update:model-value="onSecondsInput"
      />
      <span class="text-xs text-muted-foreground">秒</span>
      <span
        class="size-1.5 rounded-full"
        :class="isDelayPending ? 'animate-pulse bg-primary' : 'bg-emerald-500'"
        :title="isDelayPending ? '设置保存中…' : '设置已保存'"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()

const delay = defineModel<number>('delay', { required: true })

const {
  value: debouncedDelay,
  isPending: isDelayPending,
  flush: flushDelay
} = useDebouncedNumberModel(delay, {
  delay: 500,
  min: 1000,
  max: 10000,
  step: 100
})

const delayModel = computed({
  get: () => [debouncedDelay.value],
  set: (val: number[] | undefined) => {
    debouncedDelay.value = val?.[0] || 0
  }
})

const secondsDisplay = computed(() => Number((debouncedDelay.value / 1000).toFixed(1)))

const onSecondsInput = (val: string | number) => {
  const seconds = typeof val === 'string' ? parseFloat(val) : val
  if (!Number.isFinite(seconds)) return
  debouncedDelay.value = Math.round(seconds * 1000)
}

onBeforeUnmount(() => {
  flushDelay()
})
</script>
