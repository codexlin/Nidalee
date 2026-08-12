import { describe, expect, it } from 'vitest'
import { resolveBuildContext } from './buildContext'

describe('resolveBuildContext', () => {
  it.each([420, 440])('resolves ranked queue %s with a canonical position', (queueId) => {
    expect(resolveBuildContext({ queueId, isCustomGame: false, position: 'middle' })).toEqual({
      status: 'ready',
      scenario: 'ranked-mid',
      providerMode: 'ranked',
      providerPosition: 'MID'
    })
  })

  it('waits instead of guessing when a ranked position is unavailable', () => {
    expect(resolveBuildContext({ queueId: 420, isCustomGame: false, position: '' })).toEqual({
      status: 'waiting',
      reason: 'missing-ranked-position'
    })
  })

  it.each([400, 430, 490])('uses a champion-level normal context for queue %s', (queueId) => {
    expect(resolveBuildContext({ queueId, isCustomGame: false, position: 'TOP' })).toEqual({
      status: 'ready',
      scenario: 'normal-sr',
      providerMode: 'ranked',
      providerPosition: 'main-position'
    })
  })

  it('uses the positionless ARAM provider contract for queue 450', () => {
    expect(resolveBuildContext({ queueId: 450, isCustomGame: false })).toEqual({
      status: 'ready',
      scenario: 'aram',
      providerMode: 'aram',
      providerPosition: 'none'
    })
  })

  it('waits for queue metadata and skips custom or unsupported games', () => {
    expect(resolveBuildContext({ queueId: 0, isCustomGame: false }).status).toBe('waiting')
    expect(resolveBuildContext({ queueId: 420, isCustomGame: true, position: 'TOP' })).toEqual({
      status: 'unsupported',
      reason: 'custom-game'
    })
    expect(resolveBuildContext({ queueId: 900, isCustomGame: false })).toEqual({
      status: 'unsupported',
      reason: 'unsupported-queue'
    })
  })
})
