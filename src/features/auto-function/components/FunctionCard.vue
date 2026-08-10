<template>
  <Card class="transition-all duration-200 hover:shadow-lg border-border bg-card">
    <CardHeader class="space-y-4">
      <div class="flex items-start justify-between">
        <div class="space-y-2 flex-1">
          <CardTitle class="text-lg font-semibold text-card-foreground">
            {{ title }}
          </CardTitle>
          <CardDescription class="text-sm text-muted-foreground leading-relaxed">
            {{ description }}
          </CardDescription>
        </div>
        <Switch :model-value="enabled" @update:model-value="enabled = $event" class="ml-4" />
      </div>
    </CardHeader>

    <CardContent v-if="enabled" class="space-y-6 pt-0">
      <Separator />

      <div class="space-y-4">
        <div class="flex items-center justify-between">
          <Label class="text-sm font-medium text-foreground">执行延迟</Label>
          <div class="flex items-center gap-2 text-xs text-muted-foreground bg-muted px-2 py-1 rounded">
            <span>{{ formatDelayDisplay(debouncedDelay) }}</span>
            <div v-if="isDelayPending" class="h-2 w-2 bg-orange-500 rounded-full animate-pulse" title="设置保存中..." />
            <div v-else class="h-2 w-2 bg-green-500 rounded-full" title="设置已保存" />
          </div>
        </div>

        <div class="space-y-4">
          <Slider
            :model-value="delayModel"
            @update:model-value="(val: number[] | undefined) => (delayModel = val || [0])"
            :max="10000"
            :min="1000"
            :step="100"
            class="w-full"
          />

          <div class="flex items-center gap-3 bg-muted/50 p-3 rounded-lg">
            <Input
              :model-value="debouncedDelay"
              @update:model-value="
                (val: string | number) => (debouncedDelay = typeof val === 'string' ? parseInt(val) || 0 : val)
              "
              type="number"
              :min="1000"
              :max="10000"
              :step="100"
              class="w-28 text-center text-sm"
            />
            <span class="text-xs text-muted-foreground font-medium">毫秒</span>
          </div>
        </div>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
interface Props {
  title: string
  description: string
}

defineProps<Props>()

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

// Slider 需要数组格式，创建计算属性处理转换
const delayModel = computed({
  get: () => {
    return [debouncedDelay.value]
  },
  set: (value: number[] | undefined) => {
    const newDelay = value?.[0] || 0
    debouncedDelay.value = newDelay
  }
})

const formatDelayDisplay = (value: number) => {
  if (value < 1000) {
    return `${value}ms`
  }
  return `${(value / 1000).toFixed(1)}s`
}

onBeforeUnmount(() => {
  flushDelay()
})
</script>
