// 数据API模块
export * from './dataApi'
import type { CommunityDragonPerk, DDragonChampionsResponse, DDragonItemsResponse } from './dataApi'
import { getMapById, getQueueDisplayName } from '@/common'

// 主题配置模块
export * from './theme'
export * from './themeColor'

// 其他辅助函数
export const getPlayerProfileIcon = (participantId: number, gameDetail: GameDetail): number => {
  const identity = gameDetail.participants?.find((id) => id.participantId === participantId)
  // 兼容 profileIconId 可能为 bigint 的情况，强制转换为 number
  if (identity && identity.profileIconId !== undefined && identity.profileIconId !== null) {
    return Number(identity.profileIconId)
  }
  return 0
}

// 游戏相关的工具函数
export const formatGameMode = (mode: string): string => {
  const modeMap: Record<string, string> = {
    CLASSIC: '经典模式',
    ARAM: '大乱斗',
    URF: '无限火力',
    TUTORIAL: '教程',
    ONEFORALL: '克隆大作战',
    ARSR: '极地大乱斗',
    PRACTICETOOL: '训练工具',
    NEXUSBLITZ: '极地大乱斗'
  }
  return modeMap[mode] || mode
}

export const formatDuration = (seconds: number): string => {
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
}

export const getQueueName = (queueId: number): string => {
  // CDragon 中文名优先，本地兜底表其次
  return getQueueDisplayName(queueId)
}

export const getMapName = (mapId: number): string => {
  // 优先使用地图定义
  const map = getMapById(mapId)
  return map ? map.name : '未知地图'
}

export const formatNumber = (num: number): string => {
  return num?.toLocaleString() || '0'
}

/**
 * 根据英雄ID获取英雄图标URL
 * @param championId 英雄ID，number或string
 * @returns 英雄图标URL
 */
export const getChampionIconUrl = (championId: number | string | null): string => {
  if (!championId) return ''
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/champion-icons/${championId}.png`
}
/**
 * 根据英雄别名获取英雄图标URL
 * @param alias 英雄别名
 * @returns 英雄图标URL
 */
export const getChampionIconUrlByAlias = (alias: string): string => {
  if (!alias) return ''
  return `https://game.gtimg.cn/images/lol/act/img/champion/${alias}.png`
}
// 处理 Community Dragon 路径
export const getCommunityDragonUrl = (path: string): string => {
  if (!path) return ''
  // 移除开头的斜杠并详细完整URL
  const cleanPath = path.startsWith('/') ? path.slice(1) : path
  return `https://raw.communitydragon.org/latest/plugins/${cleanPath}`
}

/**
 * LCU / CDragon `iconPath` → 可访问的 Community Dragon CDN URL
 *
 * `/lol-game-data/assets/DATA/Spells/Icons2D/Summoner_flash.png`
 * → `.../rcp-be-lol-game-data/global/default/data/spells/icons2d/summoner_flash.png`
 */
