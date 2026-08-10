import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useSearchMatches } from './useSearchMatches'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}))

vi.mock('@/shared/stores/ui/settingsStore', () => ({
  useSettingsStore: () => ({
    applyDefaultFilterOnSearch: false,
    lastMatchMode: 'all',
    defaultQueueTypes: []
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

function summoner(displayName: string): SummonerWithMatches {
  return { displayName } as SummonerWithMatches
}

function matchStats(totalGames: number): PlayerMatchStats {
  return {
    totalGames,
    wins: 0,
    losses: totalGames,
    winRate: 0,
    avgKills: 0,
    avgDeaths: 0,
    avgAssists: 0,
    avgKda: 0,
    todayGames: 0,
    todayWins: 0,
    dpm: 0,
    cspm: 0,
    vspm: 0,
    traits: [],
    favoriteChampions: [],
    recentPerformance: []
  }
}

describe('useSearchMatches request lifecycle', () => {
  const invokeMock = vi.mocked(invoke)

  beforeEach(() => {
    invokeMock.mockReset()
  })

  it('慢返回的旧搜索不能覆盖最新搜索', async () => {
    const first = deferred<SummonerWithMatches[]>()
    const second = deferred<SummonerWithMatches[]>()
    invokeMock.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)
    const search = useSearchMatches()

    const firstRequest = search.fetchSummonerInfo(['A'])
    const secondRequest = search.fetchSummonerInfo(['B'])

    second.resolve([summoner('B')])
    await secondRequest
    expect(search.currentResult.value?.displayName).toBe('B')
    expect(search.loading.value).toBe(false)

    first.resolve([summoner('A')])
    await firstRequest
    expect(search.currentResult.value?.displayName).toBe('B')
    expect(search.error.value).toBe('')
  })

  it('旧搜索失败不能清空最新搜索的成功结果', async () => {
    const first = deferred<SummonerWithMatches[]>()
    const second = deferred<SummonerWithMatches[]>()
    invokeMock.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)
    const search = useSearchMatches()

    const firstRequest = search.fetchSummonerInfo(['A'])
    const secondRequest = search.fetchSummonerInfo(['B'])
    second.resolve([summoner('B')])
    await secondRequest

    first.reject(new Error('A 查询失败'))
    await firstRequest

    expect(search.currentResult.value?.displayName).toBe('B')
    expect(search.error.value).toBe('')
    expect(search.loading.value).toBe(false)
  })

  it('旧搜索先结束时仍保持最新搜索的加载状态', async () => {
    const first = deferred<SummonerWithMatches[]>()
    const second = deferred<SummonerWithMatches[]>()
    invokeMock.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)
    const search = useSearchMatches()

    const firstRequest = search.fetchSummonerInfo(['A'])
    const secondRequest = search.fetchSummonerInfo(['B'])

    first.resolve([summoner('A')])
    await firstRequest
    expect(search.currentResult.value).toBeNull()
    expect(search.loading.value).toBe(true)

    second.resolve([summoner('B')])
    await secondRequest
    expect(search.currentResult.value?.displayName).toBe('B')
    expect(search.loading.value).toBe(false)
  })

  it('清理搜索状态后忽略仍在途中的响应', async () => {
    const pending = deferred<SummonerWithMatches[]>()
    invokeMock.mockReturnValueOnce(pending.promise)
    const search = useSearchMatches()

    const request = search.fetchSummonerInfo(['A'])
    expect(search.loading.value).toBe(true)
    search.clearSummonerInfo()

    pending.resolve([summoner('A')])
    await request

    expect(search.currentResult.value).toBeNull()
    expect(search.result.value).toBeNull()
    expect(search.loading.value).toBe(false)
  })

  it('慢返回的旧战绩不能覆盖最新战绩', async () => {
    const first = deferred<PlayerMatchStats>()
    const second = deferred<PlayerMatchStats>()
    invokeMock.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)
    const search = useSearchMatches()

    const firstRequest = search.getRecentMatchesByPuuid(['A'])
    const secondRequest = search.getRecentMatchesByPuuid(['B'])

    second.resolve(matchStats(2))
    await secondRequest
    expect(search.summonerStats.value?.[0]?.totalGames).toBe(2)

    first.resolve(matchStats(1))
    await firstRequest
    expect(search.summonerStats.value?.[0]?.totalGames).toBe(2)
  })
})
