<template>
  <div class="surface-raised overflow-hidden">
    <div class="flex items-center gap-2 px-4 py-3 border-b border-border/60">
      <span class="w-1 self-stretch rounded-full shrink-0" :class="won ? 'bg-emerald-600' : 'bg-rose-600'" />
      <span class="text-base font-semibold">{{ title }}</span>
      <span
        class="h-6 px-2 inline-flex items-center rounded-md text-sm font-medium text-white"
        :class="won ? 'bg-emerald-600' : 'bg-rose-600'"
      >
        {{ won ? '胜' : '负' }}
      </span>
      <div v-if="bans.length" class="ml-auto flex items-center gap-1.5">
        <span class="text-sm text-muted-foreground mr-0.5">BAN</span>
        <img
          v-for="(ban, banIndex) in bans"
          :key="ban.championId ?? `${teamId}-ban-${banIndex}`"
          :src="getChampionIconUrl(ban.championId)"
          class="h-7 w-7 rounded opacity-80"
          :title="getChampionName(ban.championId)"
          alt=""
        />
      </div>
    </div>

    <div class="overflow-x-auto">
      <table class="w-full text-sm table-fixed">
        <thead>
          <tr class="text-sm text-muted-foreground border-b border-border/40">
            <th class="text-left font-medium px-3 py-2.5 w-[22%]">召唤师</th>
            <th class="text-left font-medium px-2 py-2.5 w-[16%]">英雄</th>
            <th class="text-center font-medium px-2 py-2.5 w-[10%]">天赋/技能</th>
            <th class="text-center font-medium px-2 py-2.5 w-[22%]">装备</th>
            <th class="text-center font-medium px-2 py-2.5 w-[10%]">KDA</th>
            <th class="text-center font-medium px-2 py-2.5 w-[8%]">经济</th>
            <th class="text-center font-medium px-2 py-2.5 w-[8%]">伤害</th>
            <th class="text-center font-medium px-2 py-2.5 w-[4%]">评级</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="participant in participants"
            :key="participant.participantId"
            class="border-b border-border/30 last:border-0"
            :class="participant.participantId === myParticipantId ? 'bg-primary/5' : 'hover:bg-muted/30'"
          >
            <td class="px-3 py-2.5">
              <div class="flex items-center gap-2.5 min-w-0">
                <img
                  :src="getProfileIconUrl(participant.profileIconId)"
                  class="h-9 w-9 rounded-full shrink-0 ring-1 ring-border/50"
                  alt=""
                />
                <button
                  type="button"
                  class="min-w-0 truncate text-left text-sm font-medium hover:text-foreground text-foreground/90"
                  :title="participant.summonerName"
                  @click="emit('open-summoner', participant)"
                >
                  <span>{{ displayName(participant.summonerName).name }}</span>
                  <span v-if="displayName(participant.summonerName).tag" class="text-muted-foreground font-normal"
                    >#{{ displayName(participant.summonerName).tag }}</span
                  >
                </button>
                <button
                  type="button"
                  class="shrink-0 text-muted-foreground hover:text-foreground p-0.5"
                  title="复制召唤师名"
                  @click="emit('copy-name', participant.summonerName)"
                >
                  <Copy class="h-4 w-4" />
                </button>
              </div>
            </td>
            <td class="px-2 py-2.5">
              <div class="flex items-center gap-2 min-w-0">
                <div class="relative shrink-0">
                  <img
                    :src="getChampionIconUrl(participant.championId)"
                    class="h-9 w-9 rounded-full"
                    :title="getChampionName(participant.championId)"
                    alt=""
                  />
                  <span
                    class="absolute -bottom-0.5 -right-0.5 bg-background text-foreground text-xs min-w-[18px] h-4 px-1 flex items-center justify-center rounded ring-1 ring-border tabular-nums leading-none"
                  >
                    {{ participant.stats?.champLevel || '?' }}
                  </span>
                </div>
                <span class="truncate text-sm font-medium">{{ getChampionName(participant.championId) }}</span>
              </div>
            </td>
            <td class="px-2 py-2.5">
              <div class="flex items-center justify-center gap-1.5">
                <div class="relative size-7 shrink-0" :title="primaryStyleLabel(participant)">
                  <img
                    v-if="primaryStyleIcon(participant)"
                    :src="primaryStyleIcon(participant)"
                    class="size-7 rounded bg-muted/40"
                    alt=""
                  />
                  <div v-else class="size-7 rounded bg-muted/40" />
                  <img
                    v-if="keystoneIcon(participant)"
                    :src="keystoneIcon(participant)"
                    class="absolute -bottom-0.5 -right-0.5 size-4 rounded-full ring-1 ring-background bg-background"
                    alt=""
                  />
                </div>
                <div class="flex flex-col gap-0.5">
                  <div
                    v-for="(spellId, idx) in spellIds(participant)"
                    :key="idx"
                    class="size-4 rounded overflow-hidden bg-muted/40 ring-1 ring-border/40"
                  >
                    <img
                      v-if="spellId && getSpellMeta(spellId).icon"
                      :src="getSpellMeta(spellId).icon"
                      :alt="getSpellMeta(spellId).label"
                      :title="getSpellMeta(spellId).label"
                      class="size-full object-cover"
                    />
                  </div>
                </div>
              </div>
            </td>
            <td class="px-2 py-2.5">
              <div class="flex items-center justify-center gap-1">
                <img
                  v-for="i in itemSlots"
                  :key="i"
                  :src="getItemIconUrl(itemId(participant, i) || 0, gameVersion)"
                  class="h-6 w-6 rounded bg-muted/40"
                  :class="itemId(participant, i) ? 'opacity-100' : 'opacity-30'"
                  alt=""
                />
              </div>
            </td>
            <td class="px-2 py-2.5 text-center font-mono tabular-nums text-sm font-medium">
              <span class="text-red-500">{{ participant.stats?.kills }}</span>
              <span class="text-muted-foreground/50">/</span>
              <span class="text-muted-foreground">{{ participant.stats?.deaths }}</span>
              <span class="text-muted-foreground/50">/</span>
              <span class="text-blue-500">{{ participant.stats?.assists }}</span>
            </td>
            <td class="px-2 py-2.5 text-center tabular-nums text-sm font-medium">
              {{ formatNumber(participant.stats?.goldEarned || 0) }}
            </td>
            <td class="px-2 py-2.5 text-center tabular-nums text-sm font-medium">
              {{ formatNumber(participant.stats?.totalDamageDealtToChampions || 0) }}
            </td>
            <td
              class="px-2 py-2.5 text-center text-base font-bold tabular-nums"
              :class="gradeTextClass(participantGrade(participant))"
            >
              {{ participantGrade(participant) }}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Copy } from 'lucide-vue-next'
