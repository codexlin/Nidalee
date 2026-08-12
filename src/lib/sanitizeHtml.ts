const ALLOWED_TAGS = new Set([
  'ATTENTION',
  'B',
  'BR',
  'DIV',
  'EM',
  'I',
  'MAINTEXT',
  'P',
  'PASSIVE',
  'SMALL',
  'SPAN',
  'STATS',
  'STRONG'
])

const BLOCKED_TAGS = new Set([
  'BASE',
  'BUTTON',
  'EMBED',
  'FORM',
  'IFRAME',
  'IMG',
  'INPUT',
  'LINK',
  'MATH',
  'META',
  'OBJECT',
  'SCRIPT',
  'SELECT',
  'STYLE',
  'SVG',
  'TEMPLATE',
  'TEXTAREA',
  'VIDEO'
])

/**
 * Preserve Riot's lightweight item-description markup while dropping every
 * attribute and all active/embedded content from the remote payload.
 */
export function sanitizeItemDescription(value: string): string {
  if (!value) return ''

  const parsed = new DOMParser().parseFromString(value, 'text/html')
  const output = document.createElement('div')

  const appendSafeNode = (source: Node, target: Node): void => {
    if (source.nodeType === Node.TEXT_NODE) {
      target.appendChild(document.createTextNode(source.textContent ?? ''))
      return
    }

    if (source.nodeType !== Node.ELEMENT_NODE) return

    const element = source as Element
    if (BLOCKED_TAGS.has(element.tagName)) return

    if (!ALLOWED_TAGS.has(element.tagName)) {
      element.childNodes.forEach((child) => appendSafeNode(child, target))
      return
    }

    const safeElement = document.createElement(element.tagName.toLowerCase())
    element.childNodes.forEach((child) => appendSafeNode(child, safeElement))
    target.appendChild(safeElement)
  }

  parsed.body.childNodes.forEach((child) => appendSafeNode(child, output))
  return output.innerHTML
}
