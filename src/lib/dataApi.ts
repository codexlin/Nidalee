import { getLatestVersion } from '@/lib'
import { useApiFetch, type ApiResponse } from './api/httpClient'

export { useApiFetch, type ApiResponse } from './api/httpClient'
export {
  applyOpggRunes,
  fetchHextechChampionDetail,
  fetchHextechTierList,
  fetchOpggChampionBuild,
  fetchOpggChampionBuildRaw,
  fetchOpggChampionPositions,
  fetchOpggTierList
} from './api/externalBuilds'

// 官方Data Dragon API类型定义
export interface DDragonVersions extends Array<string> {}

export interface DDragonRune {
  id: number
  key: string
  icon: string
  name: string
  shortDesc: string
  longDesc: string
}

export interface DDragonRuneTree {
  id: number
  key: string
  icon: string
  name: string
  slots: Array<{
    runes: DDragonRune[]
  }>
}

export interface DDragonItem {
  name: string
  description: string
  colloq: string
  plaintext: string
  into?: string[]
  from?: string[]
  image: {
    full: string
    sprite: string
    group: string
    x: number
    y: number
    w: number
    h: number
  }
  gold: {
    base: number
    purchasable: boolean
    total: number
    sell: number
  }
  tags: string[]
  maps: Record<string, boolean>
  stats: Record<string, number>
}

export interface DDragonItemsResponse {
  type: string
  format: string
  version: string
  data: Record<string, DDragonItem>
}

export interface DDragonChampion {
  version: string
  id: string
  key: string
  name: string
  title: string
  blurb: string
  info: {
    attack: number
    defense: number
    magic: number
    difficulty: number
  }
  image: {
    full: string
    sprite: string
    group: string
    x: number
    y: number
    w: number
    h: number
  }
  tags: string[]
  partype: string
  stats: Record<string, number>
}

export interface DDragonChampionsResponse {
  type: string
  format: string
  version: string
  data: Record<string, DDragonChampion>
}

// Community Dragon API类型定义
export interface CommunityDragonChampionSummary {
  id: number
  name: string
  description: string
  alias: string
  contentId: string
  squarePortraitPath: string
  roles: string[]
}

// Community Dragon 符文数据类型
export interface CommunityDragonPerk {
  id: number
  name: string
  majorChangePatchVersion: string
  tooltip: string
  shortDesc: string
  longDesc: string
  recommendationDescriptor: string
  iconPath: string
  endOfGameStatDescs: string[]
  recommendationDescriptorAttributes: Record<string, unknown>
}

export interface CommunityDragonSkin {
  id: number
  isBase: boolean
  name: string
  splashPath: string
  uncenteredSplashPath: string
  tilePath: string
  loadScreenPath: string
  skinType: string
  rarity: string
  isLegacy: boolean
  chromas: Array<{
    id: number
    name: string
    chromaPath: string
    colors: string[]
  }>
  questSkinInfo?: unknown
  description?: string
  regionRarityId?: number
  rarityGemPath?: string
}

export interface CommunityDragonChampion {
  id: number
  name: string
  alias: string
  squarePortraitPath: string
  roles: string[]
  skins: CommunityDragonSkin[]
  passive: {
    name: string
    abilityIconPath: string
    description: string
  }
  spells: Array<{
    spellKey: string
    name: string
    abilityIconPath: string
    description: string
    dynamicDescription: string
    range: number[]
    cooldown: number[]
    cost: number[]
    costType: string
    maxLevel: number
  }>
}

/** Riot 官方 docs 队列结构（英文） */
export interface QueueInfo {
  queueId: number
  map: string
  description: string
  notes?: string
}

/** Community Dragon 客户端队列结构（中文） */
export interface CDragonQueue {
  id: number
  name: string
  shortName?: string
  description?: string
  detailedDescription?: string
  gameSelectModeGroup?: string
  gameSelectCategory?: string
}

// =============================================================================
// 官方 Data Dragon API 调用函数
// =============================================================================

/**
 * 获取版本列表
 */