export const getLolGameDataAssetUrl = (iconPath: string): string => {
  if (!iconPath) return ''
  if (iconPath.startsWith('http')) return iconPath

  const marker = '/lol-game-data/assets/'
  const idx = iconPath.indexOf(marker)
  if (idx >= 0) {
    const rest = iconPath.slice(idx + marker.length)
    return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/${rest.toLowerCase()}`
  }

  return ''
}

/** 符文 iconPath → CDN（复用通用资产路径转换） */
export const getPerkImageUrlFromIconPath = (iconPath: string, fallbackId?: number): string => {
  const fromPath = getLolGameDataAssetUrl(iconPath)
  if (fromPath) return fromPath
  if (fallbackId === null || fallbackId === undefined) return ''
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/perk-images/${fallbackId}.png`
}

// 根据符文ID获取符文图标URL（使用Community Dragon）
export const getPerkIconUrlByCommunityDragon = (perkId: number, perks: CommunityDragonPerk[]): string => {
  const perk = perks.find((p) => p.id === perkId)
  return getPerkImageUrlFromIconPath(perk?.iconPath ?? '', perkId)
}

/** 召唤师技能目录：id → { name, iconPath }（由 CDragon JSON 填充，勿手写死表） */
export type SummonerSpellCatalogEntry = {
  id: number
  name: string
  iconPath: string
}

let summonerSpellCatalog = new Map<number, SummonerSpellCatalogEntry>()

export const setSummonerSpellCatalog = (spells: Array<{ id: number | string; name?: string; iconPath?: string }>) => {
  const next = new Map<number, SummonerSpellCatalogEntry>()
  for (const spell of spells) {
    const id = Number(spell.id)
    // 过滤无效 / 占位 ID（如 4294967295）
    if (!Number.isFinite(id) || id <= 0 || id >= 0xffff_ffff) continue
    if (!spell.iconPath) continue
    next.set(id, {
      id,
      name: spell.name?.trim() || `技能${id}`,
      iconPath: spell.iconPath
    })
  }
  summonerSpellCatalog = next
}

export const getSummonerSpellCatalogEntry = (spellId: number): SummonerSpellCatalogEntry | undefined =>
  summonerSpellCatalog.get(spellId)

/**
 * 根据玩家头像ID获取头像URL
 * @param iconId 头像ID
 * @returns 头像URL
 */
export const getProfileIconUrl = (iconId: number): string => {
  if (!iconId) return ''
  return `https://raw.communitydragon.org/latest/plugins/rcp-be-lol-game-data/global/default/v1/profile-icons/${iconId}.jpg`
}

/**
 * 根据物品ID获取物品图标URL
 * @param itemId 物品ID
 * @param gameVersion 游戏版本
 * @returns 物品图标URL
 * @example
 * import { getItemIconUrl } from '@/lib'
 * const url = getItemIconUrl(3135, '12.23.1')
 * // https://ddragon.leagueoflegends.com/cdn/12.23.1/img/item/3135.png
 */
export const getItemIconUrl = (itemId: number, gameVersion: string): string => {
  if (!itemId || itemId === 0)
    return 'data:image/svg+xml;utf8,<svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg"><rect width="32" height="32" rx="6" fill="%23e5e7eb"/><text x="16" y="22" text-anchor="middle" font-size="20" fill="%239ca3af" font-family="Arial, Helvetica, sans-serif">?</text></svg>'
  return `https://ddragon.leagueoflegends.com/cdn/${gameVersion}/img/item/${itemId}.png`
}
/**
 * 根据物品ID获取物品图标URL (腾讯CDN)
 * @param itemId 物品ID
 * @param gameVersion 游戏版本
 * @returns 物品图标URL
 * @example
 * import { getItemIconByCdnUrl } from '@/lib'
 * const url = getItemIconByCdnUrl(3135, '12.23.1')
 * // https://game.gtimg.cn/images/lol/act/img/item/3135.png
 */
export const getItemIconByCdnUrl = (itemId: number): string => {
  if (!itemId || itemId === 0)
    return 'data:image/svg+xml;utf8,<svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg"><rect width="32" height="32" rx="6" fill="%23e5e7eb"/><text x="16" y="22" text-anchor="middle" font-size="20" fill="%239ca3af" font-family="Arial, Helvetica, sans-serif">?</text></svg>'
  return `https://game.gtimg.cn/images/lol/act/img/item/${itemId}.png`
}

/**
 * 段位小图标（Community Dragon ranked-mini-crests，TFT 套更完整，含 emerald）
 *
 * 注意：`ranked-emblem/emblem-*.png` 是带大片黑底的宽幅展示图，不适合 UI 图标。
 * @example getRankIconUrl('GOLD')
 * // .../images/ranked-mini-crests/gold_tft.svg
 */
export const getRankIconUrl = (tier: string): string => {
  if (!tier) return ''
  const tierLower = tier.toLowerCase().trim()
  if (tierLower === 'none') return ''
  // unranked 文件名是连字符：unranked-tft.svg；其余为 {tier}_tft.svg
  const file = tierLower === 'unranked' ? 'unranked-tft.svg' : `${tierLower}_tft.svg`
  return `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/images/ranked-mini-crests/${file}`
}

/**
 * 挑战水晶小图标（Community Dragon challenge-mini-crystal）
 * 资源无 emerald；遇到时回退到 platinum。
 */
export const getChallengeCrystalIconUrl = (level: string | null | undefined): string => {
  if (!level) return ''
  let tier = level.toLowerCase().trim()
  if (!tier || tier === 'none' || tier === 'unranked') return ''
  // challenge-mini-crystal 目录没有 emerald.svg
  if (tier === 'emerald') tier = 'platinum'
  return `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/challenge-mini-crystal/${tier}.svg`
}

/**
 * 分路 / 位置图标（Community Dragon honor/roleicon_*）
 * 兼容后端码（TOP/MID/ADC/SUPPORT）与 LCU 码（MIDDLE/BOTTOM/UTILITY）
 */
export const getRoleIconUrl = (position: string | null | undefined): string => {
  if (!position) return ''
  const key = position.toUpperCase().trim()
  const roleMap: Record<string, string> = {
    TOP: 'top',
    JUNGLE: 'jungle',
    MID: 'middle',
    MIDDLE: 'middle',
    ADC: 'bottom',
    BOTTOM: 'bottom',
    SUPPORT: 'utility',
    UTILITY: 'utility'
  }
  const role = roleMap[key]
  if (!role) return ''
  return `https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-static-assets/global/default/honor/roleicon_${role}.png`
}

// 时间相关函数
export const formatRelativeTime = (timestamp: number): string => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now.getTime() - date.getTime()
  const hours = Math.floor(diff / (1000 * 60 * 60))

  if (hours < 1) {
    return '刚刚'
  } else if (hours < 24) {
    return `${hours}小时前`
  } else {
    const days = Math.floor(hours / 24)
    return `${days}天前`
  }
}

