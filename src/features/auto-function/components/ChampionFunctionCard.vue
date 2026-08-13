<template>
  <Card class="gap-0 py-0">
    <CardHeader class="gap-3 px-4 py-4 sm:px-5">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0 space-y-0.5">
          <div class="flex items-center gap-2">
            <CardTitle class="text-lg font-medium leading-tight">{{ title }}</CardTitle>
            <Tooltip v-if="showRiskWarning" :delay-duration="100">
              <TooltipTrigger>
                <AlertTriangle class="size-4 shrink-0 text-amber-500" />
              </TooltipTrigger>
              <TooltipContent side="top" class="max-w-xs">
                <p class="text-sm">{{ riskWarningText }}</p>
              </TooltipContent>
            </Tooltip>
          </div>
          <CardDescription class="text-xs text-muted-foreground">{{ description }}</CardDescription>
        </div>
        <Switch :model-value="enabled" class="mt-0.5 shrink-0" @update:model-value="enabled = $event" />
      </div>
    </CardHeader>

    <CardContent v-if="enabled" class="space-y-4 border-t border-border/50 px-4 py-4 sm:px-5">
      <div class="space-y-2">
        <Label class="text-sm font-medium">选择英雄（可按顺序配置多个）</Label>
        <div v-if="championList?.length" class="space-y-2">
          <div class="flex flex-wrap gap-2">
            <div
              v-for="champion in championList"
              :key="champion.id"
              class="flex items-center gap-2 rounded-xl surface-inset px-2 py-1.5"
            >
              <Avatar class="size-8 ring-1 ring-border/50">
                <AvatarImage :src="getChampionIconUrl(champion.id)" :alt="champion.name" class="object-cover" />
                <AvatarFallback class="bg-muted text-xs">{{ champion.name.slice(0, 2) }}</AvatarFallback>
              </Avatar>
              <span class="text-sm font-medium">{{ champion.name }}</span>
              <button
                type="button"
                class="rounded-lg p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-destructive"
                :title="`移除 ${champion.name}`"
                @click="handleRemoveChampion(champion.id)"
              >
                <X class="size-3.5" />
              </button>
            </div>
          </div>
          <div class="flex flex-wrap gap-2">
            <Button variant="outline" size="sm" class="h-8 gap-1.5" @click="showChampionSelector = true">
              <Plus class="size-3.5" />
              添加英雄
            </Button>
            <Button variant="outline" size="sm" class="h-8 gap-1.5 text-destructive" @click="handleClearChampion">
              <X class="size-3.5" />
              清空
            </Button>
          </div>
        </div>
        <button
          v-else
          type="button"
          class="flex w-full items-center justify-center gap-2 rounded-xl border border-dashed border-border/70 px-4 py-8 text-sm text-muted-foreground transition-colors hover:bg-muted/40 hover:text-foreground"
          @click="showChampionSelector = true"
        >
          <Plus class="size-4" />
          选择英雄
        </button>
      </div>

      <div class="space-y-3">
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
      </div>
    </CardContent>

    <Dialog v-model:open="showChampionSelector">
      <DialogContent class="!max-h-[85vh] w-[85vw] max-w-[85vw]! overflow-hidden">
        <DialogHeader class="pb-4">
          <DialogTitle class="text-lg font-medium">选择英雄</DialogTitle>
          <DialogDescription>选择要自动操作的英雄</DialogDescription>
        </DialogHeader>
        <div class="overflow-hidden">
          <ChampionSelector @select="handleChampionSelect" />
        </div>
      </DialogContent>
    </Dialog>
  </Card>
</template>

<script setup lang="ts">
import { getChampionIconUrl } from '@/lib'
import { AlertTriangle, Plus, X } from 'lucide-vue-next'

defineProps<{
  title: string
  description: string
  championList: ChampionInfo[]
  showRiskWarning?: boolean
  riskWarningText?: string
}>()

const emit = defineEmits<{
  'champion-add': [champion: ChampionInfo]
  'champion-remove': [championId: number]
  'champion-reorder': [from: number, to: number]
  'champion-clear': []
}>()

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

const enabled = defineModel<boolean>('enabled', { default: false })
const showChampionSelector = ref(false)

const handleChampionSelect = (champion: ChampionInfo) => {
  emit('champion-add', champion)
  showChampionSelector.value = false
}

const handleRemoveChampion = (championId: number) => {
  emit('champion-remove', championId)
}

const handleClearChampion = () => {
  emit('champion-clear')
}
</script>
