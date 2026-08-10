import { describe, expect, it } from 'vitest'
import { createLatestRequestGuard } from './latestRequest'

describe('createLatestRequestGuard', () => {
  it('只允许最后开始的请求提交结果', () => {
    const guard = createLatestRequestGuard()
    const first = guard.begin()
    const second = guard.begin()

    expect(first.isCurrent()).toBe(false)
    expect(second.isCurrent()).toBe(true)
  })

  it('主动清理后使当前请求失效', () => {
    const guard = createLatestRequestGuard()
    const request = guard.begin()

    guard.invalidate()

    expect(request.isCurrent()).toBe(false)
  })

  it('旧请求的延迟清理不会使新请求失效', () => {
    const guard = createLatestRequestGuard()
    const first = guard.begin()
    const second = guard.begin()

    first.invalidate()

    expect(second.isCurrent()).toBe(true)
  })
})