// 游戏数据处理函数
export const getTeamResult = (teamId: string, teams: TeamInfo[]): string => {
  if (!teams) return ''
  const team = teams.find((t) => t.teamId?.toString() === teamId)
  if (!team) return ''
  return team.win === 'Win' ? '胜方' : '败方'
}

export const getTeamParticipants = (teamId: string, gameDetail: GameDetail): ParticipantInfo[] => {
  if (!gameDetail?.participants) return []
  return gameDetail.participants.filter((p) => p.teamId.toString() === teamId)
}

export const getTeamBans = (teamId: string, teams: TeamInfo[]): BanInfo[] => {
  if (!teams) return []
  const team = teams.find((t) => t.teamId?.toString() === teamId)
  return team?.bans || []
}

// 兼容旧版 Riot API 结构（不与当前 GameDetail 一一对应），仅供本函数内部使用
interface LegacyGameDetail {
  participantIdentities?: Array<{
    participantId: number
    player?: { gameName?: string; tagLine?: string; summonerName?: string }
  }>
}

export const getPlayerDisplayName = (participantId: number, gameDetail: LegacyGameDetail): string => {
  const identity = gameDetail.participantIdentities?.find((id) => id.participantId === participantId)
  if (!identity?.player) return '未知玩家'

  const { gameName, tagLine, summonerName } = identity.player
  if (gameName && tagLine) {
    return `${gameName}#${tagLine}`
  }
  return summonerName || '未知玩家'
}

/**
 * 根据英雄名称获取英雄ID
 * @param championName 英雄名称
 * @returns 英雄ID，如果未找到返回 null
 */
