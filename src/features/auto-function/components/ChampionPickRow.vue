<template>
  <div class="space-y-2">
    <div class="flex items-center justify-between gap-2">
      <Label class="text-sm font-medium">英雄顺序</Label>
      <span v-if="champions.length > 1" class="text-xs text-muted-foreground">按住拖动调整优先级</span>
    </div>
    <div v-if="champions.length" class="space-y-2">
      <div ref="listEl" class="flex flex-wrap gap-2">
        <div
          v-for="(champion, index) in champions"
          :key="champion.id"
          data-champ-chip
          :data-index="index"
          class="champ-chip flex select-none items-center gap-2 rounded-xl surface-inset px-2 py-1.5"
          :class="
            dragIndex === index
              ? 'cursor-grabbing opacity-60 ring-1 ring-primary/50'
              : champions.length > 1
                ? 'cursor-grab'
                : ''
          "
          @pointerdown="onPointerDown(index, $event)"
        >
          <span class="w-3.5 text-center text-xs tabular-nums text-muted-foreground">{{ index + 1 }}</span>
          <GripVertical class="size-3.5 shrink-0 text-muted-foreground/70" />
          <Avatar class="pointer-events-none size-8 ring-1 ring-border/50">
            <AvatarImage
              :src="getChampionIconUrl(champion.id)"
              :alt="champion.name"
              class="pointer-events-none object-cover"
              draggable="false"
            />
            <AvatarFallback class="bg-muted text-xs">{{ champion.name.slice(0, 2) }}</AvatarFallback>
          </Avatar>
          <span class="text-sm font-medium">{{ champion.name }}</span>
          <button
            type="button"
            data-no-drag
            class="rounded-lg p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-destructive"
            :title="`移除 ${champion.name}`"
            @click="emit('remove', champion.id)"
            @pointerdown.stop
          >
            <X class="size-3.5" />
          </button>
        </div>
      </div>
      <div class="flex flex-wrap gap-2">
        <Button variant="outline" size="sm" class="h-8 gap-1.5" @click="open = true">
          <Plus class="size-3.5" />
          添加
        </Button>
        <Button variant="outline" size="sm" class="h-8 gap-1.5 text-destructive" @click="emit('clear')">
          <X class="size-3.5" />
          清空
        </Button>
      </div>
    </div>
    <button
      v-else
      type="button"
      class="flex w-full items-center justify-center gap-2 rounded-xl border border-dashed border-border/70 px-3 py-6 text-sm text-muted-foreground transition-colors hover:bg-muted/40 hover:text-foreground"
      @click="open = true"
    >
      <Plus class="size-4" />
      选择英雄
    </button>

    <Dialog v-model:open="open">
      <DialogContent class="!max-h-[85vh] w-[85vw] max-w-[85vw]! overflow-hidden">
        <DialogHeader class="pb-4">
          <DialogTitle class="text-lg font-medium">选择英雄</DialogTitle>
          <DialogDescription>按使用顺序添加，可拖动调整优先级</DialogDescription>
        </DialogHeader>
        <div class="overflow-hidden">
          <ChampionSelector @select="onSelect" />
        </div>
      </DialogContent>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { getChampionIconUrl } from '@/lib'
import { GripVertical, Plus, X } from 'lucide-vue-next'

const props = defineProps<{
  champions: ChampionInfo[]
}>()

const emit = defineEmits<{
  add: [champion: ChampionInfo]
  remove: [championId: number]
  clear: []
  reorder: [from: number, to: number]
}>()

const open = ref(false)
const listEl = ref<HTMLElement | null>(null)
const dragIndex = ref<number | null>(null)
const pointerId = ref<number | null>(null)

const onSelect = (champion: ChampionInfo) => {
  emit('add', champion)
  open.value = false
}

const chipIndexAtPoint = (x: number, y: number): number | null => {
  const el = document.elementFromPoint(x, y)
  const chip = el?.closest('[data-champ-chip]') as HTMLElement | null
  if (!chip || !listEl.value?.contains(chip)) return null
  const index = Number(chip.dataset.index)
  return Number.isFinite(index) ? index : null
}

const onPointerDown = (index: number, event: PointerEvent) => {
  if (props.champions.length < 2) return
  if ((event.target as HTMLElement | null)?.closest('[data-no-drag]')) return
  if (event.button !== 0) return

  dragIndex.value = index
  pointerId.value = event.pointerId
  ;(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId)
  window.addEventListener('pointermove', onPointerMove)
  window.addEventListener('pointerup', onPointerUp)
  window.addEventListener('pointercancel', onPointerUp)
}

const onPointerMove = (event: PointerEvent) => {
  if (dragIndex.value === null) return
  if (pointerId.value !== null && event.pointerId !== pointerId.value) return

  const over = chipIndexAtPoint(event.clientX, event.clientY)
  if (over === null || over === dragIndex.value) return

  emit('reorder', dragIndex.value, over)
  dragIndex.value = over
}

const onPointerUp = () => {
  dragIndex.value = null
  pointerId.value = null
  window.removeEventListener('pointermove', onPointerMove)
  window.removeEventListener('pointerup', onPointerUp)
  window.removeEventListener('pointercancel', onPointerUp)
}

onBeforeUnmount(() => {
  onPointerUp()
})
</script>

<style scoped>
.champ-chip {
  touch-action: none;
}

.champ-chip :deep(img) {
  -webkit-user-drag: none;
  user-select: none;
}
</style>
