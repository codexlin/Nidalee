<template>
  <Sidebar variant="inset" class="top-8 h-auto">
    <SidebarHeader>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton size="lg" asChild>
            <router-link to="/">
              <div class="flex items-center gap-3 py-2 select-none">
                <div
                  class="relative isolate overflow-hidden rounded-xl p-[1px] bg-gradient-to-br from-white/70 to-black/10"
                >
                  <img
                    src="@/assets/logo.png"
                    class="w-10 h-10 rounded-[10px] bg-white shadow-[inset_0_1px_2px_rgba(0,0,0,0.06)]"
                  />
                  <div
                    class="pointer-events-none absolute inset-0 bg-[radial-gradient(ellipse_at_top_left,rgba(255,255,255,0.6),transparent_55%)]"
                  />
                </div>
                <div class="flex flex-col justify-center min-w-0">
                  <div
                    class="font-extrabold text-xl leading-tight tracking-wide truncate bg-gradient-to-r bg-clip-text text-transparent from-primary to-purple-600"
                  >
                    <RadiantText class="transition ease-out" :duration="5">
                      <span>Nidalee~</span>
                    </RadiantText>
                  </div>
                </div>
              </div>
            </router-link>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarHeader>

    <SidebarContent>
      <SidebarGroup>
        <SidebarGroupLabel>应用功能</SidebarGroupLabel>
        <SidebarMenu>
          <SidebarMenuItem v-for="item in menuItems" :key="item.title">
            <SidebarMenuButton class="text-md" asChild :tooltip="item.title" :is-active="isActiveRoute(item.url)">
              <router-link :to="item.url">
                <component
                  :is="item.icon"
                  :size="18"
                  :stroke-width="2"
                  class="shrink-0"
                  :class="{ 'text-primary': isActiveRoute(item.url) }"
                />
                <span>{{ item.title }}</span>
              </router-link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroup>
    </SidebarContent>

    <SidebarFooter>
      <div class="px-2">
        <GitHubStarButtonBeautiful />
      </div>

      <div class="px-2 text-xs text-muted-foreground select-none">软件版本 {{ `v${appVersion}` || '-' }}</div>

      <div class="px-2 text-xs text-muted-foreground select-none">游戏版本 {{ `v${lolGameVersion}` || '-' }}</div>
    </SidebarFooter>

    <SidebarRail />
  </Sidebar>
</template>

<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app'
import { Radar, BarChart3, Settings, Sparkles, Swords, Trophy, TestTube } from 'lucide-vue-next'
const route = useRoute()

const appVersion = ref<string>('')
onMounted(async () => {
  try {
    appVersion.value = await getVersion()
  } catch {
    appVersion.value = ''
  }
})

// 游戏版本：从 dataStore 读取（由初始化逻辑 setGameVersion）
const dataStore = useDataStore()
const lolGameVersion = computed(() => dataStore.gameVersion)

const menuItems = [
  {
    title: '个人仪表盘',
    url: '/',
    icon: Trophy
  },
  {
    title: '游戏小助手',
    url: '/game-helper',
    icon: Sparkles
  },
  {
    title: '战绩查询器',
    url: '/match-search',
    icon: Radar
  },
  {
    title: '对局分析报',
    url: '/match-analysis',
    icon: Swords
  },
  {
    title: 'OP.GG查询',
    url: '/opgg',
    icon: BarChart3
  },
  {
    title: '客户端设置',
    url: '/settings',
    icon: Settings
  },
  {
    title: '数据收集测试',
    url: '/data-collection-test',
    icon: TestTube
  }
]

const isActiveRoute = (url: string) => {
  if (url === '/') {
    return route.path === '/'
  }
  return route.path.startsWith(url)
}
</script>
