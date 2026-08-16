/** 排位队列 */
export const RANKED_QUEUE_IDS = [420, 440] as const

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
