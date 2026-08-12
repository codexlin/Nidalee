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
      <!-- 首页 -->
      <SidebarGroup>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton class="text-md" asChild tooltip="个人仪表盘" :is-active="isActiveRoute('/')">
              <router-link to="/">
                <Trophy :size="18" :stroke-width="2" class="shrink-0" :class="{ 'text-primary': isActiveRoute('/') }" />
                <span>个人仪表盘</span>
              </router-link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroup>

      <!-- 数据分析 -->
      <SidebarGroup>
        <SidebarGroupLabel class="bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
          <span class="flex items-center gap-2">
            <BarChart3 :size="14" />
            数据分析
          </span>
        </SidebarGroupLabel>
        <SidebarMenu>
          <SidebarMenuItem v-for="item in analysisItems" :key="item.title">
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

      <!-- 游戏辅助 -->
      <SidebarGroup>
        <SidebarGroupLabel class="bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
          <span class="flex items-center gap-2">
            <Gamepad2 :size="14" />
            游戏辅助
          </span>
        </SidebarGroupLabel>
        <SidebarMenu>
          <SidebarMenuItem v-for="item in gameAssistItems" :key="item.title">
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

      <!-- 设置 -->
      <SidebarGroup>
        <SidebarGroupLabel class="bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
          <span class="flex items-center gap-2">
            <Settings :size="14" />
            设置
          </span>
        </SidebarGroupLabel>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton class="text-md" asChild tooltip="客户端设置" :is-active="isActiveRoute('/settings')">
              <router-link to="/settings">
                <Settings
                  :size="18"
                  :stroke-width="2"
                  class="shrink-0"
                  :class="{ 'text-primary': isActiveRoute('/settings') }"
                />
                <span>客户端设置</span>
              </router-link>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarGroup>

      <!-- 开发测试 (可折叠，仅开发模式) -->
      <SidebarGroup v-if="isDev">
        <SidebarGroupLabel class="cursor-pointer hover:bg-muted/50 transition-colors" @click="toggleDevTools">
          <span class="flex items-center gap-2">
            <Wrench :size="14" />
            开发测试
            <ChevronDown
              :size="14"
              class="ml-auto transition-transform duration-200"
              :class="{ 'rotate-180': devToolsExpanded }"
            />
          </span>
        </SidebarGroupLabel>
        <SidebarMenu v-show="devToolsExpanded">
          <SidebarMenuItem v-for="item in devToolsItems" :key="item.title">
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
import { ref, computed } from 'vue'
import { getVersion } from '@tauri-apps/api/app'
import {
  Radar,
  BarChart3,
  Settings,
  Swords,
  Trophy,
  TestTube,
  MessageSquare,
  Gamepad2,
  Wrench,
  ChevronDown
} from 'lucide-vue-next'
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

// 开发模式检测
const isDev = import.meta.env.DEV

// 开发测试展开状态（默认收起）
const devToolsExpanded = ref(false)
const toggleDevTools = () => {
  devToolsExpanded.value = !devToolsExpanded.value
}

// 数据分析
const analysisItems = [
  {
    title: '战绩查询器',
    url: '/match-search',
    icon: Radar
  },
  {
    title: '对局分析报',
    url: '/match-analysis',
    icon: Swords
  }
]

// 游戏辅助
const gameAssistItems = [
  {
    title: '构建中心',
    url: '/opgg',
    icon: BarChart3
  }
]

// 开发测试
const devToolsItems = [
  {
    title: '大厅测试工具',
    url: '/lobby-test',
    icon: MessageSquare
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
