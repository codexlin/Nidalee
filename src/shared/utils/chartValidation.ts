/**
 * 图表数据验证和错误处理工具
 */

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
export function validatePositionStats(positionData: any): PositionStatsValidation {
  const errors: string[] = []
  const gamesCount = positionData?.games || 0

  // 基础数据验证
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

  // 检查英雄池数据
  const hasChampionPool = positionData.championPool && positionData.championPool.length > 0
  if (!hasChampionPool && gamesCount > 0) {
    errors.push('英雄池数据缺失')
  }

  // 检查趋势数据
  const hasTrendData = positionData.winRateTrend && positionData.winRateTrend.length > 0
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
export function validateRadarData(stats: any): ChartDataValidation {
  const errors: string[] = []
  const warnings: string[] = []

  if (!stats) {
    errors.push('统计数据为空')
    return { isValid: false, errors, warnings }
  }

  // 检查必要字段
  const requiredFields = ['avgKda', 'cspm', 'vspm', 'avgKills', 'avgDeaths', 'avgAssists', 'dpm']
  for (const field of requiredFields) {
    if (stats[field] === undefined || stats[field] === null) {
      errors.push(`缺少必要字段: ${field}`)
    }
  }

  // 数据合理性检查
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
export function validateTrendData(trendData: any[]): ChartDataValidation {
  const errors: string[] = []
  const warnings: string[] = []

  if (!trendData || trendData.length === 0) {
    errors.push('趋势数据为空')
    return { isValid: false, errors, warnings }
  }

  // 检查数据点结构
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
export function validateChampionPool(championPool: any[]): ChartDataValidation {
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

    if (typeof champ.games !== 'number' || champ.games < 0) {
      errors.push(`第${i + 1}个英雄的场次数据异常`)
    }

    if (typeof champ.winRate !== 'number' || champ.winRate < 0 || champ.winRate > 100) {
      warnings.push(`第${i + 1}个英雄的胜率数据异常`)
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

  const errorCount = validation.errors.length
  const warningCount = validation.warnings.length

  let message = `数据验证失败 (${errorCount}个错误`
  if (warningCount > 0) {
    message += `, ${warningCount}个警告`
  }
  message += '):\n'

  validation.errors.forEach((error) => {
    message += `• ${error}\n`
  })

  if (validation.warnings.length > 0) {
    message += '\n警告:\n'
    validation.warnings.forEach((warning) => {
      message += `• ${warning}\n`
    })
  }

  return message.trim()
}

/**
 * 图表数据验证和错误处理工具
 */

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
export function validatePositionStats(positionData: any): PositionStatsValidation {
  const errors: string[] = []
  const gamesCount = positionData?.games || 0

  // 基础数据验证
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

  // 检查英雄池数据
  const hasChampionPool = positionData.championPool && positionData.championPool.length > 0
  if (!hasChampionPool && gamesCount > 0) {
    errors.push('英雄池数据缺失')
  }

  // 检查趋势数据
  const hasTrendData = positionData.winRateTrend && positionData.winRateTrend.length > 0
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
export function validateRadarData(stats: any): ChartDataValidation {
  const errors: string[] = []
  const warnings: string[] = []

  if (!stats) {
    errors.push('统计数据为空')
    return { isValid: false, errors, warnings }
  }

  // 检查必要字段
  const requiredFields = ['avgKda', 'cspm', 'vspm', 'avgKills', 'avgDeaths', 'avgAssists', 'dpm']
  for (const field of requiredFields) {
    if (stats[field] === undefined || stats[field] === null) {
      errors.push(`缺少必要字段: ${field}`)
    }
  }

  // 数据合理性检查
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
export function validateTrendData(trendData: any[]): ChartDataValidation {
  const errors: string[] = []
  const warnings: string[] = []

  if (!trendData || trendData.length === 0) {
    errors.push('趋势数据为空')
    return { isValid: false, errors, warnings }
  }

  // 检查数据点结构
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
export function validateChampionPool(championPool: any[]): ChartDataValidation {
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

    if (typeof champ.games !== 'number' || champ.games < 0) {
      errors.push(`第${i + 1}个英雄的场次数据异常`)
    }

    if (typeof champ.winRate !== 'number' || champ.winRate < 0 || champ.winRate > 100) {
      warnings.push(`第${i + 1}个英雄的胜率数据异常`)
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

  const errorCount = validation.errors.length
  const warningCount = validation.warnings.length

  let message = `数据验证失败 (${errorCount}个错误`
  if (warningCount > 0) {
    message += `, ${warningCount}个警告`
  }
  message += '):\n'

  validation.errors.forEach((error) => {
    message += `• ${error}\n`
  })

  if (validation.warnings.length > 0) {
    message += '\n警告:\n'
    validation.warnings.forEach((warning) => {
      message += `• ${warning}\n`
    })
  }

  return message.trim()
}
