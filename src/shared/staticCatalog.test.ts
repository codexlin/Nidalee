import { beforeEach, describe, expect, it } from 'vitest'

import { resolveChampionName, setChampionCatalog } from './staticCatalog'

describe('resolveChampionName', () => {
  beforeEach(() => {
    setChampionCatalog([{ id: 67, name: '暗夜猎手' }])
  })

  it('resolves a missing response name from the champion id catalog', () => {
    expect(resolveChampionName(67)).toBe('暗夜猎手')
  })

  it('keeps an explicit response name when one is available', () => {
    expect(resolveChampionName(67, 'Vayne')).toBe('Vayne')
  })
})
