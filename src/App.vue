<script setup lang="ts">
import MainWindow from './MainWindow.vue'
import { isOverlayWindow } from './shared/utils/overlayWindow'

const overlay = isOverlayWindow()
</script>

<template>
  <div v-if="overlay" id="app" class="h-screen overflow-hidden bg-transparent">
    <router-view />
  </div>
  <MainWindow v-else />
</template>