export const getChampionIdByName = (championName: string | null): number | null => {
  if (!championName) return null

  const championNameMap: Record<string, number> = {
    黑暗之女: 1,
    狂战士: 2,
    正义巨像: 3,
    卡牌大师: 4,
    德邦总管: 5,
    无畏战车: 6,
    诡术妖姬: 7,
    猩红收割者: 8,
    远古恐惧: 9,
    正义天使: 10,
    无极剑圣: 11,
    牛头酋长: 12,
    符文法师: 13,
    亡灵战神: 14,
    战争女神: 15,
    众星之子: 16,
    迅捷斥候: 17,
    麦林炮手: 18,
    祖安怒兽: 19,
    雪原双子: 20,
    赏金猎人: 21,
    寒冰射手: 22,
    蛮族之王: 23,
    武器大师: 24,
    堕落天使: 25,
    时光守护者: 26,
    炼金术士: 27,
    痛苦之拥: 28,
    瘟疫之源: 29,
    死亡颂唱者: 30,
    虚空恐惧: 31,
    殇之木乃伊: 32,
    披甲龙龟: 33,
    冰晶凤凰: 34,
    恶魔小丑: 35,
    祖安狂人: 36,
    琴瑟仙女: 37,
    虚空行者: 38,
    刀锋舞者: 39,
    风暴之怒: 40,
    海洋之灾: 41,
    英勇投弹手: 42,
    天启者: 43,
    瓦洛兰之盾: 44,
    邪恶小法师: 45,
    巨魔之王: 48,
    诺克萨斯统领: 50,
    皮城女警: 51,
    蒸汽机器人: 53,
    熔岩巨兽: 54,
    不祥之刃: 55,
    永恒梦魇: 56,
    扭曲树精: 57,
    荒漠屠夫: 58,
    德玛西亚皇子: 59,
    寡妇制造者: 60,
    盲僧: 67,
    复仇焰魂: 68,
    机械公敌: 69,
    暗夜猎手: 72,
    齐天大圣: 74,
    水晶先锋: 75,
    大发明家: 76,
    沙漠死神: 77,
    狂野女猎手: 78,
    兽灵行者: 79,
    圣锤之毅: 80,
    酒桶: 81,
    不屈之枪: 82,
    牧魂人: 83,
    离群之刺: 84,
    狂暴之心: 85,
    德玛西亚之力: 86,
    曙光女神: 89,
    虚空先知: 90,
    刀锋之影: 91,
    放逐之刃: 92,
    深渊巨口: 96,
    暮光之眼: 98,
    光辉女郎: 99,
    远古巫灵: 101,
    龙血武姬: 102,
    九尾妖狐: 103,
    法外狂徒: 104,
    潮汐海灵: 105,
    不灭狂雷: 106,
    傲之追猎者: 107,
    惩戒之箭: 110,
    机械先驱: 112,
    北地之怒: 113,
    无双剑姬: 114,
    爆破鬼才: 115,
    深海泰坦: 117,
    荣耀行刑官: 120,
    战争之影: 121,
    虚空掠夺者: 122,
    诺克萨斯之手: 126,
    未来守护者: 126,
    冰霜女巫: 127,
    皎月女神: 131,
    德玛西亚之翼: 133,
    暗黑元首: 134,
    铸星龙王: 136,
    影流之主: 137,
    暮光星灵: 141,
    荆棘之兴: 142,
    疾风剑豪: 157,
    虚空之女: 145,
    迷失之牙: 150,
    生化魔人: 154,
    山隐之焰: 155,
    暴怒骑士: 157,
    戏命师: 161,
    永猎双子: 203,
    诺提勒斯: 111,
    弗雷尔卓德之心: 201,
    河流之王: 223,
    岩雀: 163,
    青钢影: 164,
    影哨: 166,
    愁云使者: 200,
    封魔剑魂: 177,
    腕豪: 223,
    含羞蓓蕾: 166,
    灵罗娃娃: 234,
    炼金男爵: 233,
    虚空女皇: 233,
    不羁之悦: 221,
    祖安花火: 222,
    纳祖芒荣耀: 234,
    明烛: 235,
    百裂冥犬: 236,
    异画师: 237,
    炽炎雏龙: 238,
    血港鬼影: 240,
    涤魂圣枪: 241,
    残月之肃: 202,
    镕铁少女: 203,
    万花通灵: 203,
    幻翎: 201,
    逆羽: 201,
    圣枪游侠: 236
  }

  return championNameMap[championName] || null
}

