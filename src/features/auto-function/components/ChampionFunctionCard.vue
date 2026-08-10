<template>
  <Card class="transition-all duration-200 hover:shadow-lg border-border bg-card">
    <CardHeader class="space-y-4">
      <div class="flex items-start justify-between">
        <div class="space-y-2 flex-1">
          <div class="flex items-center gap-2">
            <CardTitle class="text-lg font-semibold text-card-foreground">{{ title }}</CardTitle>
            <Tooltip v-if="showRiskWarning" :delay-duration="100">
              <TooltipTrigger>
                <AlertTriangle class="h-5 w-5 text-amber-500 hover:text-amber-600 transition-colors cursor-help" />
              </TooltipTrigger>
              <TooltipContent side="top" class="max-w-xs">
                <p class="text-sm">{{ riskWarningText }}</p>
              </TooltipContent>
            </Tooltip>
          </div>
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
        <Label class="text-sm font-medium text-foreground">选择英雄（可按顺序配置多个）</Label>
        <div v-if="championList && championList.length > 0" class="space-y-2">
          <div class="flex flex-wrap gap-3">
            <div
              v-for="champion in championList"
              :key="champion.id"
              class="flex items-center gap-2 p-2 rounded-lg border border-border bg-muted/30 hover:bg-muted/50 transition-colors group"
            >
              <Avatar class="h-10 w-10 border-2 border-primary ring-1 ring-primary/20">
                <AvatarImage
                  :src="getChampionIconUrlByAlias(champion.alias)"
                  :alt="champion.name"
                  class="object-cover"
                />
                <AvatarFallback class="text-xs font-medium bg-primary/10 text-primary">
                  {{ champion.name.slice(0, 2) }}
                </AvatarFallback>
              </Avatar>
              <span class="font-medium text-foreground">{{ champion.name }}</span>
              <Button variant="ghost" size="icon" @click="handleRemoveChampion(champion.id)" class="ml-1">
                <X class="h-4 w-4 text-destructive" />
              </Button>
            </div>
          </div>
          <div class="flex gap-2 mt-2">
            <Button variant="outline" size="sm" @click="showChampionSelector = true" class="gap-2">
              <Plus class="h-4 w-4" />
              添加英雄
            </Button>
            <Button variant="destructive" size="sm" @click="handleClearChampion" class="gap-2">
              <X class="h-4 w-4" />
              清空
            </Button>
          </div>
        </div>
        <div
          v-else
          class="flex items-center justify-center p-8 rounded-lg border-2 border-dashed border-border bg-muted/20 hover:bg-muted/30 hover:border-muted-foreground/50 transition-all duration-200 group"
        >
          <Button
            variant="outline"
            @click="showChampionSelector = true"
            class="gap-2 group-hover:bg-accent group-hover:text-accent-foreground"
          >
            <Plus class="h-4 w-4" />
            选择英雄
          </Button>
        </div>
      </div>

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

    <Dialog v-model:open="showChampionSelector">
      <DialogContent class="!max-w-[85vw] w-[85vw] !max-h-[85vh] overflow-hidden">
        <DialogHeader class="pb-4">
          <DialogTitle class="text-xl font-semibold">选择英雄</DialogTitle>
          <DialogDescription class="text-muted-foreground"> 选择要自动操作的英雄 </DialogDescription>
        </DialogHeader>
        <div class="overflow-hidden">
          <ChampionSelector @select="handleChampionSelect" />
        </div>
      </DialogContent>
    </Dialog>
  </Card>
</template>

<script setup lang="ts">
import { getChampionIconUrlByAlias } from '@/lib'
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

// const delay = useDebouncedNumberModel(defineModel<number>('delay', { default: 1000 }))
// const { value: debouncedDelay, isPending: isDelayPending, flush: flushDelay } = delay

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

// 双向绑定
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
