import { colors, radiusOptions, styles } from '@/lib/theme'
import {
  normalizePerformanceCategory,
  normalizeRankedScope,
  type PerformanceCategory,
  type PerformanceScope,
  type RankedScope
} from '@/common/performanceScope'
import { DEFAULT_OVERLAY_SHORTCUT } from '@/shared/utils/accelerator'

export const useSettingsStore = defineStore(
  'settings',
  () => {
    // 主题设置
    const selectedColor = ref<string>('zinc')
    const selectedRadius = ref(0.5)
    const selectedStyle = ref('new-york')
    const isDark = ref(false)

    // 应用设置
    const autoStart = ref(false)
    const minimizeToTray = ref(true)
    const showNotifications = ref(true)
    const language = ref('zh-CN')

    // 游戏设置
    const careerBackground = ref<string>('')
    const autoRefreshData = ref(true)
    const refreshInterval = ref(30000) // 30秒
    // 仪表板与战绩查询共享同一分析范围；场数固定为最近 20 场有效样本。
    const lastPerformanceCategory = ref<PerformanceCategory>('ranked')
    const lastRankedScope = ref<RankedScope>('mixed')
    const augmentOverlayShortcut = ref(DEFAULT_OVERLAY_SHORTCUT)

    // 计算属性
    const themeConfig = computed(() => ({
      color: selectedColor.value,
      radius: selectedRadius.value,
      style: selectedStyle.value,
      isDark: isDark.value
    }))

    const performanceScope = computed<PerformanceScope>(() => ({
      category: lastPerformanceCategory.value,
      rankedScope: lastRankedScope.value
    }))

    // 主题相关class同步
    function setThemeClass(theme: string, isDark: boolean) {
      const html = document.documentElement
      const overlay = html.classList.contains('overlay-shell')
      const removeList: string[] = []
      html.classList.forEach((cls) => {
        if (cls.startsWith('theme-') || cls === 'dark') removeList.push(cls)
      })
      removeList.forEach((cls) => html.classList.remove(cls))
      html.classList.add(`theme-${theme}`)
      if (isDark || overlay) html.classList.add('dark')
      if (overlay) html.classList.add('overlay-shell')
    }

    // 设置颜色
    const setColor = (colorName: string) => {
      selectedColor.value = colorName
      setThemeClass(selectedColor.value, isDark.value)
    }

    // 设置圆角
    const setRadius = (radius: number) => {
      selectedRadius.value = radius
      document.documentElement.style.setProperty('--radius', `${radius}rem`)
    }

    // 设置风格
    const setStyle = (styleName: string) => {
      selectedStyle.value = styleName
    }

    // 切换主题
    const toggleTheme = (newValue: boolean) => {
      isDark.value = newValue
      setThemeClass(selectedColor.value, isDark.value)
    }

    // 重置主题
    const resetTheme = () => {
      selectedColor.value = 'neutral'
      selectedRadius.value = 0.5
      selectedStyle.value = 'new-york'
      isDark.value = false
      setThemeClass(selectedColor.value, isDark.value)
      document.documentElement.style.setProperty('--radius', '0.5rem')
    }

    // 初始化主题
    const initTheme = () => {
      // 检查系统主题偏好（仅在首次访问且无持久化数据时）
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

      // 如果当前 isDark 为 false 且系统偏好暗色主题，则使用系统偏好
      // 这只在首次访问应用时生效，后续以用户设置为准
      if (!isDark.value && mediaQuery.matches) {
        // 检查是否是首次访问（通过检查是否有持久化的主题配置）
        const hasPersistedTheme =
          selectedColor.value !== 'neutral' || selectedRadius.value !== 0.5 || selectedStyle.value !== 'new-york'

        if (!hasPersistedTheme) {
          isDark.value = true
        }
      }

      // 应用当前状态到 DOM
      setThemeClass(selectedColor.value, isDark.value)

      // 应用圆角设置
      document.documentElement.style.setProperty('--radius', `${selectedRadius.value}rem`)

      lastPerformanceCategory.value = normalizePerformanceCategory(lastPerformanceCategory.value)
      lastRankedScope.value = normalizeRankedScope(lastRankedScope.value)

      // 监听系统主题变化（仅作为参考，不强制覆盖用户设置）
      mediaQuery.addEventListener('change', (e) => {
        console.log('[SettingsStore] 系统主题偏好变化:', e.matches ? 'dark' : 'light')
        // 这里可以选择是否要跟随系统主题，当前保持用户设置
      })
    }

    // 应用设置方法
    const setAutoStart = (enabled: boolean) => {
      autoStart.value = enabled
    }

    const setMinimizeToTray = (enabled: boolean) => {
      minimizeToTray.value = enabled
    }

    const setShowNotifications = (enabled: boolean) => {
      showNotifications.value = enabled
    }

    const setLanguage = (lang: string) => {
      language.value = lang
    }

    // 游戏设置方法
    const setCareerBackground = (background: string) => {
      careerBackground.value = background
    }

    const setAutoRefreshData = (enabled: boolean) => {
      autoRefreshData.value = enabled
    }

    const setRefreshInterval = (interval: number) => {
      refreshInterval.value = Math.max(5000, interval) // 最小5秒
    }

    const setPerformanceScope = (scope: PerformanceScope) => {
      lastPerformanceCategory.value = normalizePerformanceCategory(scope.category)
      lastRankedScope.value = normalizeRankedScope(scope.rankedScope)
    }

    const setAugmentOverlayShortcut = (shortcut: string) => {
      augmentOverlayShortcut.value = shortcut.trim() || DEFAULT_OVERLAY_SHORTCUT
    }

    // 重置所有设置
    const resetAllSettings = () => {
      resetTheme()
      autoStart.value = false
      minimizeToTray.value = true
      showNotifications.value = true
      language.value = 'zh-CN'
      careerBackground.value = ''
      autoRefreshData.value = true
      refreshInterval.value = 30000
      setPerformanceScope({ category: 'ranked', rankedScope: 'mixed' })
      setAugmentOverlayShortcut(DEFAULT_OVERLAY_SHORTCUT)
    }

    // 导出设置
    const exportSettings = () => {
      return {
        theme: themeConfig.value,
        app: {
          autoStart: autoStart.value,
          minimizeToTray: minimizeToTray.value,
          showNotifications: showNotifications.value,
          language: language.value
        },
        game: {
          careerBackground: careerBackground.value,
          autoRefreshData: autoRefreshData.value,
          refreshInterval: refreshInterval.value,
          lastPerformanceCategory: lastPerformanceCategory.value,
          lastRankedScope: lastRankedScope.value,
          augmentOverlayShortcut: augmentOverlayShortcut.value
        }
      }
    }

    // 导入设置
    const importSettings = (settings: Partial<ReturnType<typeof exportSettings>>) => {
      if (settings.theme) {
        selectedColor.value = settings.theme.color || 'neutral'
        selectedRadius.value = settings.theme.radius || 0.5
        selectedStyle.value = settings.theme.style || 'new-york'
        isDark.value = settings.theme.isDark || false
      }

      if (settings.app) {
        autoStart.value = settings.app.autoStart || false
        minimizeToTray.value = settings.app.minimizeToTray ?? true
        showNotifications.value = settings.app.showNotifications ?? true
        language.value = settings.app.language || 'zh-CN'
      }

      if (settings.game) {
        careerBackground.value = settings.game.careerBackground || ''
        autoRefreshData.value = settings.game.autoRefreshData ?? true
        refreshInterval.value = settings.game.refreshInterval || 30000
        const category = (settings.game as { lastPerformanceCategory?: string }).lastPerformanceCategory
        const rankedScope = (settings.game as { lastRankedScope?: string }).lastRankedScope
        setPerformanceScope({
          category: normalizePerformanceCategory(category ?? 'ranked'),
          rankedScope: normalizeRankedScope(rankedScope ?? 'mixed')
        })
        const shortcut = (settings.game as { augmentOverlayShortcut?: string }).augmentOverlayShortcut
        if (typeof shortcut === 'string') setAugmentOverlayShortcut(shortcut)
      }
    }

    return {
      // 主题状态
      selectedColor,
      selectedRadius,
      selectedStyle,
      isDark,

      // 主题选项（从配置文件导入）
      colors,
      radiusOptions,
      styles,

      // 应用设置
      autoStart,
      minimizeToTray,
      showNotifications,
      language,

      // 游戏设置
      careerBackground,
      autoRefreshData,
      refreshInterval,
      lastPerformanceCategory,
      lastRankedScope,
      augmentOverlayShortcut,

      // 计算属性
      themeConfig,
      performanceScope,

      // 主题方法
      setColor,
      setRadius,
      setStyle,
      toggleTheme,
      resetTheme,
      initTheme,

      // 应用设置方法
      setAutoStart,
      setMinimizeToTray,
      setShowNotifications,
      setLanguage,

      // 游戏设置方法
      setCareerBackground,
      setAutoRefreshData,
      setRefreshInterval,
      setPerformanceScope,
      setAugmentOverlayShortcut,
      resetAllSettings,
      exportSettings,
      importSettings
    }
  },
  {
    persist: true
  }
)
