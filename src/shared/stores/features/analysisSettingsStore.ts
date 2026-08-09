import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

// 分析深度枚举
export enum AnalysisDepth {
  Simple = 'simple', // 简单分析 (2层)
  Deep = 'deep' // 深度分析 (5层)
}

// 分析模式枚举
export enum AnalysisMode {
  SoloRanked = 'soloRanked',
  FlexRanked = 'flexRanked',
  MixedRanked = 'mixedRanked',
  Aram = 'aram',
  AllModes = 'allModes'
}

// 分析配置接口
export interface AnalysisConfig {
  // 基础设置
  enabled: boolean // 是否启用智能分析
  depth: AnalysisDepth // 分析深度
  defaultMode: AnalysisMode // 默认分析模式

  // 高级设置
  enableTimelineAnalysis: boolean // 启用时间线分析
  enableOpponentAnalysis: boolean // 启用对手分析
  enableTeammateAnalysis: boolean // 启用队友分析
  enableSelfImprovement: boolean // 启用自我提升分析

  // 性能设置
  maxAnalysisGames: number // 最大分析对局数
  enableCaching: boolean // 启用分析缓存
  cacheExpirationHours: number // 缓存过期时间(小时)

  // 显示设置
  showDetailedAdvice: boolean // 显示详细建议
  showPositionComparison: boolean // 显示位置对比
  showChampionPool: boolean // 显示英雄池分析
  showTrendCharts: boolean // 显示趋势图表
}

// 默认配置
const defaultAnalysisConfig: AnalysisConfig = {
  enabled: true,
  depth: AnalysisDepth.Deep,
  defaultMode: AnalysisMode.AllModes,

  enableTimelineAnalysis: true,
  enableOpponentAnalysis: true,
  enableTeammateAnalysis: true,
  enableSelfImprovement: true,

  maxAnalysisGames: 20,
  enableCaching: true,
  cacheExpirationHours: 24,

  showDetailedAdvice: true,
  showPositionComparison: true,
  showChampionPool: true,
  showTrendCharts: true
}

