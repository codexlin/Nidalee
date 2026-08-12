export const OPGG_TIER_OPTIONS = [
  { value: 'emerald_plus', label: '翡翠+' },
  { value: 'platinum_plus', label: '铂金+' },
  { value: 'diamond_plus', label: '钻石+' },
  { value: 'all', label: '全部段位' },
  { value: 'iron', label: '黑铁' },
  { value: 'bronze', label: '青铜' },
  { value: 'silver', label: '白银' },
  { value: 'gold', label: '黄金' },
  { value: 'platinum', label: '铂金' },
  { value: 'emerald', label: '翡翠' },
  { value: 'diamond', label: '钻石' },
  { value: 'master', label: '大师' },
  { value: 'grandmaster', label: '宗师' },
  { value: 'challenger', label: '王者' }
] as const

export type OpggTier = (typeof OPGG_TIER_OPTIONS)[number]['value']

const OP_GG_TIER_VALUES = new Set<string>(OPGG_TIER_OPTIONS.map(({ value }) => value))

export const AUTO_RUNE_OPGG_TIER_OPTIONS = OPGG_TIER_OPTIONS.filter(({ value }) =>
  ['all', 'platinum_plus', 'emerald_plus', 'diamond_plus', 'master'].includes(value)
)

export function isOpggTier(value: unknown): value is OpggTier {
  return typeof value === 'string' && OP_GG_TIER_VALUES.has(value)
}

export function getOpggTierLabel(tier: OpggTier): string {
  return OPGG_TIER_OPTIONS.find(({ value }) => value === tier)?.label ?? tier
}
