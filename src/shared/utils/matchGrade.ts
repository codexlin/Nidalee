/** 与后端 thresholds::kda::grade_from_kda 对齐（旧缓存无 grade 时回退） */
export const gradeFromKda = (kda: number) => {
  if (kda >= 8) return 'S+'
  if (kda >= 6) return 'S'
  if (kda >= 4) return 'A'
  if (kda >= 2.5) return 'B'
  if (kda >= 1.5) return 'C'
  return 'D'
}

export const kdaRatio = (kills: number, deaths: number, assists: number) =>
  deaths > 0 ? (kills + assists) / deaths : kills + assists

/** 详情参与者等无数字 KDA 推评级（LCU 无官方 letter grade） */
export const gradeFromStats = (kills: number, deaths: number, assists: number) =>
  gradeFromKda(kdaRatio(kills, deaths, assists))

export const displayGrade = (game: Pick<MatchPerformance, 'grade' | 'kda'>) => game.grade || gradeFromKda(game.kda)

/** 表格/摘要用前景色（非衬底水印） */
export const gradeTextClass = (grade: string) => {
  switch (grade) {
    case 'S+':
      return 'text-orange-500 dark:text-orange-400'
    case 'S':
      return 'text-violet-500 dark:text-violet-300'
    case 'A':
      return 'text-emerald-600 dark:text-emerald-400'
    case 'B':
      return 'text-sky-600 dark:text-sky-400'
    case 'C':
      return 'text-stone-500 dark:text-stone-400'
    case 'D':
      return 'text-rose-600 dark:text-rose-400'
    default:
      return 'text-muted-foreground'
  }
}

/** 衬底字：S+ 亮橙金，S 紫；C 灰阶避免与金撞色 */
export const gradeWatermarkClass = (grade: string) => {
  switch (grade) {
    case 'S+':
      return 'text-orange-500/40 dark:text-orange-400/44'
    case 'S':
      return 'text-violet-500/35 dark:text-violet-300/40'
    case 'A':
      return 'text-emerald-600/26 dark:text-emerald-400/30'
    case 'B':
      return 'text-sky-600/26 dark:text-sky-400/30'
    case 'C':
      return 'text-stone-500/28 dark:text-stone-400/32'
    case 'D':
      return 'text-red-600/26 dark:text-red-400/30'
    default:
      return 'text-foreground/14'
  }
}

export const gradeWatermarkSizeClass = (grade: string) => {
  if (grade === 'S+') return 'text-[4.75rem] tracking-tighter'
  if (grade === 'S') return 'text-[4.5rem] tracking-tighter'
  return 'text-[3.75rem] tracking-tight'
}
