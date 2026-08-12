// 数据API模块
export * from './dataApi'
import type { CommunityDragonPerk, DDragonChampionsResponse, DDragonItemsResponse } from './dataApi'
import { getMapById, getQueueDisplayName } from '@/common'

// 主题配置模块
export * from './theme'
export * from './themeColor'

// 其他辅助函数
export const getPlayerProfileIcon = (participantId: number, gameDetail: GameDetail): number => {
  const identity = gameDetail.participants?.find((id) => id.participantId === participantId)
  // 兼容 profileIconId 可能为 bigint 的情况，强制转换为 number
  if (identity && identity.profileIconId !== undefined && identity.profileIconId !== null) {
    return Number(identity.profileIconId)
  }
  return 0
}

// 游戏相关的工具函数
export const formatGameMode = (mode: string): string => {
  const modeMap: Record<string, string> = {
    CLASSIC: '经典模式',
    ARAM: '大乱斗',
    URF: '无限火力',
    TUTORIAL: '教程',
    ONEFORALL: '克隆大作战',
    ARSR: '极地大乱斗',
    PRACTICETOOL: '训练工具',
    NEXUSBLITZ: '极地大乱斗'
  }
  return modeMap[mode] || mode
}

export const formatDuration = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
}

export const getQueueName = (queueId: number): string => {
  // CDragon 中文名优先，本地兜底表其次
  return getQueueDisplayName(queueId)
}

export const getMapName = (mapId: number): string => {
  // 优先使用地图定义
  const map = getMapById(mapId)
  return map ? map.name : '未知地图'
}

export const formatNumber = (num: number): string => {
  return num?.toLocaleString() || '0'
}

/**
 * 根据英雄ID获取英雄图标URL（Community Dragon）
 * @param championId 英雄ID，number或string
 * @returns 英雄图标URL
 */
export const getChampionIconUrl = (championId: number | string | null): string => {
  if (!championId) return ''
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${championId}.png`
}

/** 特殊模式变体别名前缀（CDragon / DDragon 会混入，中文名与本体相同） */
const MODE_CHAMPION_ALIAS_RE = /^(Jade_|Ruby_)/i

/** 正式召唤师峡谷英雄：排除模式变体（id≥10000 或 Jade_/Ruby_ 前缀） */
export const isStandardChampionId = (id: number): boolean => Number.isFinite(id) && id > 0 && id < 10000

export const isStandardChampionAlias = (alias: string): boolean => !!alias && !MODE_CHAMPION_ALIAS_RE.test(alias)

/** Community Dragon / LCU 摘要条目 */
export const isStandardChampion = (c: { id: number; alias?: string }): boolean =>
  isStandardChampionId(c.id) && (!c.alias || isStandardChampionAlias(c.alias))

/** Data Dragon champion.json 条目（`id` 为别名，`key` 为数字 ID） */
export const isStandardDDragonChampion = (c: { id: string; key: string | number }): boolean =>
  isStandardChampionId(Number(c.key)) && isStandardChampionAlias(String(c.id))
// 处理 Community Dragon 路径
export const getCommunityDragonUrl = (path: string): string => {
  if (!path) return ''
  // 移除开头的斜杠并详细完整URL
  const cleanPath = path.startsWith('/') ? path.slice(1) : path
  return `https://raw.communitydragon.org/latest/plugins/${cleanPath}`
}

/**
 * LCU / CDragon `iconPath` → 可访问的 Community Dragon CDN URL
 *
 * `/lol-game-data/assets/DATA/Spells/Icons2D/Summoner_flash.png`
 * → `.../rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_flash.png`
 */