export const useAnalysisSettingsStore = defineStore(
  'analysisSettings',
  () => {
    // 状态
    const config = ref<AnalysisConfig>({ ...defaultAnalysisConfig })
    const isLoaded = ref(false)

    // 计算属性
    const isEnabled = computed(() => config.value.enabled)
    const isDeepAnalysis = computed(() => config.value.depth === AnalysisDepth.Deep)
    const isSimpleAnalysis = computed(() => config.value.depth === AnalysisDepth.Simple)

    // 分析功能状态
    const analysisFeatures = computed(() => ({
      timeline: config.value.enableTimelineAnalysis,
      opponent: config.value.enableOpponentAnalysis,
      teammate: config.value.enableTeammateAnalysis,
      selfImprovement: config.value.enableSelfImprovement
    }))

    // 显示功能状态
    const displayFeatures = computed(() => ({
      detailedAdvice: config.value.showDetailedAdvice,
      positionComparison: config.value.showPositionComparison,
      championPool: config.value.showChampionPool,
      trendCharts: config.value.showTrendCharts
    }))

    // 方法

    /**
     * 启用/禁用智能分析
     */
    const toggleAnalysis = (enabled: boolean) => {
      config.value.enabled = enabled
    }

    /**
     * 设置分析深度
     */
    const setAnalysisDepth = (depth: AnalysisDepth) => {
      config.value.depth = depth
    }

    /**
     * 设置默认分析模式
     */
    const setDefaultMode = (mode: AnalysisMode) => {
      config.value.defaultMode = mode
    }

    /**
     * 切换分析功能
     */
    const toggleAnalysisFeature = (feature: keyof typeof analysisFeatures.value, enabled: boolean) => {
      switch (feature) {
        case 'timeline':
          config.value.enableTimelineAnalysis = enabled
          break
        case 'opponent':
          config.value.enableOpponentAnalysis = enabled
          break
        case 'teammate':
          config.value.enableTeammateAnalysis = enabled
          break
        case 'selfImprovement':
          config.value.enableSelfImprovement = enabled
          break
      }
    }

    /**
     * 切换显示功能
     */
    const toggleDisplayFeature = (feature: keyof typeof displayFeatures.value, enabled: boolean) => {
      switch (feature) {
        case 'detailedAdvice':
          config.value.showDetailedAdvice = enabled
          break
        case 'positionComparison':
          config.value.showPositionComparison = enabled
          break
        case 'championPool':
          config.value.showChampionPool = enabled
          break
        case 'trendCharts':
          config.value.showTrendCharts = enabled
          break
      }
    }

    /**
     * 设置性能参数
     */
    const setPerformanceSettings = (settings: {
      maxAnalysisGames?: number
      enableCaching?: boolean
      cacheExpirationHours?: number
    }) => {
      if (settings.maxAnalysisGames !== undefined) {
        config.value.maxAnalysisGames = Math.max(5, Math.min(50, settings.maxAnalysisGames))
      }
      if (settings.enableCaching !== undefined) {
        config.value.enableCaching = settings.enableCaching
      }
      if (settings.cacheExpirationHours !== undefined) {
        config.value.cacheExpirationHours = Math.max(1, Math.min(168, settings.cacheExpirationHours))
      }
    }

    /**
     * 重置为默认配置
     */
    const resetToDefault = () => {
      config.value = { ...defaultAnalysisConfig }
    }

    /**
     * 导出配置
     */
    const exportConfig = (): string => {
      return JSON.stringify(config.value, null, 2)
    }

    /**
     * 导入配置
     */
    const importConfig = (jsonData: string) => {
      try {
        const importedConfig = JSON.parse(jsonData)
        config.value = { ...defaultAnalysisConfig, ...importedConfig }
      } catch (error) {
        console.error('导入分析配置失败:', error)
        throw new Error('导入失败：数据格式不正确')
      }
    }

    /**
     * 获取分析模式描述
     */
    const getModeDescription = (mode: AnalysisMode): string => {
      const descriptions = {
        [AnalysisMode.SoloRanked]: '单排分析 - 只分析单排对局',
        [AnalysisMode.FlexRanked]: '灵活组排分析 - 只分析灵活组排对局',
        [AnalysisMode.MixedRanked]: '混合排位分析 - 分析单排+灵活组排对局',
        [AnalysisMode.Aram]: '大乱斗分析 - 只分析大乱斗对局',
        [AnalysisMode.AllModes]: '全部模式分析 - 分析所有对局'
      }
      return descriptions[mode]
    }

    /**
     * 获取分析深度描述
     */
    const getDepthDescription = (depth: AnalysisDepth): string => {
      const descriptions = {
        [AnalysisDepth.Simple]: '简单分析 - 基础统计和简单建议 (2层分析)',
        [AnalysisDepth.Deep]: '深度分析 - 完整的多层分析和智能建议 (5层分析)'
      }
      return descriptions[depth]
    }

    /**
     * 分析功能开关 → 后端 AnalysisFeatureFlags（进入 request.features）
     */
    const toFeatureFlags = (): AnalysisFeatureFlags => ({
      enabled: config.value.enabled,
      timeline: config.value.enableTimelineAnalysis,
      opponent: config.value.enableOpponentAnalysis,
      teammate: config.value.enableTeammateAnalysis,
      selfImprovement: config.value.enableSelfImprovement
    })

    return {
      // 状态
      config,
      isLoaded,

      // 计算属性
      isEnabled,
      isDeepAnalysis,
      isSimpleAnalysis,
      analysisFeatures,
      displayFeatures,

      // 方法
      toggleAnalysis,
      setAnalysisDepth,
      setDefaultMode,
      toggleAnalysisFeature,
      toggleDisplayFeature,
      setPerformanceSettings,
      resetToDefault,
      exportConfig,
      importConfig,
      getModeDescription,
      getDepthDescription,
      toFeatureFlags
    }
  },
  {
    persist: true
  }
)
