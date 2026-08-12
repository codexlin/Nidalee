<template>
  <div class="w-full min-w-0">
    <BuildCenter />
  </div>
</template>

<script setup lang="ts">
import BuildCenter from '@/features/build-center/BuildCenter.vue'

defineOptions({ name: 'BuildCenterView' })
</script>
