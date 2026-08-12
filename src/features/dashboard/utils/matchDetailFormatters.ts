/** 对局详情展示用纯格式化（不依赖请求 composable） */

export function multiKillLabel(n: number) {
  if (!n || n <= 1) return '无'
  if (n === 2) return '双杀'
  if (n === 3) return '三杀'
  if (n === 4) return '四杀'
  if (n >= 5) return '五杀'
  return String(n)
}

export function formatMatchDuration(seconds: number) {
  const m = Math.floor(seconds / 60)
  const s = seconds % 60
  return `${m}分 ${s}秒`
}

export function formatMatchNumber(num: number) {
  return num.toLocaleString()
}
