import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick, ref } from 'vue'
import { useGameDetail } from './useGameDetail'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@/shared/composables/utils/useActivityLogger', () => ({
  useActivityLogger: () => ({
    logError: { apiError: vi.fn() }
  })
}))

vi.mock('@/shared/stores/core/dataStore', () => ({
  useDataStore: () => ({
    gameVersion: '16.1.1',
    summonerInfo: { puuid: 'local-puuid' }
  })
}))

vi.mock('@/shared/stores/features/personalMatchAnalysisStore', () => ({
  usePersonalMatchAnalysisStore: () => ({
    lastPuuid: null,
    getMatchEvidence: () => null
  })
}))

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
  reject: (reason: unknown) => void
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  let reject!: (reason: unknown) => void
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })
  return { promise, resolve, reject }
}

function game(gameId: number, championId = 1): MatchPerformance {
  return {
    gameId,
    championId,
    championName: 'Annie',
    kills: 1,
    deaths: 2,
    assists: 3,
    kda: 2,
    win: true,
    queueId: 420,
    gameCreation: 0,
    gameDuration: 1800
  } as MatchPerformance
}

function detail(gameId: number): GameDetail {
  return {
    gameId,
    gameDuration: 1800,
    gameVersion: '16.1.1',
    participants: [],
    teams: [],
    maxDamage: 0,
    maxTank: 0,
    maxStreak: 0,
    bestPlayerChampionId: 0,
    maxTankChampionId: 0,
    maxStreakChampionId: 0
  } as GameDetail
}

describe('useGameDetail request lifecycle', () => {
  const invokeMock = vi.mocked(invoke)

  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('慢返回的旧对局详情不能覆盖最新选择', async () => {
    const first = deferred<GameDetail>()
    const second = deferred<GameDetail>()
    invokeMock.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)

    const selectedGame = ref<MatchPerformance | null>(game(101))
    const { loading, gameDetailData } = useGameDetail(selectedGame)
    await nextTick()

    selectedGame.value = game(202)
    await nextTick()

    first.resolve(detail(101))
    await first.promise
    await nextTick()
    expect(gameDetailData.value).toBeNull()

    second.resolve(detail(202))
    await second.promise
    await nextTick()
    expect(gameDetailData.value?.gameId).toBe(202)
    expect(loading.value).toBe(false)
  })

  it('清空选中对局时重置详情状态', async () => {
    invokeMock.mockResolvedValueOnce(detail(303))
    const selectedGame = ref<MatchPerformance | null>(game(303))
    const { loading, gameDetailData } = useGameDetail(selectedGame)
    await nextTick()
    await Promise.resolve()
    await nextTick()
    expect(gameDetailData.value?.gameId).toBe(303)

    selectedGame.value = null
    await nextTick()
    expect(gameDetailData.value).toBeNull()
    expect(loading.value).toBe(false)
  })
})
