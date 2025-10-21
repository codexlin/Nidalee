import { ref, readonly } from 'vue'

// Community Dragon 符文系数据接口
export interface PerkStyle {
  id: number // 符文系 ID (8000, 8100, 8200, 8300, 8400)
  name: string // 符文系名称 ("精密", "主宰", "巫术", "坚决", "启迪")
  tooltip: string // 描述
  iconPath: string // 图标路径
  slots: PerkSlot[] // 槽位数组
  allowedSubStyles: number[] // 允许的副系 ID
  defaultPerks: number[] // 默认符文配置
}

export interface PerkSlot {
  type: string // 槽位类型 ("kKeyStone", "kMixedRegularSplashable", "kStatMod")
  slotLabel: string // 槽位标签
  perks: number[] // 该槽位的符文 ID 数组
}

// 符文详细信息接口
export interface Perk {
  id: number // 符文 ID
  name: string // 符文名称
  tooltip: string // 长描述
  shortDesc: string // 短描述
  longDesc: string // 完整描述
  iconPath: string // 图标路径
}

// 符文数据响应接口
interface PerkStylesResponse {
  schemaVersion: number
  styles: PerkStyle[]
}

interface PerksResponse {
  perks: Perk[]
}

// Community Dragon CDN 基础 URL
const CD_BASE_URL = 'https://raw.communitydragon.org/latest'
const PERK_STYLES_URL = `${CD_BASE_URL}/plugins/rcp-be-lol-game-data/global/zh_cn/v1/perkstyles.json`
const PERKS_URL = `${CD_BASE_URL}/plugins/rcp-be-lol-game-data/global/zh_cn/v1/perks.json`

// 缓存配置
const CACHE_KEY_STYLES = 'nidalee-rune-styles'
const CACHE_KEY_PERKS = 'nidalee-rune-perks'
const CACHE_DURATION = 7 * 24 * 60 * 60 * 1000 // 7 天

interface CachedData<T> {
  data: T
  timestamp: number
}

/**
 * 符文数据管理 Composable
 */
