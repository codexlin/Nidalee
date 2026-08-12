import { computed, readonly, ref, watch } from 'vue'
import { readVersionedCache, writeVersionedCache, clearVersionedCache } from '@/shared/utils/versionedCache'
import { useCommunityDragonPerksQuery, useGameVersion } from '@/shared/composables/data/useVersionedData'
import type { CommunityDragonPerk } from '@/lib/dataApi'
import { getLolGameDataAssetUrl, getPerkImageUrlFromIconPath } from '@/lib'

export interface PerkStyle {
  id: number
  name: string
  tooltip: string
  iconPath: string
  slots: PerkSlot[]
  allowedSubStyles: number[]
  defaultPerks: number[]
}

export interface PerkSlot {
  type: string
  slotLabel: string
  perks: number[]
}

/** 与 CommunityDragonPerk 对齐的展示用符文 */
export type Perk = Pick<CommunityDragonPerk, 'id' | 'name' | 'tooltip' | 'shortDesc' | 'longDesc' | 'iconPath'>

interface PerkStylesResponse {
  schemaVersion: number
  styles: PerkStyle[]
}

const CD_BASE_URL = 'https://raw.communitydragon.org/latest'
const PERK_STYLES_URL = `${CD_BASE_URL}/plugins/rcp-be-lol-game-data/global/zh_cn/v1/perkstyles.json`
const CACHE_KEY_STYLES = 'nidalee-static-rune-styles'

/**
 * 符文数据：styles 自管；perks 复用 `useCommunityDragonPerksQuery`（单一数据源）
 */
export function useRuneData() {
  const { data: gameVersion } = useGameVersion()
  const perksQuery = useCommunityDragonPerksQuery()
  const perkStyles = ref<PerkStyle[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const perks = computed<Perk[]>(() => perksQuery.data.value ?? [])

  const fetchPerkStyles = async (): Promise<PerkStyle[]> => {
    const response = await fetch(PERK_STYLES_URL)
    if (!response.ok) {
      throw new Error(`获取符文系数据失败: ${response.statusText}`)
    }
    const data: PerkStylesResponse = await response.json()
    return data.styles
  }

  const loadRuneData = async (forceRefresh = false) => {
    const version = gameVersion.value
    // 必须等真实版本，禁止写入 `unknown` 污染缓存
    if (!version) {
      return
    }
    if (isLoading.value) return

    isLoading.value = true
    error.value = null

    try {
      if (!forceRefresh) {
        const cachedStyles = readVersionedCache<PerkStyle[]>(CACHE_KEY_STYLES, version)
        if (cachedStyles?.length) {
          perkStyles.value = cachedStyles
        }
      }

      if (forceRefresh || !perkStyles.value.length) {
        const stylesData = await fetchPerkStyles()
        perkStyles.value = stylesData
        writeVersionedCache(CACHE_KEY_STYLES, version, stylesData)
      }

      // perks：统一走 TanStack Query（可能已由其它组件 hydrate）
      if (forceRefresh || !perksQuery.data.value?.length) {
        await perksQuery.refetch()
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : '未知错误'
      error.value = message
      console.error('[RuneData] 加载失败:', err)
      throw err
    } finally {
      isLoading.value = false
    }
  }

  watch(
    gameVersion,
    (version, prev) => {
      if (version && version !== prev) {
        void loadRuneData(prev !== undefined && prev !== '')
      }
    },
    { immediate: true }
  )

  const getPerkStyleById = (styleId: number): PerkStyle | undefined => perkStyles.value.find((s) => s.id === styleId)

  const getPerkById = (perkId: number): Perk | undefined => perks.value.find((p) => p.id === perkId)

  const getPerkIconUrl = (iconPath: string): string => {
    return getPerkImageUrlFromIconPath(iconPath)
  }

  const getStyleIconUrl = (iconPath: string): string => {
    return getLolGameDataAssetUrl(iconPath)
  }

  const validateRuneConfig = (
    primaryStyleId: number,
    subStyleId: number,
    selectedPerkIds: number[]
  ): { valid: boolean; error?: string } => {
    const primaryStyle = getPerkStyleById(primaryStyleId)
    const subStyle = getPerkStyleById(subStyleId)

    if (!primaryStyle) {
      return { valid: false, error: `主系符文 ${primaryStyleId} 不存在` }
    }
    if (!subStyle) {
      return { valid: false, error: `副系符文 ${subStyleId} 不存在` }
    }
    if (!primaryStyle.allowedSubStyles.includes(subStyleId)) {
      return { valid: false, error: `副系 ${subStyle.name} 不能与主系 ${primaryStyle.name} 搭配` }
    }
    if (selectedPerkIds.length !== 9) {
      return { valid: false, error: `符文数量错误，应该是 9 个，当前是 ${selectedPerkIds.length} 个` }
    }
    for (const perkId of selectedPerkIds) {
      if (!getPerkById(perkId)) {
        return { valid: false, error: `符文 ${perkId} 不存在` }
      }
    }
    return { valid: true }
  }

  const clearCache = () => {
    clearVersionedCache(CACHE_KEY_STYLES)
  }

  return {
    perkStyles: readonly(perkStyles),
    perks,
    isLoading: readonly(isLoading),
    error: readonly(error),
    loadRuneData,
    getPerkStyleById,
    getPerkById,
    getPerkIconUrl,
    getStyleIconUrl,
    validateRuneConfig,
    clearCache
  }
}
