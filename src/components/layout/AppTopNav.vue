<template>
  <header class="app-top-nav">
    <!-- 左：等宽占位，保证中岛真居中 -->
    <div class="nav-region nav-region-left" aria-hidden="true" />

    <!-- 中：导航岛 -->
    <div class="nav-region nav-region-center">
      <nav class="surface-chip nav-island" aria-label="主导航">
        <button
          v-for="item in navItems"
          :key="item.url"
          type="button"
          :title="item.title"
          :class="navPillClass(isActiveRoute(item.url))"
          @click="router.push(item.url)"
        >
          <component :is="item.icon" class="size-3.5 shrink-0" />
          <span class="nav-label">{{ item.title }}</span>
        </button>
      </nav>
    </div>

    <!-- 右：开发 + 工具（与左等宽，内容靠右） -->
    <div class="nav-region nav-region-right">
      <Popover v-if="isDev" v-model:open="devMenuOpen">
        <PopoverTrigger as-child>
          <button
            type="button"
            title="开发工具"
            class="nav-icon-button"
            :class="{ 'nav-icon-button-active': isDevRouteActive }"
          >
            <Wrench class="size-3.5" />
          </button>
        </PopoverTrigger>
        <PopoverContent align="end" class="surface-overlay w-44 p-1">
          <button
            v-for="item in devToolsItems"
            :key="item.url"
            type="button"
            class="flex w-full items-center rounded-lg px-3 py-2 text-sm text-foreground outline-none transition-colors hover:bg-accent"
            @click="goDev(item.url)"
          >
            {{ item.title }}
          </button>
        </PopoverContent>
      </Popover>

      <RightToolbars />
    </div>
  </header>
</template>

<script setup lang="ts">
import { LayoutDashboard, Search, Settings, Sparkles, Swords, Wrench } from 'lucide-vue-next'
import { cn } from '@/lib/utils'
import RightToolbars from '@/components/common/RightToolbars.vue'

const route = useRoute()
const router = useRouter()
const isDev = import.meta.env.DEV
const devMenuOpen = ref(false)

const navItems = [
  { title: '仪表盘', url: '/', icon: LayoutDashboard },
  { title: '对局分析', url: '/match-analysis', icon: Swords },
  { title: '战绩查询', url: '/match-search', icon: Search },
  { title: '构建中心', url: '/opgg', icon: Sparkles },
  { title: '设置', url: '/settings', icon: Settings }
] as const

const devToolsItems = [
  { title: '大厅测试工具', url: '/lobby-test' },
  { title: '数据收集测试', url: '/data-collection-test' }
] as const

const isActiveRoute = (url: string) => {
  if (url === '/') return route.path === '/'
  return route.path.startsWith(url)
}

const isDevRouteActive = computed(() => devToolsItems.some((item) => route.path.startsWith(item.url)))

const navPillClass = (active: boolean) => cn('nav-pill', active && 'nav-pill-active')

const goDev = (url: string) => {
  devMenuOpen.value = false
  void router.push(url)
}
</script>

<style scoped>
.app-top-nav {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
  align-items: center;
  height: 48px;
  flex-shrink: 0;
  gap: 8px;
  padding: 0 16px;
  background: transparent;
}

.nav-region {
  display: flex;
  align-items: center;
  min-width: 0;
}

.nav-region-left {
  justify-content: flex-start;
}

.nav-region-center {
  justify-content: center;
}

.nav-region-right {
  justify-content: flex-end;
  gap: 6px;
}

.nav-island {
  display: flex;
  align-items: center;
  gap: 2px;
  max-width: min(100%, 44rem);
  overflow-x: auto;
  padding: 4px;
  border-radius: 9999px;
}

.nav-pill {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 12px;
  border-radius: 9999px;
  font-size: 14px;
  font-weight: 500;
  color: var(--muted-foreground);
  white-space: nowrap;
  outline: none;
  transition:
    background-color 150ms ease,
    color 150ms ease;
}

.nav-pill:hover {
  background: color-mix(in oklch, var(--muted) 55%, transparent);
  color: var(--foreground);
}

.nav-pill:focus-visible {
  border: 1px solid var(--ring);
  box-shadow: 0 0 0 3px color-mix(in oklch, var(--ring) 50%, transparent);
}

.nav-pill-active {
  background: color-mix(in oklch, var(--primary) 15%, transparent);
  color: var(--primary);
}

.nav-pill-active:hover {
  background: color-mix(in oklch, var(--primary) 20%, transparent);
  color: var(--primary);
}

.nav-icon-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 9999px;
  color: var(--muted-foreground);
  background: transparent;
  outline: none;
  transition:
    background-color 150ms ease,
    color 150ms ease;
}

.nav-icon-button:hover {
  background: var(--muted);
  color: var(--foreground);
}

.nav-icon-button-active {
  color: var(--primary);
}

@media (max-width: 1100px) {
  .nav-label {
    display: none;
  }

  .nav-pill {
    padding: 0 10px;
  }
}
</style>
