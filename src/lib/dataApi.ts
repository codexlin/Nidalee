import { getLatestVersion } from '@/lib'
import { useApiFetch, type ApiResponse } from './api/httpClient'

export { useApiFetch, type ApiResponse } from './api/httpClient'
export {
  fetchHextechChampionDetail,
  fetchHextechTierList,
  fetchOpggChampionBuild,
  fetchOpggTierList
} from './api/externalBuilds'

// =============================================================================
// Data Dragon / Community Dragon 类型（仍被 UI / index 使用）
// =============================================================================

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
// 仍有调用方的 HTTP fetch
// =============================================================================

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
 * 获取 Community Dragon 符文数据
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