export const getLolGameDataAssetUrl = (iconPath: string): string => {
  if (!iconPath) return ''
  if (iconPath.startsWith('http')) return iconPath

  const marker = '/lol-game-data/assets/'
  const idx = iconPath.indexOf(marker)
  if (idx >= 0) {
    const rest = iconPath.slice(idx + marker.length)
    return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/${rest.toLowerCase()}`
  }

  return ''
}

/** 符文 iconPath → CDN（复用通用资产路径转换） */
export const getPerkImageUrlFromIconPath = (iconPath: string, fallbackId?: number): string => {
  const fromPath = getLolGameDataAssetUrl(iconPath)
  if (fromPath) return fromPath
  if (fallbackId === null || fallbackId === undefined) return ''
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/${fallbackId}.png`
}

// 根据符文ID获取符文图标URL（使用Community Dragon）
export const getPerkIconUrlByCommunityDragon = (perkId: number, perks: CommunityDragonPerk[]): string => {
  const perk = perks.find((p) => p.id === perkId)
  return getPerkImageUrlFromIconPath(perk?.iconPath ?? '', perkId)
}

/** 响应式身份目录（英雄 / 召唤师技能）— 实现见 `@/shared/staticCatalog` */
export {
  setChampionCatalog,
  setSummonerSpellCatalog,
  getSummonerSpellCatalogEntry,
  getChampionName,
  resolveChampionName,
  type SummonerSpellCatalogEntry
} from '@/shared/staticCatalog'
import { getSummonerSpellCatalogEntry } from '@/shared/staticCatalog'

/**
 * 根据玩家头像ID获取头像URL
 * @param iconId 头像ID
 * @returns 头像URL
 */
export const getProfileIconUrl = (iconId: number): string => {
  if (!iconId) return ''
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/profile-icons/${iconId}.jpg`
}

/**
 * 根据物品ID获取物品图标URL
 * @param itemId 物品ID
 * @param gameVersion 游戏版本
 * @returns 物品图标URL
 * @example
 * import { getItemIconUrl } from '@/lib'
 * const url = getItemIconUrl(3135, '12.23.1')
 * // https://ddragon.leagueoflegends.com/cdn/12.23.1/img/item/3135.png
 */
export const getItemIconUrl = (itemId: number, gameVersion: string): string => {
  if (!itemId || itemId === 0)
    return 'data:image/svg+xml;utf8,<svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg"><rect width="32" height="32" rx="6" fill="%23e5e7eb"/><text x="16" y="22" text-anchor="middle" font-size="20" fill="%239ca3af" font-family="Arial, Helvetica, sans-serif">?</text></svg>'
  return `https://ddragon.leagueoflegends.com/cdn/${gameVersion}/img/item/${itemId}.png`
}
/**
 * 根据物品ID获取物品图标URL (腾讯CDN)
 * @param itemId 物品ID
 * @param gameVersion 游戏版本
 * @returns 物品图标URL
 * @example
 * import { getItemIconByCdnUrl } from '@/lib'
 * const url = getItemIconByCdnUrl(3135, '12.23.1')
 * // https://game.gtimg.cn/images/lol/act/img/item/3135.png
 */
export const getItemIconByCdnUrl = (itemId: number): string => {
  if (!itemId || itemId === 0)
    return 'data:image/svg+xml;utf8,<svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg"><rect width="32" height="32" rx="6" fill="%23e5e7eb"/><text x="16" y="22" text-anchor="middle" font-size="20" fill="%239ca3af" font-family="Arial, Helvetica, sans-serif">?</text></svg>'
  return `https://game.gtimg.cn/images/lol/act/img/item/${itemId}.png`
}

/**
 * 段位小图标（Community Dragon ranked-mini-crests，TFT 套更完整，含 emerald）
 *
 * 注意：`ranked-emblem/emblem-*.png` 是带大片黑底的宽幅展示图，不适合 UI 图标。
 * @example getRankIconUrl('GOLD')
 * // .../images/ranked-mini-crests/gold_tft.svg
 */
export const getRankIconUrl = (tier: string): string => {
  if (!tier) return ''
  const tierLower = tier.toLowerCase().trim()
  if (tierLower === 'none') return ''
  // unranked 文件名是连字符：unranked-tft.svg；其余为 {tier}_tft.svg
  const file = tierLower === 'unranked' ? 'unranked-tft.svg' : `${tierLower}_tft.svg`
  return `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/images/ranked-mini-crests/${file}`
}

/**
 * 挑战水晶小图标（Community Dragon challenge-mini-crystal）
 * 资源无 emerald；遇到时回退到 platinum。
 */
export const getChallengeCrystalIconUrl = (level: string | null | undefined): string => {
  if (!level) return ''
  let tier = level.toLowerCase().trim()
  if (!tier || tier === 'none' || tier === 'unranked') return ''
  // challenge-mini-crystal 目录没有 emerald.svg
  if (tier === 'emerald') tier = 'platinum'
  return `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/challenge-mini-crystal/${tier}.svg`
}

/**
 * 分路 / 位置图标（Community Dragon honor/roleicon_*）
 * 兼容后端码（TOP/MID/ADC/SUPPORT）与 LCU 码（MIDDLE/BOTTOM/UTILITY）
 */
export const getRoleIconUrl = (position: string | null | undefined): string => {
  if (!position) return ''
  const key = position.toUpperCase().trim()
  const roleMap: Record<string, string> = {
    TOP: 'top',
    JUNGLE: 'jungle',
    MID: 'middle',
    MIDDLE: 'middle',
    ADC: 'bottom',
    BOTTOM: 'bottom',
    SUPPORT: 'utility',
    UTILITY: 'utility'
  }
  const role = roleMap[key]
  if (!role) return ''
  return `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/honor/roleicon_${role}.png`
}

// 时间相关函数
export const formatRelativeTime = (timestamp: number): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const hours = Math.floor(diff / (1000 * 60 * 60))

  if (hours < 1) {
    return '刚刚'
  } else if (hours < 24) {
    return `${hours}小时前`
  } else {
    const days = Math.floor(hours / 24)
    return `${days}天前`
  }
}

