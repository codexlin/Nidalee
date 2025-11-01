<template>
  <ThemedTrendChart :position-data="positionData" />
</template>

<script setup lang="ts">
import ThemedTrendChart from '@/shared/components/charts/ThemedTrendChart.vue'

interface Props {
  positionData: PositionStats
}

defineProps<Props>()
</script>
