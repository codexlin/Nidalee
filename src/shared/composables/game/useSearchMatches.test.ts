import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useSearchMatches } from './useSearchMatches'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

vi.mock('@/shared/stores/features/searchHistoryStore', () => ({
  useSearchHistoryStore: () => ({
    items: [],
    add: vi.fn(),
    remove: vi.fn(),
    clear: vi.fn()
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

function summoner(displayName: string, puuid = displayName): SummonerInfo {
  return {
    displayName,
    gameName: displayName,
    tagLine: 'TEST',
    summonerLevel: 1,
    profileIconId: 0,
    puuid,
    accountId: '',
    summonerId: '',
    xpSinceLastLevel: 0,
    xpUntilNextLevel: 0,
    percentCompleteForNextLevel: null,
    gameStatus: null,
    availability: null,
    challengePoints: null,
    challengeCrystalLevel: null,
    soloRankTier: null,
    soloRankDivision: null,
    soloRankWins: null,
    soloRankLosses: null,
    soloRankLp: null,
    flexRankTier: null,
    flexRankDivision: null,
    flexRankWins: null,
    flexRankLosses: null,
    flexRankLp: null,
    highestRankThisSeason: null,
    currentPerkPage: null,
    primaryStyleId: null,
    subStyleId: null,
    selectedPerkIds: null
  }
}

describe('useSearchMatches identity lifecycle', () => {
  const invokeMock = vi.mocked(invoke)

  beforeEach(() => invokeMock.mockReset())

  it('只请求召唤师身份，不再请求或过滤战绩', async () => {
    invokeMock.mockResolvedValue([summoner('A')])
    const search = useSearchMatches()

    await search.fetchSummonerInfo(['A#TEST'])

    expect(invokeMock).toHaveBeenCalledOnce()
    expect(invokeMock).toHaveBeenCalledWith('get_summoners_by_names', { names: ['A#TEST'] })
    expect(search.currentResult.value?.displayName).toBe('A')
  })

  it('身份接口没有结果时给出稳定空态说明', async () => {
    invokeMock.mockResolvedValue([])
    const search = useSearchMatches()

    await search.fetchSummonerInfo(['missing#TEST'])

    expect(search.currentResult.value).toBeNull()
    expect(search.result.value).toEqual([])
    expect(search.error.value).toContain('未找到匹配的召唤师')
  })

  it('慢返回的旧搜索不能覆盖最新搜索', async () => {
    const first = deferred<SummonerInfo[]>()
    const second = deferred<SummonerInfo[]>()
    invokeMock.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)
    const search = useSearchMatches()

    const firstRequest = search.fetchSummonerInfo(['A'])
    const secondRequest = search.fetchSummonerInfo(['B'])

    second.resolve([summoner('B')])
    await secondRequest
    first.resolve([summoner('A')])
    await firstRequest

    expect(search.currentResult.value?.displayName).toBe('B')
    expect(search.loading.value).toBe(false)
  })

  it('旧搜索失败不能清空最新搜索的成功结果', async () => {
    const first = deferred<SummonerInfo[]>()
    const second = deferred<SummonerInfo[]>()
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
  })

  it('清理后忽略仍在途中的响应', async () => {
    const pending = deferred<SummonerInfo[]>()
    invokeMock.mockReturnValueOnce(pending.promise)
    const search = useSearchMatches()

    const request = search.fetchSummonerInfo(['A'])
    search.clearSummonerInfo()
    pending.resolve([summoner('A')])
    await request

    expect(search.currentResult.value).toBeNull()
    expect(search.result.value).toEqual([])
    expect(search.loading.value).toBe(false)
  })
})
