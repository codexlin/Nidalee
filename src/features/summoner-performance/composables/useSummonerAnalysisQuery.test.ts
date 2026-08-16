import { invoke } from '@tauri-apps/api/core'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import {
  buildSummonerAnalysisRequest,
  fetchSummonerAnalysisForQuery,
  summonerAnalysisQueryKey
} from './useSummonerAnalysisQuery'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

describe('summoner analysis request', () => {
  const invokeMock = vi.mocked(invoke)

  beforeEach(() => invokeMock.mockReset())

  it('排位使用深度分析但关闭昂贵时间线能力', () => {
    const request = buildSummonerAnalysisRequest({ category: 'ranked', rankedScope: 'solo' })

    expect(request).toMatchObject({ count: 20, mode: 'soloRanked', queueId: 420, depth: 'deep' })
    expect(request.features.timeline).toBe(false)
    expect(request.maxAnalysisGames).toBe(20)
  })

  it('其他模式只请求简单分析', () => {
    const request = buildSummonerAnalysisRequest({ category: 'other', rankedScope: 'mixed' })

    expect(request).toMatchObject({ count: 20, mode: 'normals', depth: 'simple' })
    expect(request.maxAnalysisGames).toBeUndefined()
  })

  it('缓存按玩家和有效范围隔离', () => {
    expect(summonerAnalysisQueryKey('p1', { category: 'ranked', rankedScope: 'flex' })).toEqual([
      'summoner-analysis',
      'p1',
      'ranked:flex',
      { category: 'ranked', rankedScope: 'flex' }
    ])
    expect(summonerAnalysisQueryKey('p1', { category: 'other', rankedScope: 'solo' })).toEqual([
      'summoner-analysis',
      'p1',
      'other',
      { category: 'other', rankedScope: 'mixed' }
    ])
  })

  it('旧查询重试时使用 query key 中的玩家与范围快照', async () => {
    invokeMock.mockResolvedValue({} as MatchAnalysisResult)
    const oldQueryKey = summonerAnalysisQueryKey('player-a', {
      category: 'ranked',
      rankedScope: 'solo'
    })

    // 模拟界面已切到另一玩家和范围后，旧查询才进入 retry。
    summonerAnalysisQueryKey('player-b', { category: 'other', rankedScope: 'mixed' })
    await fetchSummonerAnalysisForQuery({ queryKey: oldQueryKey })

    expect(invokeMock).toHaveBeenCalledWith('analyze_matches', {
      puuid: 'player-a',
      request: expect.objectContaining({ mode: 'soloRanked', queueId: 420 })
    })
  })
})
