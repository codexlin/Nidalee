<script setup lang="ts">
import { computed } from 'vue'
import type { PerformanceCategory, PerformanceScope, RankedScope } from '@/common/performanceScope'

const scope = defineModel<PerformanceScope>({ required: true })

const category = computed(() => scope.value.category)

function selectCategory(value: PerformanceCategory) {
  scope.value = { ...scope.value, category: value }
}

function selectRankedScope(value: RankedScope) {
  scope.value = { category: 'ranked', rankedScope: value }
}
</script>

<template>
  <div class="flex flex-wrap items-center gap-2" aria-label="战绩分析范围">
    <div class="surface-chip inline-flex items-center gap-1 p-1" role="tablist" aria-label="战绩分类">
      <button
        v-for="item in [
          { value: 'ranked' as const, label: '排位表现' },
          { value: 'other' as const, label: '其他模式' }
        ]"
        :key="item.value"
        type="button"
        role="tab"
        :aria-selected="category === item.value"
        class="rounded-lg px-3 py-1.5 text-sm font-medium outline-none transition-colors focus-visible:ring-3 focus-visible:ring-ring/50"
        :class="
          category === item.value
            ? 'bg-primary/15 text-primary'
            : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
        "
        @click="selectCategory(item.value)"
      >
        {{ item.label }}
      </button>
    </div>

    <div
      v-if="category === 'ranked'"
      class="surface-chip inline-flex items-center gap-1 p-1"
      role="tablist"
      aria-label="排位范围"
    >
      <button
        v-for="item in [
          { value: 'mixed' as const, label: '排位综合' },
          { value: 'solo' as const, label: '单双排' },
          { value: 'flex' as const, label: '灵活组排' }
        ]"
        :key="item.value"
        type="button"
        role="tab"
        :aria-selected="scope.rankedScope === item.value"
        class="rounded-lg px-2.5 py-1.5 text-xs font-medium outline-none transition-colors focus-visible:ring-3 focus-visible:ring-ring/50"
        :class="
          scope.rankedScope === item.value
            ? 'bg-background text-foreground shadow-sm'
            : 'text-muted-foreground hover:text-foreground'
        "
        @click="selectRankedScope(item.value)"
      >
        {{ item.label }}
      </button>
    </div>
  </div>
</template>
