/**
 * 统一战绩模式 key：
 * - all: 全部模式
 * - normals: 普通模式（非排位）
 * - mixedRanked: 排位（单双+灵活）
 * - 数字字符串: 具体 queueId（单双/灵活）
 *
 * AnalysisMode 来自 Rust → global.d.ts（IPC 契约），勿再维护前端枚举副本。
 */
export type MatchModeKey = 'all' | 'normals' | 'mixedRanked' | `${number}`

export interface MatchModeOption {
  key: MatchModeKey
  /** 本地兜底名称（CDragon 未加载时使用） */
  fallbackLabel: string
  queueIds: number[]
  analysisMode: AnalysisMode
  /** 排除排位（普通模式） */
  excludeRanked?: boolean
}

/** 排位队列 */
export const RANKED_QUEUE_IDS = [420, 440] as const

/**
 * 仪表盘模式目录（5 项）：
 * 全部 → 普通 → 排位混合 → 单双 → 灵活
 */
export const MATCH_MODE_OPTIONS: MatchModeOption[] = [
  {
    key: 'all',
    fallbackLabel: '全部模式',
    queueIds: [],
    analysisMode: 'allModes'
  },
  {
    key: 'normals',
    fallbackLabel: '普通模式',
    queueIds: [],
    analysisMode: 'normals',
    excludeRanked: true
  },
  {
    key: 'mixedRanked',
    fallbackLabel: '排位模式',
    queueIds: [420, 440],
    analysisMode: 'mixedRanked'
  },
  {
    key: '420',
    fallbackLabel: '单双排',
    queueIds: [420],
    analysisMode: 'soloRanked'
  },
  {
    key: '440',
    fallbackLabel: '灵活组排',
    queueIds: [440],
    analysisMode: 'flexRanked'
  }
]

const SELECTABLE_MATCH_MODE_KEYS = new Set<string>(MATCH_MODE_OPTIONS.map((o) => o.key))

/** 是否仍在 Dashboard 下拉里可选 */
export function isSelectableMatchMode(key: string): key is MatchModeKey {
  return SELECTABLE_MATCH_MODE_KEYS.has(key)
}

/** 旧偏好迁移：大乱斗等娱乐单项 → 普通；未知 → 全部 */
export function normalizeMatchModeKey(key: string): MatchModeKey {
  if (isSelectableMatchMode(key)) return key
  if (key === '450' || key === 'aram') return 'normals'
  if (key === '900' || key === '1700' || key === '2400' || key === '1900') return 'normals'
  if (/^\d+$/.test(key)) {
    const id = Number(key)
    if (id === 420 || id === 440) return String(id) as MatchModeKey
    return 'normals'
  }
  return 'all'
}

const FALLBACK_QUEUE_NAMES: Record<number, string> = {
  0: '自定义',
  400: '灵活匹配',
  420: '单双排',
  430: '匹配模式',
  440: '灵活组排',
  450: '极地大乱斗',
  700: '冠军杯赛',
  900: '无限火力',
  1020: '克隆大作战',
  1200: '极限闪击',
  1400: '终极魔典',
  1700: '斗魂竞技场',
  1900: '无限火力',
  2300: '神木之门',
  2400: '海克斯大乱斗',
  3110: '自定义游戏',
  4310: '经典模式'
}

/** CDragon 队列中文名缓存 */
const cdragonQueueNames = new Map<number, string>()

export function setCdragonQueueNames(entries: Array<{ id: number; name: string }>) {
  cdragonQueueNames.clear()
  for (const entry of entries) {
    const name = entry.name?.trim()
    if (name) cdragonQueueNames.set(entry.id, name)
  }
}

export function getQueueDisplayName(queueId: number): string {
  return cdragonQueueNames.get(queueId) || FALLBACK_QUEUE_NAMES[queueId] || `未知队列(${queueId})`
}

/** 紧凑卡片使用短模式名，避免 CDragon 名称携带地图前缀后挤占内容。 */
export function getCompactQueueDisplayName(queueId: number): string {
  return FALLBACK_QUEUE_NAMES[queueId] || getQueueDisplayName(queueId)
}

export function isMatchModeKey(value: string): value is MatchModeKey {
  return value === 'all' || value === 'normals' || value === 'mixedRanked' || /^\d+$/.test(value)
}

export function getMatchModeOption(key: MatchModeKey): MatchModeOption {
  return (
    MATCH_MODE_OPTIONS.find((option) => option.key === key) || {
      key,
      fallbackLabel: getQueueDisplayName(Number(key)),
      queueIds: key === 'all' || key === 'normals' || key === 'mixedRanked' ? [] : [Number(key)],
      analysisMode: 'allModes'
    }
  )
}

export function getMatchModeLabel(key: MatchModeKey): string {
  const option = getMatchModeOption(key)
  if (option.queueIds.length === 1) {
    return getQueueDisplayName(option.queueIds[0])
  }
  return option.fallbackLabel
}

export function matchModeToQueueIds(key: MatchModeKey): number[] {
  return [...getMatchModeOption(key).queueIds]
}

export function matchModeExcludesRanked(key: MatchModeKey): boolean {
  return !!getMatchModeOption(key).excludeRanked
}

/** 传给 analyze_matches / MatchAnalysisRequest：单队列用 queueId，预设模式用 analysisMode */
export function matchModeToInvokeArgs(key: MatchModeKey): {
  queueId: number | null
  analysisMode: AnalysisMode
} {
  const option = getMatchModeOption(key)
  if (key === 'all' || key === 'normals' || key === 'mixedRanked' || option.queueIds.length !== 1) {
    return { queueId: null, analysisMode: option.analysisMode }
  }
  return { queueId: option.queueIds[0], analysisMode: option.analysisMode }
}
