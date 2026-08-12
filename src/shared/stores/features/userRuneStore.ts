import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { load } from '@tauri-apps/plugin-store'
import { isOpggTier, type OpggTier } from '@/shared/utils/opggTier'

// 符文配置接口
export interface RuneConfig {
  id: string // UUID
  name: string // "劫-中路-电刑"
  championId: number | null // null = 位置通用符文
  championName: string | null
  position: string | null // null = 英雄通用符文 ("TOP" | "JUNGLE" | "MID" | "ADC" | "SUPPORT")

  // 符文数据
  primaryStyleId: number // 主系 ID (8000, 8100, 8200, 8300, 8400)
  subStyleId: number // 副系 ID
  selectedPerkIds: number[] // 选中的符文 ID 数组 (9个: 4主+2副+3碎片)

  // 元数据
  isDefault: boolean // 是否为此英雄+位置的默认配置
  source: 'opgg' | 'custom' | 'import' // 来源
  createdAt: number // 创建时间戳
  updatedAt: number // 更新时间戳
  usageCount: number // 使用次数

  // 作用域标识
  scope: 'champion-position' | 'champion-all' | 'position-all'
}

// 自动应用配置
export interface AutoApplyConfig {
  enabled: boolean // 是否启用自动应用
  strategy: 'auto' | 'opgg' | 'custom' // 优先级策略
  opggTier: OpggTier // OP.GG API 使用的规范段位值
  showToast: boolean // 是否显示应用成功提示
}

type PersistedAutoApplyConfig = Partial<Omit<AutoApplyConfig, 'opggTier'>> & {
  opggTier?: unknown
}

const createDefaultAutoApply = (): AutoApplyConfig => ({
  enabled: false,
  strategy: 'auto',
  opggTier: 'diamond_plus',
  showToast: true
})

// Store 数据结构
interface StoreData {
  configs: RuneConfig[]
  autoApply: AutoApplyConfig
}

