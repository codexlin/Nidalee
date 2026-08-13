<script setup lang="ts">
import { X } from 'lucide-vue-next'
import AugmentOverlayCard from './components/AugmentOverlayCard.vue'
import { useAugmentOverlay } from './composables/useAugmentOverlay'

const { visible, augments, winratePending, hide } = useAugmentOverlay()
</script>

<template>
  <div class="flex h-screen items-start justify-center bg-transparent px-2 pt-1">
    <div v-if="visible && augments.length" class="relative flex w-full max-w-[760px] items-stretch gap-2">
      <AugmentOverlayCard
        v-for="(augment, index) in augments"
        :key="augment.id ?? `slot-${index}`"
        :augment="augment"
        :pending="winratePending"
      />
      <button
        type="button"
        class="absolute -right-1 -top-1 flex size-6 items-center justify-center rounded-2xl surface-float text-muted-foreground hover:text-foreground"
        title="关闭"
        aria-label="关闭"
        @click="hide"
      >
        <X class="size-3.5" />
      </button>
    </div>
  </div>
</template>
