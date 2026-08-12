import { describe, expect, it } from 'vitest'
import { isOpggTier } from './opggTier'

describe('isOpggTier', () => {
  it('accepts canonical API values', () => {
    expect(isOpggTier('diamond_plus')).toBe(true)
    expect(isOpggTier('master')).toBe(true)
  })

  it('rejects non-canonical and unsupported values', () => {
    expect(isOpggTier('DIAMOND+')).toBe(false)
    expect(isOpggTier('MASTER+')).toBe(false)
    expect(isOpggTier('unsupported')).toBe(false)
  })
})
