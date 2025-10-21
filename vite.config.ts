import { fileURLToPath, URL } from 'node:url'
import { defineConfig, PluginOption } from 'vite'
import vue from '@vitejs/plugin-vue'
import vueJsx from '@vitejs/plugin-vue-jsx'
import tailwindcss from '@tailwindcss/vite'
import Components from 'unplugin-vue-components/vite'
import AutoImport from 'unplugin-auto-import/vite'
import removeConsole from 'vite-plugin-remove-console'
import { visualizer } from 'rollup-plugin-visualizer'
import vueDevTools from 'vite-plugin-vue-devtools'

const isDev = process.env.NODE_ENV === 'development'
const isProd = process.env.NODE_ENV === 'production'
const host = process.env.TAURI_DEV_HOST || false

export default defineConfig({
  plugins: [
    vue({
      // Vue 3 优化配置
      template: {
        compilerOptions: {
          // 生产环境移除注释和空白
          comments: !isProd,
          whitespace: isProd ? 'condense' : 'preserve'
        }
      }
    }),
    vueJsx(),
    tailwindcss(),
    AutoImport({
      imports: ['vue', 'vue-router', 'pinia'],
      dts: 'types/auto-imports.d.ts',
      // 使用 ** 递归扫描所有嵌套模块
      dirs: ['./src/shared/composables/**', './src/shared/stores/**']
    }),
    Components({
      // 组件的有效文件扩展名。
      extensions: ['vue', 'tsx', 'jsx'],
      // 相对路径，用于搜索组件的目录。
      dirs: ['src/components/**', 'src/features/**'],
      // 用于转换目标的过滤器。
      include: [/\.vue$/, /\.vue\?vue/, /\.tsx$/],
      dts: 'types/components.d.ts'
    }),
    // 开发环境插件
    isDev && vueDevTools(),
    // 生产环境插件
    isProd && removeConsole(),
    // 构建分析
    isProd &&
      process.env.ANALYZE === 'true' &&
      visualizer({
        filename: 'dist/stats.html',
        open: true,
        gzipSize: true,
        brotliSize: true
      })
  ].filter(Boolean) as PluginOption[],

  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },

  server: {
    port: 1422,
    strictPort: false,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host: host,
          port: 1423
        }
      : undefined,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  clearScreen: false,
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM == 'windows' ? 'chrome105' : 'safari13',
    outDir: 'dist',
    assetsDir: 'assets',
    sourcemap: false,
    minify: 'esbuild',
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules')) {
            if (id.includes('vue')) return 'vendor_vue'
            if (id.includes('pinia')) return 'vendor_pinia'
            if (id.includes('@tanstack')) return 'vendor_tanstack'
            if (id.includes('tailwind-merge') || id.includes('tw-animate-css') || id.includes('tailwind-scrollbar'))
              return 'vendor_tailwind'
            return 'vendor'
          }
        }
      }
    }
  }
})