import {
  getChampionIconUrl,
  getChampionName,
  getItemIconUrl,
  getPerkIconUrlByCommunityDragon,
  getPerkImageUrlFromIconPath,
  getProfileIconUrl,
  getSpellMeta
} from '@/lib'
import { gradeFromStats, gradeTextClass } from '../../utils/matchGrade'

defineProps<{
  title: string
  teamId: string
  won: boolean
  bans: BanInfo[]
  participants: ParticipantInfo[]
  myParticipantId: number | null
  gameVersion: string
}>()

const emit = defineEmits<{
  (e: 'open-summoner', participant: ParticipantInfo): void
  (e: 'copy-name', name: string): void
}>()

/** 与 OP.GG RunesCard / 符文设置页同一套图标解析；技能目录由 App 初始化预加载 */
const { data: communityPerks } = useCommunityDragonPerksQuery()
useSummonerSpells()
const runeData = useRuneData()

onMounted(() => {
  void runeData.loadRuneData()
})

const itemSlots = [0, 1, 2, 3, 4, 5, 6] as const

const itemId = (participant: ParticipantInfo, slot: number) => {
  const stats = participant.stats as ParticipantStats & Record<string, number | null | undefined>
  return stats[`item${slot}`] ?? 0
}

const spellIds = (participant: ParticipantInfo): [number, number] => [
  participant.spell1Id ?? 0,
  participant.spell2Id ?? 0
]

const displayName = (full: string) => {
  const idx = full.lastIndexOf('#')
  if (idx <= 0 || idx >= full.length - 1) return { name: full, tag: '' }
  return { name: full.slice(0, idx), tag: full.slice(idx + 1) }
}

const primaryStyleIcon = (participant: ParticipantInfo) => {
  const id = participant.perkPrimaryStyle
  if (!id) return ''
  const style = runeData.getPerkStyleById(id)
  return getPerkImageUrlFromIconPath(style?.iconPath ?? '', id)
}

const primaryStyleLabel = (participant: ParticipantInfo) => {
  const id = participant.perkPrimaryStyle
  if (!id) return '主系'
  return runeData.getPerkStyleById(id)?.name ?? `主系 ${id}`
}

const keystoneIcon = (participant: ParticipantInfo) => {
  const perkId = participant.perk0
  if (!perkId) return ''
  return getPerkIconUrlByCommunityDragon(perkId, communityPerks.value ?? [])
}

const participantGrade = (participant: ParticipantInfo) =>
  gradeFromStats(participant.stats?.kills ?? 0, participant.stats?.deaths ?? 0, participant.stats?.assists ?? 0)

const formatNumber = (num: number) => num.toLocaleString()
</script>
