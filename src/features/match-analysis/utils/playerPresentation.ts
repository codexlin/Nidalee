const RANK_LABELS: Record<string, string> = {
  IRON: '坚韧黑铁',
  BRONZE: '英勇黄铜',
  SILVER: '不屈白银',
  GOLD: '荣耀黄金',
  PLATINUM: '华贵铂金',
  EMERALD: '流光翡翠',
  DIAMOND: '璀璨钻石',
  MASTER: '超凡大师',
  GRANDMASTER: '傲世宗师',
  CHALLENGER: '最强王者'
}

export function formatRankLabel(tier?: string | null, division?: string | null, leaguePoints?: number | null): string {
  const normalizedTier = tier?.trim().toUpperCase()
  if (!normalizedTier) return ''

  const parts = [RANK_LABELS[normalizedTier] ?? normalizedTier]
  const normalizedDivision = division?.trim().toUpperCase()
  if (normalizedDivision && normalizedDivision !== 'NA') parts.push(normalizedDivision)

  const rank = parts.join(' ')
  return leaguePoints === null || leaguePoints === undefined ? rank : `${rank} · ${leaguePoints} LP`
}
