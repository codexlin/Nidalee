<script setup lang="ts">
import { Search, X } from 'lucide-vue-next'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card'
import { Progress } from '@/components/ui/progress'
import { Spinner } from '@/components/ui/spinner'

interface Props {
  estimatedSeconds: number
  elapsedSeconds: number
  progress: number
  cancelling?: boolean
}

defineProps<Props>()

defineEmits<{
  cancel: []
}>()

function formatDuration(value: number) {
  const seconds = Math.max(0, Math.floor(value))
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
}
</script>

<template>
  <Card class="surface-raised w-full max-w-xl gap-0 overflow-hidden py-0">
    <CardHeader class="flex-row items-center gap-4 border-b border-border/60 px-5 py-5">
      <div class="surface-inset relative flex size-12 shrink-0 items-center justify-center rounded-xl">
        <Search class="size-5 text-primary" />
        <span class="absolute right-2 top-2 size-1.5 rounded-full bg-primary" />
      </div>

      <div class="min-w-0 flex-1">
        <CardTitle class="text-base">正在寻找对局</CardTitle>
        <CardDescription class="mt-1">正在匹配实力相近的玩家</CardDescription>
      </div>

      <Badge variant="secondary" class="gap-1.5">
        <span class="size-1.5 rounded-full bg-primary motion-safe:animate-pulse" />
        匹配中
      </Badge>
    </CardHeader>

    <CardContent class="flex flex-col gap-5 px-5 py-5">
      <div class="grid grid-cols-2 gap-3">
        <div class="surface-inset flex flex-col gap-1 rounded-xl px-4 py-3">
          <span class="text-xs text-muted-foreground">已等待</span>
          <span class="text-2xl font-bold tabular-nums text-foreground">
            {{ formatDuration(elapsedSeconds) }}
          </span>
        </div>
        <div class="surface-inset flex flex-col gap-1 rounded-xl px-4 py-3">
          <span class="text-xs text-muted-foreground">预计用时</span>
          <span class="text-2xl font-bold tabular-nums text-foreground">
            {{ estimatedSeconds > 0 ? formatDuration(estimatedSeconds) : '计算中' }}
          </span>
        </div>
      </div>

      <div class="flex flex-col gap-2">
        <div class="flex items-center justify-between text-xs text-muted-foreground">
          <span>匹配进度</span>
          <span>预计时间仅供参考</span>
        </div>
        <Progress :model-value="progress" class="h-1.5" />
      </div>
    </CardContent>

    <CardFooter class="border-t border-border/60 px-5 py-4">
      <Button variant="outline" class="w-full" :disabled="cancelling" @click="$emit('cancel')">
        <Spinner v-if="cancelling" data-icon="inline-start" />
        <X v-else data-icon="inline-start" />
        {{ cancelling ? '正在取消' : '取消匹配' }}
      </Button>
    </CardFooter>
  </Card>
</template>
