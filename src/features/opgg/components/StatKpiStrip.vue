<template>
  <div
    class="ml-auto grid w-full gap-1 sm:w-auto"
    :class="items.length >= 4 ? 'grid-cols-4 sm:min-w-[18rem]' : 'grid-cols-3 sm:min-w-[14rem]'"
  >
    <div v-for="(kpi, index) in items" :key="kpi.label" class="surface-inset rounded-xl px-2 py-1.5 text-center">
      <p class="text-xs leading-none text-muted-foreground">{{ kpi.label }}</p>
      <p
        class="mt-1 font-medium tabular-nums leading-none"
        :class="[
          index === 0 ? 'text-lg' : 'text-sm',
          kpi.tone || (index === 0 ? 'text-sky-600 dark:text-sky-400' : 'text-foreground')
        ]"
      >
        {{ kpi.value }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
export interface StatKpiItem {
  label: string
  value: string
  tone?: string
}

defineProps<{
  items: StatKpiItem[]
}>()
</script>
