export type HextechGuideAugment = {
  id: number
  name: string
  iconUrl: string
  rarityName: string
  rarityDisplayName: string
  winRate: number
  pickRate: number
  games: number | null
  tier: number | null
}

export type HextechGuideTrio = {
  augments: HextechGuideAugment[]
  winRate: number
  pickRate: number
  games: number | null
}

export type HextechRarityKey = 'prismatic' | 'gold' | 'silver' | 'other'

export type HextechRarityGroup = {
  key: HextechRarityKey
  label: string
  items: HextechGuideAugment[]
}

export type HextechTierKey = 1 | 2 | 3 | 4 | 0

export type HextechTierGroup = {
  tier: HextechTierKey
  label: string
  items: HextechGuideAugment[]
}

export const HEXTECH_RARITY_ORDER: HextechRarityKey[] = ['prismatic', 'gold', 'silver']

/** 低于 50% 不当推荐 */
export const WIN_RATE_FLOOR = 0.5

const RARITY_LABELS: Record<HextechRarityKey, string> = {
  prismatic: '棱彩',
  gold: '黄金',
  silver: '白银',
  other: '其他'
}

export function normalizeRarityKey(rarityName?: string | null, displayName?: string | null): HextechRarityKey {
  const raw = `${rarityName ?? ''} ${displayName ?? ''}`.trim().toLowerCase()
  if (!raw) return 'other'
  if (raw.includes('prism') || raw.includes('棱彩')) return 'prismatic'
  if (raw.includes('gold') || raw.includes('黄金')) return 'gold'
  if (raw.includes('silver') || raw.includes('白银')) return 'silver'
  return 'other'
}

export function rarityLabel(key: HextechRarityKey, displayName?: string | null): string {
  if (key === 'other') return displayName?.trim() || RARITY_LABELS.other
  return RARITY_LABELS[key]
}

export function groupAugmentsByRarity(augments: HextechGuideAugment[]): HextechRarityGroup[] {
  const buckets = new Map<HextechRarityKey, HextechGuideAugment[]>()
  for (const item of augments) {
    const key = normalizeRarityKey(item.rarityName, item.rarityDisplayName)
    const list = buckets.get(key) ?? []
    list.push(item)
    buckets.set(key, list)
  }

  const extras = [...buckets.keys()].filter((key) => !HEXTECH_RARITY_ORDER.includes(key))
  return [...HEXTECH_RARITY_ORDER, ...extras]
    .filter((key) => (buckets.get(key)?.length ?? 0) > 0)
    .map((key) => {
      const items = [...(buckets.get(key) ?? [])].sort(compareByWinRate)
      return {
        key,
        label: rarityLabel(key, items[0]?.rarityDisplayName),
        items
      }
    })
}

export function groupAugmentsByTier(items: HextechGuideAugment[]): HextechTierGroup[] {
  const buckets = new Map<HextechTierKey, HextechGuideAugment[]>()
  for (const item of items) {
    const key: HextechTierKey = item.tier === 1 || item.tier === 2 || item.tier === 3 || item.tier === 4 ? item.tier : 0
    const list = buckets.get(key) ?? []
    list.push(item)
    buckets.set(key, list)
  }
  const order: HextechTierKey[] = [1, 2, 3, 4, 0]
  return order
    .filter((tier) => (buckets.get(tier)?.length ?? 0) > 0)
    .map((tier) => ({
      tier,
      label: tier === 0 ? '其他' : `T${tier}`,
      items: [...(buckets.get(tier) ?? [])].sort(compareByWinRate)
    }))
}

/** 接口里大量 winRate/games 为 null，解析后会变成 0；没有场次的 0 不当成真实胜率 */
export function hasSampledWinRate(item: HextechGuideAugment): boolean {
  if (item.winRate === null || item.winRate === undefined || Number.isNaN(item.winRate)) return false
  if ((item.games === null || item.games === undefined) && item.winRate === 0) return false
  return true
}

export function filterRecommendedAugments(items: HextechGuideAugment[]): HextechGuideAugment[] {
  return items.filter((item) => hasSampledWinRate(item) && item.winRate >= WIN_RATE_FLOOR)
}

export function formatAugmentWinRate(item: HextechGuideAugment): string {
  return hasSampledWinRate(item) ? formatHextechPct(item.winRate) : '—'
}

function compareByWinRate(a: HextechGuideAugment, b: HextechGuideAugment): number {
  const sampled = Number(hasSampledWinRate(b)) - Number(hasSampledWinRate(a))
  if (sampled !== 0) return sampled
  return b.winRate - a.winRate
}

export function formatHextechPct(value: number | null | undefined): string {
  if (value === null || value === undefined || Number.isNaN(value)) return '—'
  return `${(value * 100).toFixed(1)}%`
}

export function formatHextechGames(value: number | null | undefined): string {
  if (value === null || value === undefined) return '—'
  if (value >= 1000) return `${(value / 1000).toFixed(1)}k`
  return String(value)
}
