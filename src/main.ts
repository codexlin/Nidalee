import { VueQueryPlugin, QueryClient } from '@tanstack/vue-query'
import App from './App.vue'
import router from './router'
import './style.css'

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

app.mount('#app')
