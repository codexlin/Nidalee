/**
 * 对局分析模块专用类型定义
 * 🎉 简化版：大部分类型已通过 ts-rs 自动生成，这里只保留 UI 特定的类型
 */

export type GamePhase =
  | 'None'
  | 'Lobby'
  | 'Matchmaking'
  | 'ChampSelect'
  | 'InProgress'
  | 'EndOfGame'
  | 'ReadyCheck'
  | 'Reconnect'

/** 对局分析组件直接消费后端玩家契约，不再维护 UI 平行副本。 */
export type UIPlayerData = PlayerAnalysisData
