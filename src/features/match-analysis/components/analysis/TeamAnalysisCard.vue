<template>
  <div class="flex h-full min-h-0 flex-col gap-1">
    <Card class="min-h-0 flex-1 border-none bg-transparent p-0 shadow-none">
      <div class="grid h-full min-h-0 grid-cols-5 gap-1.5">
        <CompactPlayerCard
          v-for="(player, index) in teamData.players"
          :key="(player.displayName || player.summonerId || player.puuid || index) + '-' + index"
          :player="player"
          :player-stats="getPlayerStats(index)"
          :is-local="player.cellId === teamData.localPlayerCellId"
          :is-ally="teamType === 'ally'"
          @select="$emit('select-player', player, getPlayerStats(index))"
        />
      </div>
    </Card>
    <div v-if="enemyStatusMessage" class="text-center text-[10px] text-muted-foreground">
      {{ enemyStatusMessage }}
    </div>
  </div>
</template>

<script setup lang="ts">
import type { EnrichedPlayerMatchStats, TeamData, UIPlayerData } from '@/types/match-analysis'

/** 战绩匹配时可能附带的标识字段 */
type MatchablePlayerStats = EnrichedPlayerMatchStats & {
  puuid?: string | null
  cellId?: number
}

const props = withDefaults(
  defineProps<{
    teamData: TeamData
    teamStats?: (MatchablePlayerStats | null)[]
    teamType: 'ally' | 'enemy'
    localPlayerCellId?: number | null
  }>(),
  {
    teamStats: () => []
  }
)

defineEmits<{
  'select-player': [player: UIPlayerData, stats: MatchablePlayerStats | null]
}>()

const enemyStatusMessage = computed(() => {
  if (props.teamType !== 'enemy') return ''
  const loading = props.teamData.players.filter((player) => !player.isBot && player.analysisStatus === 'loading').length
  if (loading > 0) return `已识别敌方玩家，正在分析 ${loading} 人的近期战绩`
  return ''
})

// 🔥 性能优化：预先匹配所有玩家的战绩，避免重复计算
const playerStatsMap = computed(() => {
  const teamStats = props.teamStats ?? []
  if (teamStats.length === 0) {
    return new Map<number, MatchablePlayerStats>()
  }

  const map = new Map<number, MatchablePlayerStats>()

  props.teamData.players.forEach((player, index) => {
    if (!player) return

    // 0. 与 players 同序的槽位（store 已按队伍顺序保留 null 槽）
    const byIndex = teamStats[index]
    if (byIndex) {
      map.set(index, byIndex)
      return
    }

    // 通过 puuid, displayName 或 cellId 匹配战绩
    const matchedStats = teamStats.find((stats) => {
      if (!stats) return false

      // 1. 优先通过 puuid 匹配 (最可靠)
      if (player.puuid && stats.puuid) {
        return player.puuid === stats.puuid
      }

      // 2. 备选：通过 displayName 匹配 (兼容旧数据或 puuid 缺失的情况)
      if (stats.displayName && player.displayName) {
        return stats.displayName.toLowerCase() === player.displayName.toLowerCase()
      }

      // 3. 备选：通过 cellId 匹配 (仅限当前对局)
      if (stats.cellId !== undefined && player.cellId !== undefined) {
        return stats.cellId === player.cellId
      }

      return false
    })

    if (matchedStats) {
      map.set(index, matchedStats)
    }
  })

  return map
})

// 🔥 优化后：直接从缓存的 Map 中获取（不会重复计算）
const getPlayerStats = (index: number): MatchablePlayerStats | null => {
  return playerStatsMap.value.get(index) || null
}
</script>
