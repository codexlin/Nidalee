/** 与后端 ModeAffinityTraitStrategy 对齐：按当前范围样本推断模式身份特征。 */

const AFFINITY_RATIO = 0.6
const AFFINITY_MIN_GAMES = 3
const RANKED_QUEUE_IDS = new Set([420, 440])

function countQueue(games: MatchPerformance[], queueId: number): number {
  return games.filter((g) => Number(g.queueId) === queueId).length
}

function makeTrait(key: string, name: string, count: number, total: number, description: string): DeterministicTrait {
  return {
    key,
    name,
    description,
    sentiment: 'neutral',
    sampleCount: count,
    frequency: total > 0 ? count / total : 0,
    confidence: 'medium',
    supportsConclusion: true,
    evidenceGameIds: []
  }
}

function passes(count: number, total: number): boolean {
  return count >= AFFINITY_MIN_GAMES && count / total >= AFFINITY_RATIO
}

/**
 * 在「其他」桶等场景下，用对局列表本地推断模式亲和（不依赖搜索页未投影的 traits）。
 */
export function inferModeAffinityTraits(games: MatchPerformance[] | null | undefined): DeterministicTrait[] {
  const list = games || []
  const total = list.length
  if (total < AFFINITY_MIN_GAMES) return []

  const hextech = countQueue(list, 2400)
  const aram = countQueue(list, 450)
  const ranked = list.filter((g) => RANKED_QUEUE_IDS.has(Number(g.queueId))).length
  const fun = total - ranked

  if (passes(hextech, total)) {
    const pct = Math.round((hextech / total) * 100)
    return [
      makeTrait(
        'mode_affinity_hextech',
        '海克斯常驻',
        hextech,
        total,
        `最近 ${hextech}/${total} 场都在海克斯大乱斗（${pct}%），已经玩出肌肉记忆了。`
      )
    ]
  }

  if (passes(aram, total)) {
    const pct = Math.round((aram / total) * 100)
    return [
      makeTrait(
        'mode_affinity_aram',
        '乱斗选手',
        aram,
        total,
        `最近 ${aram}/${total} 场泡在极地大乱斗（${pct}%），乱斗魂拉满。`
      )
    ]
  }

  if (passes(fun, total)) {
    const pct = Math.round((fun / total) * 100)
    return [
      makeTrait(
        'mode_affinity_fun',
        '娱乐为主',
        fun,
        total,
        `最近 ${fun}/${total} 场都是娱乐局（${pct}%），排位可以先放一放。`
      )
    ]
  }

  if (passes(ranked, total)) {
    const pct = Math.round((ranked / total) * 100)
    return [
      makeTrait(
        'mode_affinity_ranked',
        '排位为主',
        ranked,
        total,
        `最近 ${ranked}/${total} 场在打排位（${pct}%），认真上分的状态。`
      )
    ]
  }

  // 主导具名队列兜底（与后端 dominant_named_queue 同思路，仅覆盖常见娱乐）
  const named: { id: number; key: string; name: string }[] = [
    { id: 1700, key: 'mode_affinity_queue_1700', name: '斗魂竞技场' },
    { id: 900, key: 'mode_affinity_queue_900', name: '无限火力' },
    { id: 1900, key: 'mode_affinity_queue_1900', name: '无限火力' },
    { id: 430, key: 'mode_affinity_queue_430', name: '匹配模式' },
    { id: 400, key: 'mode_affinity_queue_400', name: '灵活匹配' }
  ]
  for (const q of named) {
    const count = countQueue(list, q.id)
    if (passes(count, total)) {
      const pct = Math.round((count / total) * 100)
      return [makeTrait(q.key, q.name, count, total, `最近 ${count}/${total} 场在打${q.name}（${pct}%）。`)]
    }
  }

  return []
}
