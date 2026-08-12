<template>
  <section class="border-b border-border/60 px-4 py-3 sm:px-5">
    <div class="mb-2 flex items-end justify-between gap-2">
      <h3 class="text-lg font-medium leading-tight">{{ title }}</h3>
      <div
        v-if="showRates"
        class="hidden items-center gap-3 text-sm text-muted-foreground sm:flex"
        :class="actionSlot && 'pr-16'"
      >
        <span class="w-12 text-right">胜率</span>
        <span class="w-12 text-right">选取</span>
        <span class="w-14 text-right">场次</span>
      </div>
      <slot name="header-extra" />
    </div>
    <slot />
  </section>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    title: string
    showRates?: boolean
    /** 右侧有操作按钮时给列头留位 */
    actionSlot?: boolean
  }>(),
  { showRates: false, actionSlot: false }
)
</script>
