<script setup lang="ts">
import { X } from 'lucide-vue-next'
import OverlayGuidePanel from './components/OverlayGuidePanel.vue'
import { useAugmentOverlay } from './composables/useAugmentOverlay'

const { visible, recommendedAugments, recommendedTrios, championName, winratePending, toggleShortcut, hideSidePanel } =
  useAugmentOverlay()
</script>

<template>
  <div class="h-screen bg-transparent p-1.5">
    <OverlayGuidePanel
      v-if="visible"
      :champion-name="championName"
      :trios="recommendedTrios"
      :augments="recommendedAugments"
      :pending="winratePending"
      :shortcut-label="toggleShortcut"
      @close="hideSidePanel"
    >
      <template #close-icon>
        <X class="size-3.5" />
      </template>
    </OverlayGuidePanel>
  </div>
</template>
