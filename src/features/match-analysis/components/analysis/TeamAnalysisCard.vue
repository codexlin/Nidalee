<script setup lang="ts">
import type { UIPlayerData } from '@/types/match-analysis'

const props = defineProps<{
  players: UIPlayerData[]
  teamType: 'ally' | 'enemy'
  localPlayerCellId: number
  isPlayerRetrying?: (player: UIPlayerData) => boolean
}>()

defineEmits<{
  'select-player': [player: UIPlayerData]
  'retry-player': [player: UIPlayerData]
}>()

const enemyStatusMessage = computed(() => {
  if (props.teamType !== 'enemy') return ''
  const loading = props.players.filter((player) => !player.isBot && player.analysisStatus === 'loading').length
  return loading > 0 ? `已识别敌方玩家，正在分析 ${loading} 人的近期战绩` : ''
})
</script>

<template>
  <div class="flex h-full min-h-0 flex-col gap-1">
    <Card class="min-h-0 flex-1 border-none bg-transparent p-0 shadow-none">
      <div class="grid h-full min-h-0 grid-cols-5 gap-1.5">
        <CompactPlayerCard
          v-for="player in players"
          :key="`${teamType}-${player.cellId}`"
          :player="player"
          :is-local="player.cellId === localPlayerCellId"
          :is-ally="teamType === 'ally'"
          :retrying="isPlayerRetrying?.(player) ?? false"
          @select="$emit('select-player', player)"
          @retry="$emit('retry-player', player)"
        />
      </div>
    </Card>
    <div v-if="enemyStatusMessage" class="text-center text-[10px] text-muted-foreground">
      {{ enemyStatusMessage }}
    </div>
  </div>
</template>
