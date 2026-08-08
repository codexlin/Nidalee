// --- 手动添加的前端专用类型 ---

/**
 * 英雄信息接口，用于自动选/禁等功能
 * 这是一个纯前端的类型，不与后端直接对应
 */
interface ChampionInfo {
  id: number
  name: string
  alias: string
  squarePortraitPath: string
  roles: string[]
}
// 为游戏中的玩家定义一个新的、独立的“增强”类型
interface EnrichedLivePlayer {
  displayName: string
  championName: string
  championId?: number | null
  assignedPosition?: string | null
  team: string
  isBot: boolean
  isLocal: boolean // 通常对于敌方队伍总是 false
  championIcon?: string
  // 添加 TeamCard 需要的其他字段
  cellId?: number
  puuid?: string | null
  summonerId?: string | null
  championPickIntent?: number | null
  selectedSkinId?: number | null
  spell1Id?: number | null
  spell2Id?: number | null
  tagLine?: string | null
  profileIconId?: bigint | null
  tier?: string | null
  recentMatches?: Array<MatchPerformance> | null
}
// 和 TeamCard.vue 中使用相同的通用接口
interface PlayerDisplayInfo {
  championId?: number | null // championId 现在是可选的
  displayName: string
  championName?: string
  tier?: string | null
  assignedPosition?: string
  spell1Id?: number | null
  spell2Id?: number | null
  cellId?: number
  puuid?: string | null
}
// 定义一个“富集”后的玩家类型，继承自原始类型并添加前端需要的字段
interface EnrichedChampSelectPlayer extends ChampSelectPlayer {
  displayName: string // 覆盖可选类型，确保始终有值
  avatar: string
  isBot: boolean
  isLocal: boolean
  spell1Icon?: string
  spell2Icon?: string
  championIcon?: string
  rankIcon?: string
}
interface ConnectedState {
  state: ConnectionState
  auth_info: LcuAuthInfo | null
  consecutive_failures: number
  error_message: string | null
}
