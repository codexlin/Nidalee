/**
 * 运行时身份目录（英雄名 / 召唤师技能）
 * 响应式：模板内 getChampionName / getSpellMeta 会随 hydrate 自动刷新
 */
import { shallowRef } from 'vue'

export type SummonerSpellCatalogEntry = {
  id: number
  name: string
  iconPath: string
}

const championCatalog = shallowRef(new Map<number, string>())
const summonerSpellCatalog = shallowRef(new Map<number, SummonerSpellCatalogEntry>())

/** JADE/经典模式变体：CDragon id = 60000 + 本体 id */
const JADE_CHAMPION_ID_OFFSET = 60000

export const setChampionCatalog = (champions: Array<{ id: number | string; name?: string }>) => {
  const next = new Map<number, string>()
  for (const champ of champions) {
    const id = Number(champ.id)
    if (!Number.isFinite(id) || id <= 0) continue
    const name = champ.name?.trim()
    if (!name) continue
    next.set(id, name)
  }
  championCatalog.value = next
}

export const setSummonerSpellCatalog = (spells: Array<{ id: number | string; name?: string; iconPath?: string }>) => {
  const next = new Map<number, SummonerSpellCatalogEntry>()
  for (const spell of spells) {
    const id = Number(spell.id)
    if (!Number.isFinite(id) || id <= 0 || id >= 0xffff_ffff) continue
    if (!spell.iconPath) continue
    next.set(id, {
      id,
      name: spell.name?.trim() || `技能${id}`,
      iconPath: spell.iconPath
    })
  }
  summonerSpellCatalog.value = next
}

export const getSummonerSpellCatalogEntry = (spellId: number): SummonerSpellCatalogEntry | undefined =>
  summonerSpellCatalog.value.get(spellId)

/** 英雄中文名：目录 hydrate 后模板会因 shallowRef 替换而重渲染 */
export const getChampionName = (championId: number | string | null): string => {
  if (!championId && championId !== 0) return '未选择英雄'
  const id = Number(championId)
  if (!Number.isFinite(id) || id <= 0) return '未选择英雄'
  const catalog = championCatalog.value
  const direct = catalog.get(id)
  if (direct) return direct
  if (id >= JADE_CHAMPION_ID_OFFSET && id < JADE_CHAMPION_ID_OFFSET + 10000) {
    const base = catalog.get(id - JADE_CHAMPION_ID_OFFSET)
    if (base) return base
  }
  return `英雄${id}`
}

export const resolveChampionName = (
  championId: number | string | null | undefined,
  knownName?: string | null
): string => {
  const name = knownName?.trim()
  if (name) return name
  return getChampionName(championId ?? null)
}

/** 供测试 / 调试：当前目录 size */
export const getChampionCatalogSize = () => championCatalog.value.size
export const getSummonerSpellCatalogSize = () => summonerSpellCatalog.value.size
