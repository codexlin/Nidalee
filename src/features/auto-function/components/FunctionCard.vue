<template>
  <Card class="gap-0 py-0">
    <CardHeader class="gap-3 px-4 py-4 sm:px-5">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0 space-y-0.5">
          <CardTitle class="text-lg font-medium leading-tight">{{ title }}</CardTitle>
          <CardDescription class="text-xs text-muted-foreground">{{ description }}</CardDescription>
        </div>
        <Switch :model-value="enabled" class="mt-0.5 shrink-0" @update:model-value="enabled = $event" />
      </div>
    </CardHeader>

    <CardContent v-if="enabled" class="space-y-4 border-t border-border/50 px-4 py-4 sm:px-5">
      <div class="flex items-center justify-between gap-3">
        <Label class="text-sm font-medium">执行延迟</Label>
        <div class="flex items-center gap-1.5 text-xs text-muted-foreground">
          <span class="tabular-nums">{{ formatDelayDisplay(debouncedDelay) }}</span>
          <span
            class="size-1.5 rounded-full"
            :class="isDelayPending ? 'animate-pulse bg-primary' : 'bg-emerald-500'"
            :title="isDelayPending ? '设置保存中…' : '设置已保存'"
          />
        </div>
      </div>

      <Slider
        :model-value="delayModel"
        :max="10000"
        :min="1000"
        :step="100"
        class="w-full"
        @update:model-value="(val: number[] | undefined) => (delayModel = val || [0])"
      />

      <div class="flex items-center gap-2 rounded-xl surface-inset px-3 py-2">
        <Input
          :model-value="debouncedDelay"
          type="number"
          :min="1000"
          :max="10000"
          :step="100"
          class="h-9 w-28 text-center text-sm tabular-nums"
          @update:model-value="
            (val: string | number) => (debouncedDelay = typeof val === 'string' ? parseInt(val) || 0 : val)
          "
        />
        <span class="text-xs text-muted-foreground">毫秒</span>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  description: string
}>()

const enabled = defineModel<boolean>('enabled', { default: false })
const delay = defineModel<number>('delay', { default: 1000 })

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

function formatDelayDisplay(val: number) {
  if (val >= 1000) return (val / 1000).toFixed(1) + ' 秒'
  return val + ' 毫秒'
}

onBeforeUnmount(() => {
  flushDelay()
})
</script>
