/** OP.GG 构建/强度榜支持的模式（不含 TFT） */
export type OpggGameMode = 'ranked' | 'aram' | 'urf' | 'arena'

export const OPGG_MODES: Array<{ value: OpggGameMode; label: string; hint?: string }> = [
  { value: 'ranked', label: '排位赛' },
  { value: 'aram', label: '大乱斗' },
  { value: 'urf', label: '无限火力', hint: '限时模式，数据可能停更' },
  { value: 'arena', label: '斗魂竞技场' }
]

export function isRankedMode(mode: string): boolean {
  return mode === 'ranked'
}

export function usesLanePosition(mode: string): boolean {
  return mode === 'ranked'
}

export function usesTierFilter(mode: string): boolean {
  return mode === 'ranked'
}

/** 强度榜主指标列名 */
export function primaryRateLabel(mode: string): string {
  return mode === 'arena' ? '吃鸡' : '胜率'
}

export function showBanColumn(mode: string): boolean {
  return mode === 'ranked' || mode === 'arena'
}

export function showCounterColumn(mode: string): boolean {
  return mode === 'ranked'
}

export function showRoleColumn(mode: string): boolean {
  return mode === 'aram' || mode === 'hextech'
}

/** 构建请求用的 position 槽（与后端 resolve 一致，前端也传对） */
export function buildRequestPosition(mode: string, position: string): string | null {
  if (mode === 'aram' || mode === 'urf') return 'none'
  if (mode === 'arena') return null
  return position || 'MID'
}
