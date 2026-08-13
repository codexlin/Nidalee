<script setup lang="ts">
import HextechAugmentGroups from '@/shared/components/hextech/HextechAugmentGroups.vue'
import HextechTrioList from '@/shared/components/hextech/HextechTrioList.vue'
import type { HextechGuideAugment, HextechGuideTrio } from '@/shared/hextech/guideAugment'

defineProps<{
  championName: string | null
  trios: HextechGuideTrio[]
  augments: HextechGuideAugment[]
  pending: boolean
  shortcutLabel?: string
}>()

defineEmits<{
  close: []
}>()
</script>

<template>
  <div class="flex h-full min-h-0 flex-col overflow-hidden rounded-2xl surface-overlay">
    <header class="flex shrink-0 items-center justify-between gap-2 border-b border-white/10 px-2.5 py-1.5">
      <div data-tauri-drag-region class="min-w-0 flex-1">
        <p class="text-[10px] leading-tight tracking-wide text-muted-foreground">推荐方案</p>
        <h1 class="truncate text-sm font-medium leading-tight text-foreground">
          {{ championName || '当前英雄' }}
        </h1>
      </div>
      <kbd
        v-if="shortcutLabel"
        class="shrink-0 rounded-md bg-white/10 px-1.5 py-0.5 text-[10px] tabular-nums text-muted-foreground"
        :title="`${shortcutLabel} 显示或隐藏`"
      >
        {{ shortcutLabel }}
      </kbd>
      <button
        type="button"
        class="flex size-6 shrink-0 items-center justify-center rounded-lg text-muted-foreground hover:bg-white/10 hover:text-foreground"
        title="关闭"
        aria-label="关闭"
        @click="$emit('close')"
      >
        <slot name="close-icon" />
      </button>
    </header>

    <div class="min-h-0 flex-1 overflow-y-auto px-2.5 py-2 scrollbar-thin scrollbar-track-transparent">
      <section v-if="pending && !trios.length && !augments.length" class="py-8 text-center text-sm text-muted-foreground">
        正在读取构建中心推荐…
      </section>

      <section v-else-if="!trios.length && !augments.length" class="py-8 text-center text-sm text-muted-foreground">
        暂无该英雄推荐
      </section>

      <section v-else class="space-y-2.5">
        <div v-if="trios.length">
          <HextechTrioList :trios="trios" variant="overlay" />
        </div>

        <div v-if="augments.length">
          <h2 class="mb-1.5 text-[10px] font-medium tracking-wide text-muted-foreground">推荐增强</h2>
          <HextechAugmentGroups :augments="augments" variant="overlay" />
        </div>
      </section>
    </div>
  </div>
</template>