export const getChampionName = (championId: number | string | null): string => {
  if (!championId) return '未选择英雄'
  const championMap: Record<number | string, string> = {
    '1': '黑暗之女',
    '2': '狂战士',
    '3': '正义巨像',
    '4': '卡牌大师',
    '5': '德邦总管',
    '6': '无畏战车',
    '7': '诡术妖姬',
    '8': '猩红收割者',
    '9': '远古恐惧',
    '10': '正义天使',
    '11': '无极剑圣',
    '12': '牛头酋长',
    '13': '符文法师',
    '14': '亡灵战神',
    '15': '战争女神',
    '16': '众星之子',
    '17': '迅捷斥候',
    '18': '麦林炮手',
    '19': '祖安怒兽',
    '20': '雪原双子',
    '21': '赏金猎人',
    '22': '寒冰射手',
    '23': '蛮族之王',
    '24': '武器大师',
    '25': '堕落天使',
    '26': '时光守护者',
    '27': '炼金术士',
    '28': '痛苦之拥',
    '29': '瘟疫之源',
    '30': '死亡颂唱者',
    '31': '虚空恐惧',
    '32': '殇之木乃伊',
    '33': '披甲龙龟',
    '34': '冰晶凤凰',
    '35': '恶魔小丑',
    '36': '祖安狂人',
    '37': '琴瑟仙女',
    '38': '虚空行者',
    '39': '刀锋舞者',
    '40': '风暴之怒',
    '41': '海洋之灾',
    '42': '英勇投弹手',
    '43': '天启者',
    '44': '瓦洛兰之盾',
    '45': '邪恶小法师',
    '48': '巨魔之王',
    '50': '诺克萨斯统领',
    '51': '皮城女警',
    '53': '蒸汽机器人',
    '54': '熔岩巨兽',
    '55': '不祥之刃',
    '56': '永恒梦魇',
    '57': '扭曲树精',
    '58': '荒漠屠夫',
    '59': '德玛西亚皇子',
    '60': '蜘蛛女皇',
    '61': '发条魔灵',
    '62': '齐天大圣',
    '63': '复仇焰魂',
    '64': '盲僧',
    '67': '暗夜猎手',
    '68': '机械公敌',
    '69': '魔蛇之拥',
    '72': '上古领主',
    '74': '大发明家',
    '75': '沙漠死神',
    '76': '狂野女猎手',
    '77': '兽灵行者',
    '78': '圣锤之毅',
    '79': '酒桶',
    '80': '不屈之枪',
    '81': '探险家',
    '82': '铁铠冥魂',
    '83': '牧魂人',
    '84': '离群之刺',
    '85': '狂暴之心',
    '86': '德玛西亚之力',
    '89': '曙光女神',
    '90': '虚空先知',
    '91': '刀锋之影',
    '92': '放逐之刃',
    '96': '深渊巨口',
    '98': '暮光之眼',
    '99': '光辉女郎',
    '101': '远古巫灵',
    '102': '龙血武姬',
    '103': '九尾妖狐',
    '104': '法外狂徒',
    '105': '潮汐海灵',
    '106': '不灭狂雷',
    '107': '傲之追猎者',
    '110': '惩戒之箭',
    '111': '深海泰坦',
    '112': '奥术先驱',
    '113': '北地之怒',
    '114': '无双剑姬',
    '115': '爆破鬼才',
    '117': '仙灵女巫',
    '119': '荣耀行刑官',
    '120': '战争之影',
    '121': '虚空掠夺者',
    '122': '诺克萨斯之手',
    '126': '未来守护者',
    '127': '冰霜女巫',
    '131': '皎月女神',
    '133': '德玛西亚之翼',
    '134': '暗黑元首',
    '136': '铸星龙王',
    '141': '影流之镰',
    '142': '暮光星灵',
    '143': '荆棘之兴',
    '145': '虚空之女',
    '147': '星籁歌姬',
    '150': '迷失之牙',
    '154': '生化魔人',
    '157': '疾风剑豪',
    '161': '虚空之眼',
    '163': '岩雀',
    '164': '青钢影',
    '166': '影哨',
    '200': '虚空女皇',
    '201': '弗雷尔卓德之心',
    '202': '戏命师',
    '203': '永猎双子',
    '221': '祖安花火',
    '222': '暴走萝莉',
    '223': '河流之王',
    '233': '狂厄蔷薇',
    '234': '破败之王',
    '235': '涤魂圣枪',
    '236': '圣枪游侠',
    '238': '影流之主',
    '240': '暴怒骑士',
    '245': '时间刺客',
    '246': '元素女皇',
    '254': '皮城执法官',
    '266': '暗裔剑魔',
    '267': '唤潮鲛姬',
    '268': '沙漠皇帝',
    '350': '魔法猫咪',
    '360': '沙漠玫瑰',
    '412': '魂锁典狱长',
    '420': '海兽祭司',
    '421': '虚空遁地兽',
    '427': '翠神',
    '429': '复仇之矛',
    '432': '星界游神',
    '497': '幻翎',
    '498': '逆羽',
    '516': '山隐之焰',
    '517': '解脱者',
    '518': '万花通灵',
    '523': '残月之肃',
    '526': '镕铁少女',
    '555': '血港鬼影',
    '711': '愁云使者',
    '777': '封魔剑魂',
    '799': '铁血狼母',
    '800': '流光镜影',
    '875': '腕豪',
    '876': '含羞蓓蕾',
    '887': '灵罗娃娃',
    '888': '炼金男爵',
    '893': '双界灵兔',
    '895': '不羁之悦',
    '897': '纳祖芒荣耀',
    '901': '炽炎雏龙',
    '902': '明烛',
    '910': '异画师',
    '950': '百裂冥犬',
    '804': '不破之誓'
  }
  return championMap[championId] || `英雄${championId}`
}

