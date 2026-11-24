import { beforeEach, describe, expect, it } from 'vitest'

import {
  getChampionCatalogSize,
  installChampionCatalog,
  resolveChampionName,
  setChampionCatalog
} from './staticCatalog'

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

it('offers only the regular Lee Sin and retains both records for match lookups', () => {
  const base = {
    id: 64,
    alias: 'LeeSin',
    name: '盲僧',
    description: '李青',
    roles: ['fighter', 'assassin'],
    squarePortraitPath: '',
    selectable: true
  }
  const variant = { ...base, id: 60064, alias: 'Jade_LeeSin', selectable: false }
  expect(installChampionCatalog([variant, base])).toEqual([base])
  expect(getChampionCatalogSize()).toBe(2)
  expect(resolveChampionName(60064)).toBe('盲僧')
})
