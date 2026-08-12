<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = defineProps<{
  class?: HTMLAttributes['class']
}>()
</script>

<template>
  <div data-slot="card" :class="cn('surface-raised text-card-foreground flex flex-col gap-6 py-6', props.class)">
    <slot />
  </div>
</template>