/** 召唤师技能图标：目录里的 iconPath → CDN（需先 setSummonerSpellCatalog） */
export const getSpellIconUrl = (spellId: number | null): string => {
  if (!spellId) return ''
  const entry = summonerSpellCatalog.get(Number(spellId))
  return entry ? getLolGameDataAssetUrl(entry.iconPath) : ''
}

/** 召唤师技能名称 + 图标（数据来自 CDragon summoner-spells.json，非本地死表） */
export const getSpellMeta = (spellId: number | bigint | null): { label: string; icon: string } => {
  if (spellId === null || spellId === undefined) return { label: '', icon: '' }
  const id = Number(spellId)
  if (!id) return { label: '', icon: '' }
  const entry = summonerSpellCatalog.get(id)
  if (!entry) return { label: `技能${id}`, icon: '' }
  return {
    label: entry.name,
    icon: getLolGameDataAssetUrl(entry.iconPath)
  }
}
/** 段位图标（Community Dragon，等同 getRankIconUrl） */
export const getTierIconUrl = (tier: string | undefined): string => {
  if (!tier) return ''
  return getRankIconUrl(tier)
}
// 获取最新版本号
export const getLatestVersion = async () => {
  const response = await fetch('https://ddragon.leagueoflegends.com/api/versions.json')
  const versions = await response.json()
  const latestVersion = versions[0]
  return latestVersion || '15.12.1'
}
// 获取所有符文详细信息（含描述等）
export const getAllRunes = async (gameVersion: string, language: string = 'zh_CN') => {
  const resp = await fetch(`https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/runesReforged.json`)
  return await resp.json()
}

// 获取所有召唤师技能详细信息
export const getAllSummonerSpells = async (gameVersion: string, language: string = 'zh_CN') => {
  const resp = await fetch(`https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/summoner.json`)
  return await resp.json()
}
// 获取所有英雄基础信息
export const getAllChampions = async (
  gameVersion: string,
  language: string = 'zh_CN'
): Promise<DDragonChampionsResponse> => {
  const resp = await fetch(`https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/champion.json`)
  return await resp.json()
}

// 获取单个英雄详细信息
export const getChampionDetail = async (
  championName: string,
  gameVersion: string,
  language: string = 'zh_CN'
): Promise<unknown> => {
  const resp = await fetch(
    `https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/champion/${championName}.json`
  )
  return await resp.json()
}
// 获取单个英雄详细信息根据id
export const getChampionInfoById = async (
  championId: number,
  gameVersion: string,
  language: string = 'zh_CN'
): Promise<unknown> => {
  const resp = await fetch(
    `https://raw.communitydragon.org/${gameVersion}/plugins/rcp-be-lol-game-data/global/${language}/v1/champions/${championId}.json`
  )
  return await resp.json()
}
// 获取所有物品数据
export const getAllItems = async (gameVersion: string, language: string = 'zh_CN'): Promise<DDragonItemsResponse> => {
  const resp = await fetch(`https://ddragon.leagueoflegends.com/cdn/${gameVersion}/data/${language}/item.json`)
  return await resp.json()
}
