export const DEFAULT_OVERLAY_SHORTCUT = 'Insert'

const MODIFIER_CODES = new Set([
  'ControlLeft',
  'ControlRight',
  'ShiftLeft',
  'ShiftRight',
  'AltLeft',
  'AltRight',
  'MetaLeft',
  'MetaRight',
  'OSLeft',
  'OSRight'
])

type AcceleratorEvent = Pick<KeyboardEvent, 'code' | 'key' | 'ctrlKey' | 'altKey' | 'shiftKey' | 'metaKey'>

export function eventToAccelerator(event: AcceleratorEvent): string | null {
  if (MODIFIER_CODES.has(event.code)) return null
  const key = codeToToken(event.code, event.key)
  if (!key) return null

  const parts: string[] = []
  if (event.ctrlKey) parts.push('Ctrl')
  if (event.altKey) parts.push('Alt')
  if (event.shiftKey) parts.push('Shift')
  if (event.metaKey) parts.push('Super')
  parts.push(key)
  return parts.join('+')
}

export function formatAccelerator(raw: string): string {
  return raw
    .split(/[+\s]+/)
    .map((part) => part.trim())
    .filter(Boolean)
    .map(formatPart)
    .join('+')
}

function formatPart(part: string): string {
  const lower = part.toLowerCase()
  if (lower === 'ctrl' || lower === 'control') return 'Ctrl'
  if (lower === 'alt' || lower === 'option') return 'Alt'
  if (lower === 'shift') return 'Shift'
  if (lower === 'super' || lower === 'meta' || lower === 'cmd' || lower === 'command' || lower === 'win') {
    return 'Super'
  }
  if (lower === 'space') return 'Space'
  if (lower === 'home') return 'Home'
  if (lower === 'insert' || lower === 'ins') return 'Insert'
  if (/^f\d{1,2}$/.test(lower)) return lower.toUpperCase()
  if (part.length === 1) return part.toUpperCase()
  return part[0].toUpperCase() + part.slice(1)
}

function codeToToken(code: string, key: string): string | null {
  if (/^F\d{1,2}$/.test(code)) return code
  if (code.startsWith('Key') && code.length === 4) return code.slice(3)
  if (code.startsWith('Digit') && code.length === 6) return code.slice(5)
  if (code === 'Space' || key === ' ') return 'Space'
  if (code === 'Home' || key === 'Home') return 'Home'
  if (code === 'Insert' || key === 'Insert') return 'Insert'
  if (key.length === 1 && /[a-zA-Z0-9]/.test(key)) return key.toUpperCase()
  return null
}
