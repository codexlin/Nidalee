<script setup lang="ts">
import { Toaster } from 'vue-sonner'
import 'vue-sonner/style.css'
import { appContextKey } from './types'
import ClientDisconnected from './components/common/ClientDisconnected.vue'
import TitleBar from './components/layout/TitleBar.vue'
import AppTopNav from './components/layout/AppTopNav.vue'

const { isDark, checkConnection, isConnected, fetchMatchHistory } = useApp()
const theme = computed(() => (isDark.value ? 'dark' : 'light'))
// 提供方法给子组件使用
provide(appContextKey, {
  checkConnection,
  fetchMatchHistory,
  isConnected,
  isDark
})
const transitions = ['fade', 'slide-fade', 'scale', 'slide-up']
const currentTransition = ref(transitions[0])
const randomTransition = () => {
  const index = Math.floor(Math.random() * transitions.length)
  currentTransition.value = transitions[index]
}
const handleRouteChange = () => {
  randomTransition()
}
const route = useRoute()
const router = useRouter()

watch(
  isConnected,
  () => {
    if (route.name !== 'dashboard') {
      void router.replace({ name: 'dashboard' })
    }
  },
  { immediate: true }
)
</script>

<template>
  <div id="app" class="flex h-screen flex-col overflow-hidden bg-background">
    <Toaster richColors :theme />
    <TooltipProvider :delay-duration="300">
      <TitleBar />
      <template v-if="route.path === '/forbidden'">
        <router-view />
      </template>
      <template v-else>
        <div class="flex min-h-0 flex-1 flex-col overflow-hidden bg-background pt-10">
          <AppTopNav v-if="isConnected" />

          <div
            class="min-h-0 flex-1 overflow-y-auto scroll-smooth scrollbar-thin scrollbar-track-transparent scrollbar-thumb-rounded-full scrollbar-thumb-slate-400/50 dark:scrollbar-thumb-slate-500/50"
          >
            <div
              class="flex min-h-full flex-col bg-background"
              :class="isConnected ? 'gap-6 p-6' : 'p-0'"
            >
              <router-view v-slot="{ Component }">
                <transition :name="currentTransition" mode="out-in" @before-leave="handleRouteChange">
                  <KeepAlive :include="['DashboardView', 'OpggView']">
                    <component :is="isConnected ? Component : ClientDisconnected" />
                  </KeepAlive>
                </transition>
              </router-view>
            </div>
          </div>
        </div>
      </template>
    </TooltipProvider>
  </div>
</template>