export async function fetchVersions(): Promise<ApiResponse<DDragonVersions>> {
  try {
    const { data, error, statusCode } = await useApiFetch(
      'https://ddragon.leagueoflegends.com/api/versions.json'
    ).json<DDragonVersions>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: data.value[0] // 最新版本
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

/**
 * 获取符文数据
 */
export async function fetchRunes(version?: string): Promise<ApiResponse<DDragonRuneTree[]>> {
  try {
    const gameVersion = version || (await getLatestVersion())
    const url = `https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/zh_CN/runesReforged.json`

    const { data, error, statusCode } = await useApiFetch(url).json<DDragonRuneTree[]>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: gameVersion
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

/**
 * 获取装备数据
 */
export async function fetchItems(version?: string): Promise<ApiResponse<DDragonItemsResponse>> {
  try {
    const gameVersion = version || (await getLatestVersion())
    const url = `https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/zh_CN/item.json`

    const { data, error, statusCode } = await useApiFetch(url).json<DDragonItemsResponse>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: gameVersion
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

/**
 * 获取英雄列表数据
 */
export async function fetchChampions(version?: string): Promise<ApiResponse<DDragonChampionsResponse>> {
  try {
    const gameVersion = version || (await getLatestVersion())
    const url = `https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/zh_CN/champion.json`

    const { data, error, statusCode } = await useApiFetch(url).json<DDragonChampionsResponse>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: gameVersion
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

/** Community Dragon 召唤师技能（含 iconPath） */
export interface CommunityDragonSummonerSpell {
  id: number
  name: string
  description: string
  summonerLevel: number
  cooldown: number
  gameModes: string[]
  iconPath: string
}

/**
 * 获取召唤师技能数据（Data Dragon，旧格式）
 */
export async function fetchSummonerSpells(version?: string): Promise<ApiResponse<unknown>> {
  try {
    const gameVersion = version || (await getLatestVersion())
    const url = `https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/zh_CN/summoner.json`

    const { data, error, statusCode } = await useApiFetch(url).json<unknown>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: gameVersion
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

/**
 * Community Dragon 召唤师技能列表（id + iconPath，与后端 load_summoner_spell_data 同源）
 */
export async function fetchCommunityDragonSummonerSpells(): Promise<ApiResponse<CommunityDragonSummonerSpell[]>> {
  try {
    const url =
      'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/summoner-spells.json'

    const { data, error, statusCode } = await useApiFetch(url).json<CommunityDragonSummonerSpell[]>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

// =============================================================================
// Community Dragon API 调用函数
// =============================================================================

/**
 * 获取皮肤数据
 */
export async function fetchSkins(): Promise<ApiResponse<CommunityDragonSkin[]>> {
  try {
    const url = 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/skins.json'

    const { data, error, statusCode } = await useApiFetch(url).json<CommunityDragonSkin[]>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: 'latest'
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

/**
 * 获取队列信息（Community Dragon 中文，跟客户端一致）
 */
export async function fetchQueues(): Promise<ApiResponse<CDragonQueue[]>> {
  try {
    const url = 'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/queues.json'

    const { data, error, statusCode } = await useApiFetch(url).json<CDragonQueue[]>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: 'cdragon-zh_cn'
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

/**
 * 获取指定英雄的详细信息
 */
export async function fetchChampionDetails(championId: number): Promise<ApiResponse<CommunityDragonChampion>> {
  try {
    const url = `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/champions/${championId}.json`

    const { data, error, statusCode } = await useApiFetch(url).json<CommunityDragonChampion>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: 'latest'
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

/**
 * 获取英雄摘要数据（包含头像路径）
 */
export async function fetchChampionSummary(): Promise<ApiResponse<CommunityDragonChampionSummary[]>> {
  try {
    const url =
      'https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/champion-summary.json'

    const { data, error, statusCode } = await useApiFetch(url).json<CommunityDragonChampionSummary[]>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: 'latest'
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

/**
 * 获取Community Dragon符文数据
 */
export async function fetchCommunityDragonPerks(version?: string): Promise<ApiResponse<CommunityDragonPerk[]>> {
  try {
    const gameVersion = version || (await getLatestVersion())
    const url = `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/zh_cn/v1/perks.json`

    const { data, error, statusCode } = await useApiFetch(url).json<CommunityDragonPerk[]>()

    if (error.value) {
      throw new Error(error.value)
    }

    if (statusCode.value !== 200) {
      throw new Error(`HTTP ${statusCode.value}`)
    }

    if (!data.value) {
      throw new Error('No data received')
    }

    return {
      success: true,
      data: data.value,
      version: gameVersion
    }
  } catch (error) {
    return {
      success: false,
      data: null,
      error: error instanceof Error ? error.message : 'Unknown error'
    }
  }
}

// =============================================================================
// 官方 Data Dragon 及 Riot 静态数据 API 调用函数补全
// =============================================================================

/**
 * 地图数据类型
 */
export interface RiotMap {
  mapId: number
  mapName: string
  notes?: string
  description?: string
}

/**
 * 游戏模式类型
 */
export interface RiotGameMode {
  gameMode: string
  description: string
}

/**
 * 游戏类型类型
 */
export interface RiotGameType {
  gametype: string
  description: string
}

/**
 * 获取地图数据
 */
export async function fetchMaps(): Promise<ApiResponse<RiotMap[]>> {
  try {
    const url = 'https://static.developer.riotgames.com/docs/lol/maps.json'
    const { data, error, statusCode } = await useApiFetch(url).json<RiotMap[]>()
    if (error.value) throw new Error(error.value)
    if (statusCode.value !== 200) throw new Error(`HTTP ${statusCode.value}`)
    if (!data.value) throw new Error('No data received')
    return { success: true, data: data.value, version: 'static' }
  } catch (error) {
    return { success: false, data: null, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

/**
 * 获取游戏模式数据
 */
export async function fetchGameModes(): Promise<ApiResponse<RiotGameMode[]>> {
  try {
    const url = 'https://static.developer.riotgames.com/docs/lol/gameModes.json'
    const { data, error, statusCode } = await useApiFetch(url).json<RiotGameMode[]>()
    if (error.value) throw new Error(error.value)
    if (statusCode.value !== 200) throw new Error(`HTTP ${statusCode.value}`)
    if (!data.value) throw new Error('No data received')
    return { success: true, data: data.value, version: 'static' }
  } catch (error) {
    return { success: false, data: null, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}

/**
 * 获取游戏类型数据
 */
export async function fetchGameTypes(): Promise<ApiResponse<RiotGameType[]>> {
  try {
    const url = 'https://static.developer.riotgames.com/docs/lol/gameTypes.json'
    const { data, error, statusCode } = await useApiFetch(url).json<RiotGameType[]>()
    if (error.value) throw new Error(error.value)
    if (statusCode.value !== 200) throw new Error(`HTTP ${statusCode.value}`)
    if (!data.value) throw new Error('No data received')
    return { success: true, data: data.value, version: 'static' }
  } catch (error) {
    return { success: false, data: null, error: error instanceof Error ? error.message : 'Unknown error' }
  }
}
