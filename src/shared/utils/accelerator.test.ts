import { describe, expect, it } from 'vitest'
import { eventToAccelerator, formatAccelerator } from './accelerator'

describe('eventToAccelerator', () => {
  it('maps Insert without modifiers', () => {
    expect(
      eventToAccelerator({
        code: 'Insert',
        key: 'Insert',
        ctrlKey: false,
        altKey: false,
        shiftKey: false,
        metaKey: false
      })
    ).toBe('Insert')
  })

  it('maps Home without modifiers', () => {
    expect(
      eventToAccelerator({ code: 'Home', key: 'Home', ctrlKey: false, altKey: false, shiftKey: false, metaKey: false })
    ).toBe('Home')
  })

  it('maps F8 without modifiers', () => {
    expect(eventToAccelerator({ code: 'F8', key: 'F8', ctrlKey: false, altKey: false, shiftKey: false, metaKey: false })).toBe(
      'F8'
    )
  })

  it('maps Ctrl+Shift+H', () => {
    expect(
      eventToAccelerator({ code: 'KeyH', key: 'H', ctrlKey: true, altKey: false, shiftKey: true, metaKey: false })
    ).toBe('Ctrl+Shift+H')
  })

  it('ignores modifier-only presses', () => {
    expect(
      eventToAccelerator({
        code: 'ControlLeft',
        key: 'Control',
        ctrlKey: true,
        altKey: false,
        shiftKey: false,
        metaKey: false
      })
    ).toBeNull()
  })
})

describe('formatAccelerator', () => {
  it('normalizes plugin-style shortcuts', () => {
    expect(formatAccelerator('insert')).toBe('Insert')
    expect(formatAccelerator('home')).toBe('Home')
    expect(formatAccelerator('f8')).toBe('F8')
    expect(formatAccelerator('ctrl+shift+h')).toBe('Ctrl+Shift+H')
  })
})
