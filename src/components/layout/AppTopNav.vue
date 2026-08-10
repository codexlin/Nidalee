<template>
  <header class="flex h-12 shrink-0 items-center justify-between gap-4 border-b border-border/40 px-4">
    <nav class="surface-chip flex min-w-0 items-center gap-0.5 overflow-x-auto p-1">
      <button
        v-for="item in navItems"
        :key="item.url"
        type="button"
        :title="item.title"
        :class="navItemClass(isActiveRoute(item.url))"
        @click="router.push(item.url)"
      >
        {{ item.title }}
      </button>

      <template v-if="isDev">
        <div class="mx-0.5 h-4 w-px shrink-0 bg-border/60" />
        <Popover v-model:open="devMenuOpen">
          <PopoverTrigger as-child>
            <button type="button" title="开发工具" :class="navItemClass(isDevRouteActive)">
              开发
              <ChevronDown class="size-3.5 opacity-60" />
            </button>
          </PopoverTrigger>
          <PopoverContent align="start" class="surface-overlay w-44 p-1">
            <button
              v-for="item in devToolsItems"
              :key="item.url"
              type="button"
              class="flex w-full items-center rounded-lg px-3 py-2 text-sm text-foreground hover:bg-accent"
              @click="goDev(item.url)"
            >
              {{ item.title }}
            </button>
          </PopoverContent>
        </Popover>
      </template>
    </nav>

    <div class="flex shrink-0 items-center gap-2">
      <ConnectionStatus />
      <RightToolbars />
    </div>
  </header>
</template>

<script setup lang="ts">
import { ChevronDown } from 'lucide-vue-next'
import { cn } from '@/lib/utils'
import ConnectionStatus from '@/components/layout/ConnectionStatus.vue'
import RightToolbars from '@/components/common/RightToolbars.vue'

const route = useRoute()
const router = useRouter()
const isDev = import.meta.env.DEV
const devMenuOpen = ref(false)

const navItems = [
  { title: '仪表盘', url: '/' },
  { title: '对局分析', url: '/match-analysis' },
  { title: '战绩查询', url: '/match-search' },
  { title: '构建推荐', url: '/opgg' },
  { title: '小助手', url: '/game-helper' },
  { title: '设置', url: '/settings' }
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

const navItemClass = (active: boolean) =>
  cn(
    'inline-flex shrink-0 items-center gap-1 rounded-lg px-3 py-1.5 text-sm font-medium outline-none transition-colors',
    'focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px]',
    active
      ? 'bg-primary/15 text-primary'
      : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
  )

const goDev = (url: string) => {
  devMenuOpen.value = false
  void router.push(url)
}
</script>
