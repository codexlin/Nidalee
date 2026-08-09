/**
 * 图表数据验证和错误处理工具
 */

/** 与后端 TrendPoint 对齐；PositionStats.winRateTrend 已是该类型 */
export type WinRateTrendPoint = TrendPoint

/** PositionStats 已包含 winRateTrend，不再交叉扩展造成冲突 */
export type PositionStatsWithTrend = PositionStats

export interface ChartDataValidation {
  isValid: boolean
  errors: string[]
  warnings: string[]
}

export interface PositionStatsValidation {
  hasValidStats: boolean
  hasChampionPool: boolean
  hasTrendData: boolean
  gamesCount: number
  errors: string[]
}

/**
 * 验证位置统计数据
 */
export function validatePositionStats(
  positionData: PositionStatsWithTrend | null | undefined
): PositionStatsValidation {
  const errors: string[] = []
  const gamesCount = positionData?.games || 0

  if (!positionData) {
    errors.push('位置数据为空')
    return {
      hasValidStats: false,
      hasChampionPool: false,
      hasTrendData: false,
      gamesCount: 0,
      errors
    }
  }

  if (!positionData.stats) {
    errors.push('统计数据缺失')
  }

  if (gamesCount === 0) {
    errors.push('该位置没有对局数据')
  }

  const hasChampionPool = !!(positionData.championPool?.length || positionData.stats?.favoriteChampions?.length)
  if (!hasChampionPool && gamesCount > 0) {
    errors.push('英雄池数据缺失')
  }

  const hasTrendData = !!positionData.winRateTrend?.length
  if (!hasTrendData && gamesCount > 0) {
    errors.push('胜率趋势数据缺失')
  }

  return {
    hasValidStats: errors.length === 0,
    hasChampionPool,
    hasTrendData,
    gamesCount,
    errors
  }
}

/**
 * 验证雷达图数据
 */
export function validateRadarData(stats: PlayerMatchStats | null | undefined): ChartDataValidation {
  const errors: string[] = []
  const warnings: string[] = []

  if (!stats) {
    errors.push('统计数据为空')
    return { isValid: false, errors, warnings }
  }

  const requiredFields = ['avgKda', 'cspm', 'vspm', 'avgKills', 'avgDeaths', 'avgAssists', 'dpm'] as const
  for (const field of requiredFields) {
    if (stats[field] === undefined || stats[field] === null) {
      errors.push(`缺少必要字段: ${field}`)
    }
  }

  if (stats.avgKda < 0) warnings.push('KDA值异常')
  if (stats.cspm < 0) warnings.push('补刀数据异常')
  if (stats.vspm < 0) warnings.push('视野数据异常')
  if (stats.dpm < 0) warnings.push('伤害数据异常')

  return {
    isValid: errors.length === 0,
    errors,
    warnings
  }
}

/**
 * 验证趋势图数据
 */
export function validateTrendData(
  trendData: WinRateTrendPoint[] | null | undefined
): ChartDataValidation {
  const errors: string[] = []
  const warnings: string[] = []

  if (!trendData || trendData.length === 0) {
    errors.push('趋势数据为空')
    return { isValid: false, errors, warnings }
  }

  for (let i = 0; i < trendData.length; i++) {
    const point = trendData[i]
    if (!point) {
      errors.push(`第${i + 1}个数据点为空`)
      continue
    }

    if (typeof point.cumulativeWinRate !== 'number') {
      errors.push(`第${i + 1}个数据点的累计胜率格式错误`)
    }

    if (typeof point.movingAvgWinRate !== 'number') {
      errors.push(`第${i + 1}个数据点的移动平均胜率格式错误`)
    }

    if (point.cumulativeWinRate < 0 || point.cumulativeWinRate > 100) {
      warnings.push(`第${i + 1}个数据点的累计胜率超出正常范围`)
    }
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings
  }
}

/**
 * 验证英雄池数据
 */
export function validateChampionPool(
  championPool: Array<ChampionStat | AnalysisChampionStats> | null | undefined
): ChartDataValidation {
  const errors: string[] = []
  const warnings: string[] = []

  if (!championPool || championPool.length === 0) {
    errors.push('英雄池数据为空')
    return { isValid: false, errors, warnings }
  }

  for (let i = 0; i < championPool.length; i++) {
    const champ = championPool[i]
    if (!champ) {
      errors.push(`第${i + 1}个英雄数据为空`)
      continue
    }

    if (!champ.championId) {
      errors.push(`第${i + 1}个英雄缺少ID`)
    }

    if (champ.games <= 0) {
      warnings.push(`第${i + 1}个英雄场次异常`)
    }
  }

  return {
    isValid: errors.length === 0,
    errors,
    warnings
  }
}

/**
 * 生成图表错误信息
 */
export function generateChartErrorMessage(validation: ChartDataValidation): string {
  if (validation.isValid) return ''
  const parts = [...validation.errors]
  if (validation.warnings.length) {
    parts.push(...validation.warnings.map((w) => `警告: ${w}`))
  }
  return parts.join('\n')
}
