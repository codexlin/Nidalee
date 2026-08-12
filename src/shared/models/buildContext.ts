import {
  normalizeBuildPosition,
  rankedScenarioFromPosition,
  type BuildPosition,
  type BuildScenario
} from './buildPreset'

export type SupportedOpggMode = 'ranked' | 'aram'

export interface ReadyBuildContext {
  status: 'ready'
  scenario: BuildScenario
  providerMode: SupportedOpggMode
  /** 普通峡谷没有可靠分路，需要从推荐数据中解析英雄主流位置。 */
  providerPosition: BuildPosition | 'none' | 'main-position'
}

export interface WaitingBuildContext {
  status: 'waiting'
  reason: 'missing-queue' | 'missing-ranked-position'
}

export interface UnsupportedBuildContext {
  status: 'unsupported'
  reason: 'custom-game' | 'unsupported-queue'
}

export type BuildContextResolution = ReadyBuildContext | WaitingBuildContext | UnsupportedBuildContext

export interface BuildContextInput {
  queueId: number
  isCustomGame: boolean
  position?: string | null
}

const RANKED_QUEUE_IDS = new Set([420, 440])
const NORMAL_SR_QUEUE_IDS = new Set([400, 430, 490])

/** Resolves LCU queue state into the only scenarios allowed to modify a rune page automatically. */
export function resolveBuildContext(input: BuildContextInput): BuildContextResolution {
  if (input.isCustomGame) return { status: 'unsupported', reason: 'custom-game' }
  if (!Number.isInteger(input.queueId) || input.queueId <= 0) {
    return { status: 'waiting', reason: 'missing-queue' }
  }

  if (RANKED_QUEUE_IDS.has(input.queueId)) {
    const position = normalizeBuildPosition(input.position)
    const scenario = rankedScenarioFromPosition(position)
    return position && scenario
      ? { status: 'ready', scenario, providerMode: 'ranked', providerPosition: position }
      : { status: 'waiting', reason: 'missing-ranked-position' }
  }

  if (NORMAL_SR_QUEUE_IDS.has(input.queueId)) {
    return {
      status: 'ready',
      scenario: 'normal-sr',
      providerMode: 'ranked',
      providerPosition: 'main-position'
    }
  }

  if (input.queueId === 450) {
    return { status: 'ready', scenario: 'aram', providerMode: 'aram', providerPosition: 'none' }
  }

  return { status: 'unsupported', reason: 'unsupported-queue' }
}
