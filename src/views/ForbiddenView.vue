<template>
  <div
    class="fixed inset-0 flex flex-col items-center justify-center bg-gradient-to-br from-slate-100 to-slate-300 dark:from-slate-900 dark:to-slate-800"
  >
    <div
      class="flex flex-col items-center justify-center p-10 bg-white/80 dark:bg-slate-900/80 rounded-2xl shadow-2xl border border-slate-200 dark:border-slate-700 animate-fade-in"
    >
      <svg
        class="w-24 h-24 text-red-500 mb-6 animate-bounce"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        viewBox="0 0 48 48"
      >
        <circle cx="24" cy="24" r="22" stroke="currentColor" stroke-width="3" fill="none" />
        <path d="M24 14v12" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
        <circle cx="24" cy="34" r="2.5" fill="currentColor" />
      </svg>
      <h1 class="text-5xl font-bold text-red-600 dark:text-red-400 mb-4 drop-shadow-lg">403</h1>
      <p class="text-xl text-slate-700 dark:text-slate-200 mb-2 font-semibold">禁止访问</p>
      <p class="text-base text-slate-500 dark:text-slate-400 mb-6 text-center">
        您无权访问本应用。<br />
        如有疑问请联系管理员。
      </p>
      <button
        class="mt-2 px-6 py-2 rounded-lg bg-red-500 text-white font-bold shadow hover:bg-red-600 transition"
        @click="handleContact"
      >
        联系客服
      </button>
    </div>
    <ParticlesBg
      class="absolute inset-0 h-full pointer-events-none"
      :quantity="80"
      :ease="100"
      :color="isDark ? '#FFF' : '#000'"
      :staticity="10"
      refresh
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useColorMode } from '@vueuse/core'

const isDark = computed(() => useColorMode().value === 'dark')

function handleContact() {
  window.open('https://codexlin.com', '_blank')
}
</script>

<style scoped>
@keyframes fade-in {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}
.animate-fade-in {
  animation: fade-in 0.7s cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
