<script setup lang="ts">
import type { HTMLAttributes } from 'vue'
import { cn } from '@/lib/utils'

const props = withDefaults(
  defineProps<{
    class?: HTMLAttributes['class']
    title?: string
    ariaLabel?: string
    type?: 'button' | 'submit' | 'reset'
    disabled?: boolean
    /** icon：方钮；pill：文字胶囊（如「加载更多」） */
    variant?: 'icon' | 'pill'
  }>(),
  {
    type: 'button',
    variant: 'icon',
    disabled: false
  }
)
</script>

<template>
  <button
    :type="props.type"
    :title="props.title"
    :aria-label="props.ariaLabel ?? props.title"
    :disabled="props.disabled"
    :class="
      cn(
        'surface-float group relative inline-flex cursor-pointer items-center justify-center outline-none',
        'text-muted-foreground hover:text-foreground',
        'focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]',
        '[&_svg]:pointer-events-none [&_svg]:shrink-0',
        'disabled:pointer-events-none disabled:opacity-50',
        props.variant === 'pill' ? 'gap-1.5 px-4 py-2 text-sm font-medium' : 'p-3',
        props.class
      )
    "
  >
    <slot />
  </button>
</template>
