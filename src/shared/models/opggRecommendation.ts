import { normalizeBuildPosition, type BuildPosition } from './buildPreset'

/** Selects the champion's most-played valid ranked position; ties keep provider order. */
export function selectMainOpggPosition(tierList: OpggTierList, championId: number): BuildPosition | null {
  const champion = tierList.data.find((item) => item.championId === championId)
  if (!champion) return null

  let selected: BuildPosition | null = null
  let selectedPlay = -1
  for (const candidate of champion.positions) {
    const position = normalizeBuildPosition(candidate.name)
    const play = candidate.stats.play
    if (!position || !Number.isFinite(play) || play <= 0 || play <= selectedPlay) continue
    selected = position
    selectedPlay = play
  }
  return selected
}
