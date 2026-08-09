import { colors, radiusOptions, styles } from '@/lib/theme'
import {
  isMatchModeKey,
  matchModeToAnalysisMode,
  matchModeToQueueIds,
  type MatchModeKey
} from '@/common/queueCatalog'
import { useAnalysisSettingsStore } from '@/shared/stores/features/analysisSettingsStore'

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
    // 是否记住仪表盘模式/场数（关闭则下次启动恢复全部/20）
    const rememberMatchPreferences = ref(true)
    // 当前拉取偏好（仪表盘改动始终写入，供自动刷新与两条战绩接口共用）
    const lastMatchMode = ref<MatchModeKey>('all')
    // 由 lastMatchMode 派生，供战绩搜索过滤复用
    const defaultQueueTypes = ref<number[]>([])
    const applyDefaultFilterOnSearch = ref(true)
    const lastMatchCount = ref<number>(20)
    const allowedMatchCounts = [20, 25, 30] as const

    // 计算属性
    const themeConfig = computed(() => ({
      color: selectedColor.value,
      radius: selectedRadius.value,
      style: selectedStyle.value,
      isDark: isDark.value
    }))

    // 主题相关class同步
    function setThemeClass(theme: string, isDark: boolean) {
      const html = document.documentElement
      const removeList: string[] = []
      html.classList.forEach((cls) => {
        if (cls.startsWith('theme-') || cls === 'dark') removeList.push(cls)
      })
      removeList.forEach((cls) => html.classList.remove(cls))
      html.classList.add(`theme-${theme}`)
      if (isDark) html.classList.add('dark')
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

      // 兼容旧字段 defaultMatchMode → lastMatchMode
      try {
        const raw = localStorage.getItem('settings')
        if (raw) {
          const data = JSON.parse(raw) as Record<string, unknown>
          if (
            !data.lastMatchMode &&
            typeof data.defaultMatchMode === 'string' &&
            isMatchModeKey(data.defaultMatchMode)
          ) {
            lastMatchMode.value = data.defaultMatchMode
          }
          if (
            data.lastMatchCount == null &&
            typeof data.defaultMatchCount === 'number' &&
            (allowedMatchCounts as readonly number[]).includes(data.defaultMatchCount)
          ) {
            lastMatchCount.value = data.defaultMatchCount
          }
        }
      } catch {
        // ignore
      }

      // 「记住」关闭时：启动恢复为全部 / 20，不沿用上次选择
      if (!rememberMatchPreferences.value) {
        lastMatchMode.value = 'all'
        lastMatchCount.value = 20
      }

      // 当前拉取偏好与派生队列过滤 / 分析策略保持一致
      defaultQueueTypes.value = matchModeToQueueIds(lastMatchMode.value)
      try {
        useAnalysisSettingsStore().setDefaultMode(matchModeToAnalysisMode(lastMatchMode.value))
      } catch {
        // ignore
      }

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

    const setRememberMatchPreferences = (enabled: boolean) => {
      rememberMatchPreferences.value = enabled
    }

    /** 写入上次战绩模式（供搜索过滤 / 分析策略同步） */
    const setLastMatchMode = (mode: MatchModeKey) => {
      lastMatchMode.value = mode
      defaultQueueTypes.value = matchModeToQueueIds(mode)
      try {
        useAnalysisSettingsStore().setDefaultMode(matchModeToAnalysisMode(mode))
      } catch {
        // Pinia 尚未就绪时忽略
      }
    }

    // 战绩默认过滤方法（兼容旧逻辑 / 搜索页）
    const setDefaultQueueTypes = (queues: number[]) => {
      const unique = Array.from(new Set(queues))
      unique.sort((a, b) => a - b)
      defaultQueueTypes.value = unique
    }

    const setApplyDefaultFilterOnSearch = (enabled: boolean) => {
      applyDefaultFilterOnSearch.value = enabled
    }

    const setLastMatchCount = (count: number) => {
      lastMatchCount.value = (allowedMatchCounts as readonly number[]).includes(count) ? count : 20
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
      rememberMatchPreferences.value = true
      setLastMatchMode('all')
      applyDefaultFilterOnSearch.value = true
      setLastMatchCount(20)
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
          rememberMatchPreferences: rememberMatchPreferences.value,
          lastMatchMode: lastMatchMode.value,
          lastMatchCount: lastMatchCount.value,
          defaultQueueTypes: defaultQueueTypes.value,
          applyDefaultFilterOnSearch: applyDefaultFilterOnSearch.value
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
        const mode =
          (settings.game as { lastMatchMode?: string; defaultMatchMode?: string }).lastMatchMode ??
          (settings.game as { defaultMatchMode?: string }).defaultMatchMode
        if (typeof mode === 'string' && isMatchModeKey(mode)) {
          setLastMatchMode(mode)
        } else if (Array.isArray(settings.game.defaultQueueTypes)) {
          const queues = settings.game.defaultQueueTypes
          if (queues.length === 0) setLastMatchMode('all')
          else if (queues.length === 2 && queues.includes(420) && queues.includes(440)) {
            setLastMatchMode('mixedRanked')
          } else if (queues.length === 1) setLastMatchMode(String(queues[0]) as MatchModeKey)
          else setDefaultQueueTypes(queues)
        }
        applyDefaultFilterOnSearch.value = settings.game.applyDefaultFilterOnSearch ?? true
        rememberMatchPreferences.value =
          (settings.game as { rememberMatchPreferences?: boolean }).rememberMatchPreferences ?? true
        const count =
          (settings.game as { lastMatchCount?: number; defaultMatchCount?: number }).lastMatchCount ??
          (settings.game as { defaultMatchCount?: number }).defaultMatchCount
        if (typeof count === 'number') setLastMatchCount(count)
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
      rememberMatchPreferences,
      lastMatchMode,
      lastMatchCount,
      allowedMatchCounts,
      defaultQueueTypes,
      applyDefaultFilterOnSearch,

      // 计算属性
      themeConfig,

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
      setRememberMatchPreferences,
      setLastMatchMode,
      setLastMatchCount,
      setDefaultQueueTypes,
      setApplyDefaultFilterOnSearch,
      resetAllSettings,
      exportSettings,
      importSettings
    }
  },
  {
    persist: true
  }
)
