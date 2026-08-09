import type { Component } from 'vue'
import { Gamepad2, Swords, Users, Zap, Flame, Target, Crown, Trophy, Shield, Gamepad } from 'lucide-vue-next'

// 游戏类型定义接口
export interface GameType {
  id: number
  name: string
  description: string
  icon: Component
  category: 'ranked' | 'casual' | 'special' | 'ai' | 'custom'
}

// 游戏类型定义
export const gameTypes: GameType[] = [
  // 排位赛
  {
    id: 420,
    name: '单双排',
    description: 'Solo/Duo 排位赛',
    icon: Crown,
    category: 'ranked'
  },
  {
    id: 440,
    name: '灵活组排',
    description: 'Flex 组队排位',
    icon: Users,
    category: 'ranked'
  },

  // 匹配模式
  {
    id: 430,
    name: '匹配模式',
    description: 'Normal 普通匹配',
    icon: Gamepad,
    category: 'casual'
  },
  {
    id: 400,
    name: '灵活匹配',
    description: '匹配模式(征召)',
    icon: Shield,
    category: 'casual'
  },

  // 自定义
  {
    id: 0,
    name: '自定义',
    description: '自定义游戏',
    icon: Gamepad2,
    category: 'custom'
  },

  // 大乱斗
  {
    id: 450,
    name: '极地大乱斗',
    description: 'ARAM 大乱斗',
    icon: Target,
    category: 'casual'
  },

  // 特殊模式
  {
    id: 900,
    name: '无限乱斗',
    description: 'URF 无限火力',
    icon: Zap,
    category: 'special'
  },
  {
    id: 1700,
    name: '斗魂竞技场',
    description: 'Arena 2v2v2v2',
    icon: Flame,
    category: 'special'
  },
  {
    id: 700,
    name: '冠军杯赛',
    description: 'Clash 冠军杯',
    icon: Trophy,
    category: 'special'
  },

  // 人机对战
  {
    id: 800,
    name: '人机对战',
    description: 'AI 人机对战',
    icon: Shield,
    category: 'ai'
  },

  // 克隆模式
  {
    id: 1020,
    name: '克隆大作战',
    description: 'Clash 克隆',
    icon: Swords,
    category: 'special'
  },

  // 其他热门模式
  {
    id: 1200,
    name: '极限闪击',
    description: 'Nexus Blitz 极限闪击',
    icon: Target,
    category: 'special'
  },
  {
    id: 1400,
    name: '终极魔典',
    description: 'Ultimate Spellbook',
    icon: Zap,
    category: 'special'
  },
  {
    id: 1900,
    name: '无限火力',
    description: 'URF 无限火力',
    icon: Zap,
    category: 'special'
  },
  {
    id: 2300,
    name: '神木之门',
    description: 'Arena 神木之门',
    icon: Shield,
    category: 'special'
  },
  {
    id: 2400,
    name: '海克斯大乱斗',
    description: 'ARAM: Mayhem / KIWI',
    icon: Target,
    category: 'special'
  }
]

// 预设选择定义
export interface GameTypePreset {
  key: string
  label: string
  types: number[]
  selected: boolean
  partial: boolean
}

export const gameTypePresets: Omit<GameTypePreset, 'selected' | 'partial'>[] = [
  {
    key: 'ranked',
    label: '排位赛',
    types: [420, 440] // 单双排 + 灵活组排
  },
  {
    key: 'casual',
    label: '休闲模式',
    types: [0, 430, 400, 450] // 自定义 + 匹配 + 灵活匹配 + 大乱斗
  },
  {
    key: 'special',
    label: '特殊模式',
    types: [900, 1700, 700, 1200, 1400, 1900, 2400] // 含海克斯大乱斗
  },
  {
    key: 'clone',
    label: '克隆模式',
    types: [1020, 2300] // 克隆大作战 + 神木之门
  },
  {
    key: 'ai',
    label: '人机对战',
    types: [800] // 人机对战
  }
]

// 根据队列ID获取游戏类型信息
export const getGameTypeById = (queueId: number): GameType | undefined => {
  return gameTypes.find((type) => type.id === queueId)
}

// 根据分类获取游戏类型
export const getGameTypesByCategory = (category: GameType['category']): GameType[] => {
  return gameTypes.filter((type) => type.category === category)
}

// 获取所有队列ID
export const getAllQueueIds = (): number[] => {
  return gameTypes.map((type) => type.id)
}