export function useRuneData() {
  const perkStyles = ref<PerkStyle[]>([])
  const perks = ref<Perk[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  /**
   * 从缓存加载数据
   */
  const loadFromCache = <T>(key: string): T | null => {
    try {
      const cached = localStorage.getItem(key)
      if (!cached) return null

      const { data, timestamp }: CachedData<T> = JSON.parse(cached)

      // 检查是否过期
      if (Date.now() - timestamp > CACHE_DURATION) {
        localStorage.removeItem(key)
        return null
      }

      return data
    } catch (err) {
      console.error(`从缓存加载失败 (${key}):`, err)
      return null
    }
  }

  /**
   * 保存到缓存
   */
  const saveToCache = <T>(key: string, data: T) => {
    try {
      const cached: CachedData<T> = {
        data,
        timestamp: Date.now()
      }
      localStorage.setItem(key, JSON.stringify(cached))
    } catch (err) {
      console.error(`保存到缓存失败 (${key}):`, err)
    }
  }

  /**
   * 从 Community Dragon 获取符文系数据
   */
  const fetchPerkStyles = async (): Promise<PerkStyle[]> => {
    const response = await fetch(PERK_STYLES_URL)
    if (!response.ok) {
      throw new Error(`获取符文系数据失败: ${response.statusText}`)
    }

    const data: PerkStylesResponse = await response.json()
    return data.styles
  }

  /**
   * 从 Community Dragon 获取符文详细信息
   */
  const fetchPerks = async (): Promise<Perk[]> => {
    const response = await fetch(PERKS_URL)
    if (!response.ok) {
      throw new Error(`获取符文详细信息失败: ${response.statusText}`)
    }

    const data: PerksResponse = await response.json()
    return data.perks || data // 兼容不同的响应格式
  }

  /**
   * 加载所有符文数据 (带缓存)
   */
  const loadRuneData = async (forceRefresh = false) => {
    if (isLoading.value) return

    isLoading.value = true
    error.value = null

    try {
      // 1. 尝试从缓存加载
      if (!forceRefresh) {
        const cachedStyles = loadFromCache<PerkStyle[]>(CACHE_KEY_STYLES)
        const cachedPerks = loadFromCache<Perk[]>(CACHE_KEY_PERKS)

        if (cachedStyles && cachedPerks) {
          perkStyles.value = cachedStyles
          perks.value = cachedPerks
          console.log('从缓存加载符文数据成功')
          isLoading.value = false
          return
        }
      }

      // 2. 从 Community Dragon 获取
      console.log('从 Community Dragon 获取符文数据...')
      const [stylesData, perksData] = await Promise.all([fetchPerkStyles(), fetchPerks()])

      perkStyles.value = stylesData
      perks.value = perksData

      // 3. 保存到缓存
      saveToCache(CACHE_KEY_STYLES, stylesData)
      saveToCache(CACHE_KEY_PERKS, perksData)

      console.log('符文数据加载成功:', {
        styles: stylesData.length,
        perks: perksData.length
      })
    } catch (err) {
      const message = err instanceof Error ? err.message : '未知错误'
      error.value = message
      console.error('加载符文数据失败:', err)
      throw err
    } finally {
      isLoading.value = false
    }
  }

  /**
   * 根据 ID 查找符文系
   */
  const getPerkStyleById = (styleId: number): PerkStyle | undefined => {
    return perkStyles.value.find((s) => s.id === styleId)
  }

  /**
   * 根据 ID 查找符文
   */
  const getPerkById = (perkId: number): Perk | undefined => {
    return perks.value.find((p) => p.id === perkId)
  }

  /**
   * 获取符文图标完整 URL
   */
  const getPerkIconUrl = (iconPath: string): string => {
    if (iconPath.startsWith('http')) {
      return iconPath
    }
    return `${CD_BASE_URL}${iconPath}`
  }

  /**
   * 获取符文系图标 URL
   */
  const getStyleIconUrl = (iconPath: string): string => {
    if (iconPath.startsWith('http')) {
      return iconPath
    }
    return `${CD_BASE_URL}${iconPath}`
  }

  /**
   * 验证符文配置是否有效
   */
  const validateRuneConfig = (
    primaryStyleId: number,
    subStyleId: number,
    selectedPerkIds: number[]
  ): { valid: boolean; error?: string } => {
    // 1. 检查符文系是否存在
    const primaryStyle = getPerkStyleById(primaryStyleId)
    const subStyle = getPerkStyleById(subStyleId)

    if (!primaryStyle) {
      return { valid: false, error: `主系符文 ${primaryStyleId} 不存在` }
    }

    if (!subStyle) {
      return { valid: false, error: `副系符文 ${subStyleId} 不存在` }
    }

    // 2. 检查副系是否在允许列表中
    if (!primaryStyle.allowedSubStyles.includes(subStyleId)) {
      return { valid: false, error: `副系 ${subStyle.name} 不能与主系 ${primaryStyle.name} 搭配` }
    }

    // 3. 检查符文数量 (应该是 9 个)
    if (selectedPerkIds.length !== 9) {
      return { valid: false, error: `符文数量错误，应该是 9 个，当前是 ${selectedPerkIds.length} 个` }
    }

    // 4. 检查所有符文是否存在
    for (const perkId of selectedPerkIds) {
      const perk = getPerkById(perkId)
      if (!perk) {
        return { valid: false, error: `符文 ${perkId} 不存在` }
      }
    }

    return { valid: true }
  }

  /**
   * 清除缓存
   */
  const clearCache = () => {
    localStorage.removeItem(CACHE_KEY_STYLES)
    localStorage.removeItem(CACHE_KEY_PERKS)
    console.log('符文数据缓存已清除')
  }

  return {
    // 状态
    perkStyles: readonly(perkStyles),
    perks: readonly(perks),
    isLoading: readonly(isLoading),
    error: readonly(error),

    // 方法
    loadRuneData,
    getPerkStyleById,
    getPerkById,
    getPerkIconUrl,
    getStyleIconUrl,
    validateRuneConfig,
    clearCache
  }
}