// 游戏数据处理函数
export const getTeamResult = (teamId: string, teams: TeamInfo[]): string => {
  if (!teams) return ''
  const team = teams.find((t) => t.teamId?.toString() === teamId)
  if (!team) return ''
  return team.win === 'Win' ? '胜方' : '败方'
}

export const getTeamParticipants = (teamId: string, gameDetail: GameDetail): ParticipantInfo[] => {
  if (!gameDetail?.participants) return []
  return gameDetail.participants.filter((p) => p.teamId.toString() === teamId)
}

export const getTeamBans = (teamId: string, teams: TeamInfo[]): BanInfo[] => {
  if (!teams) return []
  const team = teams.find((t) => t.teamId?.toString() === teamId)
  return team?.bans || []
}

// 兼容旧版 Riot API 结构（不与当前 GameDetail 一一对应），仅供本函数内部使用
interface LegacyGameDetail {
  participantIdentities?: Array<{
    participantId: number
    player?: { gameName?: string; tagLine?: string; summonerName?: string }
  }>
}

export const getPlayerDisplayName = (participantId: number, gameDetail: LegacyGameDetail): string => {
  const identity = gameDetail.participantIdentities?.find((id) => id.participantId === participantId)
  if (!identity?.player) return '未知玩家'

  const { gameName, tagLine, summonerName } = identity.player
  if (gameName && tagLine) {
    return `${gameName}#${tagLine}`
  }
  return summonerName || '未知玩家'
}

/** 召唤师技能图标：目录里的 iconPath → CDN（需先 setSummonerSpellCatalog） */
export const getSpellIconUrl = (spellId: number | null): string => {
  if (!spellId) return ''
  const entry = getSummonerSpellCatalogEntry(Number(spellId))
  return entry ? getLolGameDataAssetUrl(entry.iconPath) : ''
}

/** 召唤师技能名称 + 图标（数据来自 Rust 静态目录 IPC） */
export const getSpellMeta = (spellId: number | bigint | null): { label: string; icon: string } => {
  if (spellId === null || spellId === undefined) return { label: '', icon: '' }
  const id = Number(spellId)
  if (!id) return { label: '', icon: '' }
  const entry = getSummonerSpellCatalogEntry(id)
  if (!entry) return { label: `技能${id}`, icon: '' }
  return {
    label: entry.name,
    icon: getLolGameDataAssetUrl(entry.iconPath)
  }
}
/** 段位图标（Community Dragon，等同 getRankIconUrl） */
export const getTierIconUrl = (tier: string | undefined): string => {
  if (!tier) return ''
  return getRankIconUrl(tier)
}
// 获取最新版本号
export const getLatestVersion = async () => {
  const response = await fetch('https://ddragon.leagueoflegends.com/api/versions.json')
  const versions = await response.json()
  const latestVersion = versions[0]
  return latestVersion || '15.12.1'
}
// 获取所有符文详细信息（含描述等）
export const getAllRunes = async (gameVersion: string, language: string = 'zh_CN') => {
  const resp = await fetch(`https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/runesReforged.json`)
  return await resp.json()
}

// 获取所有召唤师技能详细信息
export const getAllSummonerSpells = async (gameVersion: string, language: string = 'zh_CN') => {
  const resp = await fetch(`https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/summoner.json`)
  return await resp.json()
}
// 获取所有英雄基础信息
export const getAllChampions = async (
  gameVersion: string,
  language: string = 'zh_CN'
): Promise<DDragonChampionsResponse> => {
  const resp = await fetch(`https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/champion.json`)
  return await resp.json()
}

// 获取单个英雄详细信息
export const getChampionDetail = async (
  championName: string,
  gameVersion: string,
  language: string = 'zh_CN'
): Promise<unknown> => {
  const resp = await fetch(
    `https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/champion/${championName}.json`
  )
  return await resp.json()
}
// 获取单个英雄详细信息根据id
export const getChampionInfoById = async (
  championId: number,
  gameVersion: string,
  language: string = 'zh_CN'
): Promise<unknown> => {
  const resp = await fetch(
    `https://raw.communitydragon.org/${gameVersion}/plugins/rcp-be-lol-game-data/global/${language}/v1/champions/${championId}.json`
  )
  return await resp.json()
}
// 获取所有物品数据
export const getAllItems = async (gameVersion: string, language: string = 'zh_CN'): Promise<DDragonItemsResponse> => {
  const resp = await fetch(`https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/item.json`)
  return await resp.json()
}
