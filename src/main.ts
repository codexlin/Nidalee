import { VueQueryPlugin, QueryClient } from '@tanstack/vue-query'
import App from './App.vue'
import router from './router'
import './style.css'
import { isOverlayWindow, markOverlayDocument, overlayRoute } from './shared/utils/overlayWindow'

/** 挂载前只等 Regular；Medium/Bold 挂载后后台预热，缩短首屏空白 */
async function warmHarmonyFonts() {
  if (typeof document === 'undefined' || !document.fonts?.load) return
  try {
    await Promise.race([
      document.fonts.load('400 16px "HarmonyOS Sans SC"'),
      new Promise<void>((resolve) => setTimeout(resolve, 500))
    ])
  } catch {
    // 字体失败不阻塞启动
  }
}

function warmSecondaryHarmonyFonts() {
  if (typeof document === 'undefined' || !document.fonts?.load) return
  void Promise.all([
    document.fonts.load('500 16px "HarmonyOS Sans SC"'),
    document.fonts.load('700 16px "HarmonyOS Sans SC"')
  ]).catch(() => {})
}

async function bootstrap() {
  await warmHarmonyFonts()

  const app = createApp(App)

  // 配置 TanStack Query
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        // 数据保持新鲜的时间（内不重新请求）
        staleTime: 1000 * 60 * 5, // 5 分钟
        // 缓存保留时间（后垃圾回收）
        gcTime: 1000 * 60 * 60 * 24, // 24 小时
        // 窗口聚焦时不自动重新请求
        refetchOnWindowFocus: false,
        // 组件挂载时不自动重新请求（如果数据在 gcTime 内）
        refetchOnMount: false,
        // 重试次数
        retry: 1
      }
    }
  })

  app.use(VueQueryPlugin, { queryClient })
  app.use(stores)
  app.use(router)

  if (isOverlayWindow()) {
    markOverlayDocument()
    const target = overlayRoute()
    if (router.currentRoute.value.path !== target) {
      await router.replace(target)
    }
  }

  app.mount('#app')
  warmSecondaryHarmonyFonts()
}

void bootstrap()
