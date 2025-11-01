<template>
  <ThemedRadarChart :position-data="positionData" />
</template>

<script setup lang="ts">
import ThemedRadarChart from '@/shared/components/charts/ThemedRadarChart.vue'

interface Props {
  positionData: PositionStats
  allPositions?: PositionStats[]
}

defineProps<Props>()
</script>
