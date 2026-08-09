/**
 * 读取主题 CSS 变量（oklch 完整色值）。
 * Token 真源：src/styles/theme.css
 */

function readToken(token: string): string {
  if (typeof document === 'undefined') return ''
  const name = token.startsWith('--') ? token : `--${token}`
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim()
}

/**
 * @param token CSS 变量名，如 `--primary` 或 `primary`
 * @param alpha 可选透明度 0–1，写入 oklch(... / alpha)
 */
export function themeColor(token: string, alpha?: number): string {
  const value = readToken(token)
  if (!value) {
    return alpha === undefined ? 'oklch(0.5 0 0)' : `oklch(0.5 0 0 / ${alpha})`
  }
  if (alpha === undefined) return value

  if (value.startsWith('oklch(') && value.endsWith(')')) {
    const inner = value.slice(6, -1).trim()
    const base = inner.split('/')[0].trim()
    return `oklch(${base} / ${alpha})`
  }

  // 已是带 alpha 的其它格式时，尽量用 color-mix
  return `color-mix(in oklch, ${value} ${Math.round(alpha * 100)}%, transparent)`
}

export function themeColors() {
  return {
    primary: themeColor('--primary'),
    primaryForeground: themeColor('--primary-foreground'),
    secondary: themeColor('--secondary'),
    background: themeColor('--background'),
    foreground: themeColor('--foreground'),
    muted: themeColor('--muted'),
    mutedForeground: themeColor('--muted-foreground'),
    border: themeColor('--border'),
    card: themeColor('--card'),
    cardForeground: themeColor('--card-foreground'),
    destructive: themeColor('--destructive'),
    destructiveForeground: themeColor('--destructive-foreground')
  }
}
