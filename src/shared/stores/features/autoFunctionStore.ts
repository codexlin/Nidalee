// 自动功能配置接口
interface AutoFunctionConfig {
  enabled: boolean
  delay: number // 延迟时间(毫秒)
}

// 自动选择英雄配置（支持多选和排序）
interface AutoSelectConfig extends AutoFunctionConfig {
  championList: ChampionInfo[]
}

// 自动禁用英雄配置（支持多选和排序）
interface AutoBanConfig extends AutoFunctionConfig {
  championList: ChampionInfo[]
}

// 自动功能状态
interface AutoFunctions {
  acceptMatch: AutoFunctionConfig
  selectChampion: AutoSelectConfig
  banChampion: AutoBanConfig
}

export const useAutoFunctionStore = defineStore(
  'autoFunction',
  () => {
    // 自动功能状态
    const autoFunctions = ref<AutoFunctions>({
      acceptMatch: {
        enabled: false,
        delay: 1000 // 默认1秒延迟
      },
      selectChampion: {
        enabled: false,
        delay: 2000, // 默认2秒延迟
        championList: []
      },
      banChampion: {
        enabled: false,
        delay: 2000, // 默认2秒延迟
        championList: []
      }
    })

    // 功能名称映射
    const functionNames = {
      acceptMatch: '自动接受对局',
      selectChampion: '自动选择英雄',
      banChampion: '自动禁用英雄'
    } as const

    // 切换自动功能开关
    const toggleAutoFunction = (key: keyof AutoFunctions) => {
      autoFunctions.value[key].enabled = !autoFunctions.value[key].enabled
      return {
        key,
        enabled: autoFunctions.value[key].enabled,
        name: functionNames[key]
      }
    }

    // 启用指定功能
    const enableFunction = (key: keyof AutoFunctions) => {
      console.log(`[autoFunctionStore] Enabling function:`, key)
      autoFunctions.value[key].enabled = true
      console.log(`[autoFunctionStore] Function ${key} enabled. Current state:`, autoFunctions.value[key])
      return {
        key,
        enabled: true,
        name: functionNames[key]
      }
    }

    // 禁用指定功能
    const disableFunction = (key: keyof AutoFunctions) => {
      console.log(`[autoFunctionStore] Disabling function:`, key)
      autoFunctions.value[key].enabled = false
      console.log(`[autoFunctionStore] Function ${key} disabled. Current state:`, autoFunctions.value[key])
      return {
        key,
        enabled: false,
        name: functionNames[key]
      }
    }

    // 禁用所有功能
    const disableAllFunctions = () => {
      Object.keys(autoFunctions.value).forEach((key) => {
        autoFunctions.value[key as keyof AutoFunctions].enabled = false
      })
    }

    // 设置功能延迟
    const setFunctionDelay = (key: keyof AutoFunctions, delay: number) => {
      const newDelay = Math.max(0, delay)
      console.log(`[autoFunctionStore] Setting delay for ${key}:`, {
        old: autoFunctions.value[key].delay,
        new: newDelay
      })
      autoFunctions.value[key].delay = newDelay
      console.log(`[autoFunctionStore] Delay set for ${key}. Current state:`, autoFunctions.value[key])
    }

    // 设置选择英雄列表（覆盖）
    const setChampionSelectList = (list: ChampionInfo[]) => {
      autoFunctions.value.selectChampion.championList = [...list]
    }

    // 添加选择英雄
    const addChampionSelect = (championInfo: ChampionInfo) => {
      const list = autoFunctions.value.selectChampion.championList
      if (!list.find((c) => c.id === championInfo.id)) {
        list.push(championInfo)
      }
    }

    // 移除选择英雄
    const removeChampionSelect = (championId: number) => {
      autoFunctions.value.selectChampion.championList = autoFunctions.value.selectChampion.championList.filter(
        (c) => c.id !== championId
      )
    }

    // 重新排序选择英雄
    const reorderChampionSelect = (from: number, to: number) => {
      const list = autoFunctions.value.selectChampion.championList
      if (from < 0 || to < 0 || from >= list.length || to >= list.length) return
      const [item] = list.splice(from, 1)
      list.splice(to, 0, item)
    }

    // 清空选择英雄
    const clearChampionSelect = () => {
      autoFunctions.value.selectChampion.championList = []
    }

    // 设置禁用英雄列表（覆盖）
    const setChampionBanList = (list: ChampionInfo[]) => {
      autoFunctions.value.banChampion.championList = [...list]
    }

    // 添加禁用英雄
    const addChampionBan = (championInfo: ChampionInfo) => {
      const list = autoFunctions.value.banChampion.championList
      if (!list.find((c) => c.id === championInfo.id)) {
        list.push(championInfo)
      }
    }

    // 移除禁用英雄
    const removeChampionBan = (championId: number) => {
      autoFunctions.value.banChampion.championList = autoFunctions.value.banChampion.championList.filter(
        (c) => c.id !== championId
      )
    }

    // 重新排序禁用英雄
    const reorderChampionBan = (from: number, to: number) => {
      const list = autoFunctions.value.banChampion.championList
      if (from < 0 || to < 0 || from >= list.length || to >= list.length) return
      const [item] = list.splice(from, 1)
      list.splice(to, 0, item)
    }

    // 清空禁用英雄
    const clearChampionBan = () => {
      autoFunctions.value.banChampion.championList = []
    }

    // 获取功能名称
    const getFunctionName = (key: keyof AutoFunctions): string => {
      return functionNames[key]
    }

    // 计算属性
    const enabledFunctionsCount = computed(() => {
      return Object.values(autoFunctions.value).filter((config) => config.enabled).length
    })

    const enabledFunctions = computed(() => {
      return Object.entries(autoFunctions.value)
        .filter(([, config]) => config.enabled)
        .map(([key]) => ({
          key: key as keyof AutoFunctions,
          name: functionNames[key as keyof AutoFunctions]
        }))
    })

    const isAnyFunctionEnabled = computed(() => enabledFunctionsCount.value > 0)

    // 获取特定功能的配置
    const getFunctionConfig = (key: keyof AutoFunctions) => {
      return autoFunctions.value[key]
    }

    return {
      // 状态
      autoFunctions,

      // 计算属性
      enabledFunctionsCount,
      enabledFunctions,
      isAnyFunctionEnabled,

      // 方法
      toggleAutoFunction,
      enableFunction,
      disableFunction,
      disableAllFunctions,
      setFunctionDelay,
      setChampionSelectList,
      addChampionSelect,
      removeChampionSelect,
      reorderChampionSelect,
      clearChampionSelect,
      setChampionBanList,
      addChampionBan,
      removeChampionBan,
      reorderChampionBan,
      clearChampionBan,
      getFunctionName,
      getFunctionConfig
    }
  },
  {
    persist: true
  }
)

// 导出类型
export type { AutoFunctions, AutoFunctionConfig, AutoSelectConfig, AutoBanConfig }
