import { invoke } from '@tauri-apps/api/core'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useSummonerAndMatchUpdater } from './useSummonerAndMatchUpdater'

const mocks = vi.hoisted(() => ({
  analyzeMatches: vi.fn(),
  cancelPendingMatchAnalysis: vi.fn()
}))

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('./useMatchAnalysis', () => ({
  useMatchAnalysis: () => ({ analyzeMatches: mocks.analyzeMatches }),
  cancelPendingMatchAnalysis: mocks.cancelPendingMatchAnalysis
}))

interface Deferred<T> {
  promise: Promise<T>
  resolve: (value: T) => void
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise
  })
  return { promise, resolve }
}

function summoner(displayName: string, puuid: string): SummonerInfo {
  return {
    displayName,
    gameName: displayName,
    tagLine: 'TEST',
    summonerLevel: 1,
    profileIconId: 0,
    puuid
  } as SummonerInfo
}

describe('account initialization lifecycle', () => {
  const invokeMock = vi.mocked(invoke)
  const dataStore = {
    summonerInfo: null as SummonerInfo | null,
    startLoadingSummoner: vi.fn(),
    setSummonerInfo: vi.fn((info: SummonerInfo) => {
      dataStore.summonerInfo = info
    }),
    clearSummonerInfo: vi.fn(() => {
      dataStore.summonerInfo = null
    })
  }
  const personalAnalysis = {
    overallStats: null,
    setLoading: vi.fn(),
    clear: vi.fn()
  }
  const activityStore = { addActivity: vi.fn() }
  let updater: ReturnType<typeof useSummonerAndMatchUpdater>

  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
    dataStore.summonerInfo = null
    dataStore.setSummonerInfo.mockImplementation((info: SummonerInfo) => {
      dataStore.summonerInfo = info
    })
    dataStore.clearSummonerInfo.mockImplementation(() => {
      dataStore.summonerInfo = null
    })
    vi.stubGlobal('window', { setTimeout })
    vi.stubGlobal('useDataStore', () => dataStore)
    vi.stubGlobal('usePersonalMatchAnalysisStore', () => personalAnalysis)
    vi.stubGlobal('useActivityStore', () => activityStore)
    updater = useSummonerAndMatchUpdater()
  })

  afterEach(() => {
    updater.cancelPendingUpdates()
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('没有显式账号身份的重复 Connected 调用复用当前初始化', async () => {
    const accountA = summoner('A', 'puuid-a')
    const identity = deferred<SummonerInfo>()
    invokeMock.mockReturnValue(identity.promise)
    mocks.analyzeMatches.mockResolvedValue({} as MatchAnalysisResult)

    const first = updater.updateSummonerAndMatches()
    const second = updater.updateSummonerAndMatches()

    expect(second).toBe(first)
    expect(invokeMock).toHaveBeenCalledTimes(1)

    identity.resolve(accountA)
    await vi.advanceTimersByTimeAsync(350)
    await first
    expect(mocks.analyzeMatches).toHaveBeenCalledTimes(1)
  })

  it('未知目标切到 B 后拒绝旧 HTTP 身份，直到 endpoint 返回 B', async () => {
    const accountA = summoner('A', 'puuid-a')
    const accountB = summoner('B', 'puuid-b')
    const identityA = deferred<SummonerInfo>()
    invokeMock.mockReturnValueOnce(identityA.promise).mockResolvedValueOnce(accountA).mockResolvedValueOnce(accountB)
    mocks.analyzeMatches.mockResolvedValue({} as MatchAnalysisResult)

    const runUnknown = updater.updateSummonerAndMatches()
    const runB = updater.updateSummonerAndMatches(accountB)

    expect(runB).not.toBe(runUnknown)
    expect(mocks.cancelPendingMatchAnalysis).toHaveBeenCalledTimes(1)
    expect(dataStore.summonerInfo?.puuid).toBe('puuid-b')

    identityA.resolve(accountA)
    await vi.advanceTimersByTimeAsync(500)
    await Promise.all([runUnknown, runB])
    expect(dataStore.summonerInfo?.puuid).toBe('puuid-b')
    expect(dataStore.setSummonerInfo.mock.calls.every(([info]) => info.puuid === 'puuid-b')).toBe(true)
    expect(mocks.analyzeMatches).toHaveBeenCalledTimes(1)
  })

  it('A 分析未完成时切到 B，不合流且旧任务不能清掉 B flight', async () => {
    const accountA = summoner('A', 'puuid-a')
    const accountB = summoner('B', 'puuid-b')
    const analysisA = deferred<MatchAnalysisResult | null>()
    const analysisB = deferred<MatchAnalysisResult | null>()
    invokeMock.mockResolvedValueOnce(accountA).mockResolvedValueOnce(accountB)
    mocks.analyzeMatches.mockReturnValueOnce(analysisA.promise).mockReturnValueOnce(analysisB.promise)

    const runA = updater.updateSummonerAndMatches(accountA)
    await vi.advanceTimersByTimeAsync(350)
    expect(mocks.analyzeMatches).toHaveBeenCalledTimes(1)

    const runB = updater.updateSummonerAndMatches(accountB)
    expect(mocks.cancelPendingMatchAnalysis).toHaveBeenCalledTimes(1)
    expect(personalAnalysis.clear).toHaveBeenCalledTimes(1)
    expect(dataStore.summonerInfo?.puuid).toBe('puuid-b')
    await vi.advanceTimersByTimeAsync(350)
    expect(mocks.analyzeMatches).toHaveBeenCalledTimes(2)

    analysisA.resolve(null)
    await runA
    const joinedB = updater.updateSummonerAndMatches(accountB)
    expect(joinedB).toBe(runB)
    expect(mocks.analyzeMatches).toHaveBeenCalledTimes(2)

    analysisB.resolve({} as MatchAnalysisResult)
    await runB
    expect(dataStore.summonerInfo?.puuid).toBe('puuid-b')
  })
})