export const useUserRuneStore = defineStore('userRune', () => {
  // Tauri Store 实例（延迟初始化）
  let tauriStore: Awaited<ReturnType<typeof load>> | null = null

  // 状态
  const configs = ref<RuneConfig[]>([])
  const autoApply = ref<AutoApplyConfig>(createDefaultAutoApply())

  const isLoaded = ref(false)

  // 计算属性
  const configCount = computed(() => configs.value.length)

  // 从 Tauri Store 加载数据
  const loadFromStore = async () => {
    try {
      // 初始化 Store
      if (!tauriStore) {
        tauriStore = await load('rune-configs.json')
      }

      const storedConfigs = await tauriStore.get<RuneConfig[]>('configs')
      const storedAutoApply = await tauriStore.get<PersistedAutoApplyConfig>('autoApply')

      if (storedConfigs) {
        configs.value = storedConfigs
      }

      if (storedAutoApply) {
        const opggTier = isOpggTier(storedAutoApply.opggTier)
          ? storedAutoApply.opggTier
          : createDefaultAutoApply().opggTier
        autoApply.value = {
          ...createDefaultAutoApply(),
          ...storedAutoApply,
          opggTier
        }

        if (storedAutoApply.opggTier !== opggTier) {
          await tauriStore.set('autoApply', autoApply.value)
          await tauriStore.save()
        }
      }

      isLoaded.value = true
    } catch (error) {
      console.error('从 Tauri Store 加载符文配置失败:', error)
      isLoaded.value = true
    }
  }

  // 保存到 Tauri Store
  const saveToStore = async () => {
    try {
      // 确保 Store 已初始化
      if (!tauriStore) {
        tauriStore = await load('rune-configs.json')
      }

      await tauriStore.set('configs', configs.value)
      await tauriStore.set('autoApply', autoApply.value)
      await tauriStore.save() // 立即保存到磁盘
    } catch (error) {
      console.error('保存符文配置到 Tauri Store 失败:', error)
      throw error
    }
  }

  // CRUD 操作

  /**
   * 添加新配置
   */
  const addConfig = async (config: RuneConfig) => {
    configs.value.push(config)
    await saveToStore()
  }

  /**
   * 更新配置
   */
  const updateConfig = async (id: string, updates: Partial<RuneConfig>) => {
    const index = configs.value.findIndex((c) => c.id === id)
    if (index === -1) {
      throw new Error(`配置 ID ${id} 不存在`)
    }

    configs.value[index] = {
      ...configs.value[index],
      ...updates,
      updatedAt: Date.now()
    }

    await saveToStore()
  }

  /**
   * 删除配置
   */
  const deleteConfig = async (id: string) => {
    const index = configs.value.findIndex((c) => c.id === id)
    if (index === -1) {
      throw new Error(`配置 ID ${id} 不存在`)
    }

    configs.value.splice(index, 1)
    await saveToStore()
  }

  /**
   * 设置为默认配置
   * 逻辑：同一 championId + position 组合只能有一个默认配置
   */
  const setAsDefault = async (id: string) => {
    const config = configs.value.find((c) => c.id === id)
    if (!config) {
      throw new Error(`配置 ID ${id} 不存在`)
    }

    // 取消同一组合的其他默认配置
    configs.value.forEach((c) => {
      if (c.championId === config.championId && c.position === config.position && c.id !== id) {
        c.isDefault = false
      }
    })

    // 设置当前配置为默认
    config.isDefault = true
    config.updatedAt = Date.now()

    await saveToStore()
  }

  /**
   * 增加使用次数
   */
  const incrementUsageCount = async (id: string) => {
    const config = configs.value.find((c) => c.id === id)
    if (config) {
      config.usageCount++
      config.updatedAt = Date.now()
      await saveToStore()
    }
  }

  // 查询操作

  /**
   * 根据 championId 和 position 查找最佳匹配的符文配置
   * 优先级：
   * 1. 英雄+位置专属 + 默认
   * 2. 英雄+位置专属
   * 3. 英雄通用 + 默认
   * 4. 英雄通用
   * 5. 位置通用 + 默认
   * 6. 位置通用
   * 7. null (无匹配，需要回退到 OP.GG)
   */
  const findBestMatch = (championId: number, position?: string): RuneConfig | null => {
    // 1. 精确匹配：英雄 + 位置 + 默认
    let match = configs.value.find((c) => c.championId === championId && c.position === position && c.isDefault)
    if (match) return match

    // 2. 精确匹配：英雄 + 位置
    match = configs.value.find((c) => c.championId === championId && c.position === position)
    if (match) return match

    // 3. 英雄通用 + 默认
    match = configs.value.find((c) => c.championId === championId && c.position === null && c.isDefault)
    if (match) return match

    // 4. 英雄通用
    match = configs.value.find((c) => c.championId === championId && c.position === null)
    if (match) return match

    // 5. 位置通用 + 默认
    if (position) {
      match = configs.value.find((c) => c.championId === null && c.position === position && c.isDefault)
      if (match) return match
    }

    // 6. 位置通用
    if (position) {
      match = configs.value.find((c) => c.championId === null && c.position === position)
      if (match) return match
    }

    // 7. 无匹配
    return null
  }

  /**
   * 获取指定英雄的所有配置
   */
  const getConfigsByChampion = (championId: number): RuneConfig[] => {
    return configs.value.filter((c) => c.championId === championId)
  }

  /**
   * 获取指定位置的所有配置
   */
  const getConfigsByPosition = (position: string): RuneConfig[] => {
    return configs.value.filter((c) => c.position === position)
  }

  /**
   * 根据 ID 获取配置
   */
  const getConfigById = (id: string): RuneConfig | undefined => {
    return configs.value.find((c) => c.id === id)
  }

  /**
   * 更新自动应用配置
   */
  const updateAutoApply = async (updates: Partial<AutoApplyConfig>) => {
    autoApply.value = {
      ...autoApply.value,
      ...updates
    }
    await saveToStore()
  }

  /**
   * 导出所有配置 (JSON 格式)
   */
  const exportConfigs = (): string => {
    const data: StoreData = {
      configs: configs.value,
      autoApply: autoApply.value
    }
    return JSON.stringify(data, null, 2)
  }

  /**
   * 导入配置
   */
  const importConfigs = async (jsonData: string) => {
    try {
      const data: StoreData = JSON.parse(jsonData)

      if (data.configs && Array.isArray(data.configs)) {
        // 合并导入的配置（避免 ID 冲突）
        const newConfigs = data.configs.map((c) => ({
          ...c,
          id: crypto.randomUUID(), // 重新生成 ID
          createdAt: Date.now(),
          updatedAt: Date.now(),
          source: 'import' as const
        }))

        configs.value.push(...newConfigs)
      }

      if (data.autoApply) {
        autoApply.value = {
          ...createDefaultAutoApply(),
          ...data.autoApply,
          opggTier: isOpggTier(data.autoApply.opggTier) ? data.autoApply.opggTier : createDefaultAutoApply().opggTier
        }
      }

      await saveToStore()
    } catch (error) {
      console.error('导入符文配置失败:', error)
      throw new Error('导入失败：数据格式不正确')
    }
  }

  /**
   * 清空所有配置 (危险操作)
   */
  const clearAllConfigs = async () => {
    configs.value = []
    await saveToStore()
  }

  return {
    // 状态
    configs,
    autoApply,
    isLoaded,

    // 计算属性
    configCount,

    // 方法
    loadFromStore,
    saveToStore,
    addConfig,
    updateConfig,
    deleteConfig,
    setAsDefault,
    incrementUsageCount,
    findBestMatch,
    getConfigsByChampion,
    getConfigsByPosition,
    getConfigById,
    updateAutoApply,
    exportConfigs,
    importConfigs,
    clearAllConfigs
  }
})
