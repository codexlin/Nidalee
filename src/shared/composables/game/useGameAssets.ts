export function useGameAssets() {
  const gameVersion = ref('14.23.1')

  const getProfileIconUrl = (iconId: number): string => {
    if (!iconId) return ''
    return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/profile-icons/${iconId}.jpg`
  }

  const getChampionIconUrl = (championId: number): string => {
    if (!championId) return ''
    return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${championId}.png`
  }

  const getItemIconUrl = (itemId: unknown): string => {
    if (!itemId || typeof itemId !== 'number' || itemId === 0) return ''
    return `https://ddragon.leagueoflegends.com/cdn/${gameVersion.value}/img/item/${itemId}.png`
  }

  const getRankIconUrl = (tier: string): string => {
    if (!tier) return ''
    const tierLower = tier.toLowerCase().trim()
    if (tierLower === 'none') return ''
    const file = tierLower === 'unranked' ? 'unranked-tft.svg' : `${tierLower}_tft.svg`
    return `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/images/ranked-mini-crests/${file}`
  }

  const getPerkStyleIconUrl = (styleId: number | undefined): string => {
    if (!styleId) return ''
    return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/styles/icon/7200_${styleId}.png`
  }

  const getPerkIconUrl = (perkId: number | undefined): string => {
    if (!perkId) return ''
    return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/${perkId}.png`
  }

  const setGameVersion = (version: string) => {
    gameVersion.value = version
  }

  return {
    gameVersion,
    getProfileIconUrl,
    getChampionIconUrl,
    getItemIconUrl,
    getRankIconUrl,
    getPerkStyleIconUrl,
    getPerkIconUrl,
    setGameVersion
  }
}
