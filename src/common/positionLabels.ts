/** 后端位置码 → 中文展示（展示层唯一映射） */
const POSITION_LABELS: Record<string, string> = {
  TOP: '上单',
  JUNGLE: '打野',
  MID: '中单',
  ADC: 'ADC',
  SUPPORT: '辅助',
  ARAM: '大乱斗',
  FLEX: '灵活',
  UNKNOWN: '未知'
}

export function getPositionLabel(code: string | null | undefined): string {
  if (!code) return POSITION_LABELS.UNKNOWN
  return POSITION_LABELS[code] || POSITION_LABELS[code.toUpperCase()] || code
}
