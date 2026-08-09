import { AnalysisMode } from '@/shared/stores/features/analysisSettingsStore'

/**
 * 统一战绩模式 key：
 * - all: 全部模式
 * - mixedRanked: 单双+灵活
 * - 数字字符串: 具体 queueId
 */
export type MatchModeKey = 'all' | 'mixedRanked' | `${number}`

export interface MatchModeOption {
  key: MatchModeKey
  /** 本地兜底名称（CDragon 未加载时使用） */
  fallbackLabel: string
  queueIds: number[]
  analysisMode: AnalysisMode
}

/** 仪表盘 / 游戏设置共用的模式目录 */
export const MATCH_MODE_OPTIONS: MatchModeOption[] = [
  {
    key: 'all',
    fallbackLabel: '全部模式',
    queueIds: [],
    analysisMode: AnalysisMode.AllModes
  },
  {
    key: 'mixedRanked',
    fallbackLabel: '排位赛（单双+灵活）',
    queueIds: [420, 440],
    analysisMode: AnalysisMode.MixedRanked
  },
  {
    key: '420',
    fallbackLabel: '单双排',
    queueIds: [420],
    analysisMode: AnalysisMode.SoloRanked
  },
  {
    key: '440',
    fallbackLabel: '灵活组排',
    queueIds: [440],
    analysisMode: AnalysisMode.FlexRanked
  },
  {
    key: '450',
    fallbackLabel: '极地大乱斗',
    queueIds: [450],
    analysisMode: AnalysisMode.Aram
  },
  {
    key: '2400',
    fallbackLabel: '海克斯大乱斗',
    queueIds: [2400],
    analysisMode: AnalysisMode.AllModes
  },
  {
    key: '1700',
    fallbackLabel: '斗魂竞技场',
    queueIds: [1700],
    analysisMode: AnalysisMode.AllModes
  },
  {
    key: '900',
    fallbackLabel: '无限火力',
    queueIds: [900],
    analysisMode: AnalysisMode.AllModes
  }
]

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
  2400: '海克斯大乱斗'
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

export function isMatchModeKey(value: string): value is MatchModeKey {
  return value === 'all' || value === 'mixedRanked' || /^\d+$/.test(value)
}

export function getMatchModeOption(key: MatchModeKey): MatchModeOption {
  return (
    MATCH_MODE_OPTIONS.find((option) => option.key === key) || {
      key,
      fallbackLabel: getQueueDisplayName(Number(key)),
      queueIds: key === 'all' || key === 'mixedRanked' ? [] : [Number(key)],
      analysisMode: AnalysisMode.AllModes
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

export function matchModeToAnalysisMode(key: MatchModeKey): AnalysisMode {
  return getMatchModeOption(key).analysisMode
}

export function matchModeToQueueIds(key: MatchModeKey): number[] {
  return [...getMatchModeOption(key).queueIds]
}

/** 传给 analyze_matches / MatchAnalysisRequest：单队列用 queueId，预设模式用 analysisMode */
export function matchModeToInvokeArgs(key: MatchModeKey): {
  queueId: number | null
  analysisMode: AnalysisMode
} {
  const option = getMatchModeOption(key)
  if (key === 'all' || key === 'mixedRanked' || option.queueIds.length !== 1) {
    return { queueId: null, analysisMode: option.analysisMode }
  }
  return { queueId: option.queueIds[0], analysisMode: option.analysisMode }
}
