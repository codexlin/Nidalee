<template>
  <div v-if="embedded" class="space-y-2">
    <div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
      <Select v-model="rankQueue">
        <SelectTrigger class="h-9 w-full text-sm sm:w-[8.5rem]">
          <SelectValue placeholder="队列" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="q in rankQueues" :key="q.value" :value="q.value">
            {{ q.label }}
          </SelectItem>
        </SelectContent>
      </Select>

      <Select v-model="rankTier">
        <SelectTrigger class="h-9 w-full text-sm sm:w-[11rem]">
          <span class="flex min-w-0 items-center gap-2">
            <img v-if="getTierIconUrl(rankTier)" :src="getTierIconUrl(rankTier)" alt="" class="size-5 shrink-0" />
            <span class="truncate">{{ tierLabelMap[rankTier] || rankTier }}</span>
          </span>
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="t in rankTiers" :key="t" :value="t">
            <div class="flex items-center gap-2">
              <img v-if="getTierIconUrl(t)" :src="getTierIconUrl(t)" alt="" class="size-5" />
              <span>{{ tierLabelMap[t] || t }}</span>
            </div>
          </SelectItem>
        </SelectContent>
      </Select>

      <Select v-model="rankDivision" :disabled="noDivisionTiers.includes(rankTier)">
        <SelectTrigger class="h-9 w-full text-sm sm:w-[5.5rem]">
          <SelectValue placeholder="小段" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem v-for="d in rankDivisions" :key="d" :value="d">
            {{ d }}
          </SelectItem>
        </SelectContent>
      </Select>

      <Button class="h-9 shrink-0" :disabled="updatingRank" @click="handleSave">
        {{ updatingRank ? '保存中…' : '保存段位' }}
      </Button>
    </div>
    <p v-if="noDivisionTiers.includes(rankTier)" class="text-xs text-muted-foreground">
      大师、宗师、王者无小段位，自动设为 I
    </p>
  </div>

  <Card v-else class="gap-0 py-0">
    <CardHeader class="gap-1 px-4 py-3 sm:px-5">
      <CardTitle class="flex items-center gap-2 text-base font-medium">
        <Trophy class="size-4 shrink-0 text-muted-foreground" />
        段位设置
      </CardTitle>
      <p class="text-xs text-muted-foreground">自定义你的段位信息</p>
    </CardHeader>
    <CardContent class="space-y-2 px-4 pb-4 sm:px-5">
      <div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
        <Select v-model="rankQueue">
          <SelectTrigger class="h-9 w-full text-sm sm:w-[8.5rem]">
            <SelectValue placeholder="队列" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="q in rankQueues" :key="q.value" :value="q.value">
              {{ q.label }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Select v-model="rankTier">
          <SelectTrigger class="h-9 w-full text-sm sm:w-[11rem]">
            <span class="flex min-w-0 items-center gap-2">
              <img v-if="getTierIconUrl(rankTier)" :src="getTierIconUrl(rankTier)" alt="" class="size-5 shrink-0" />
              <span class="truncate">{{ tierLabelMap[rankTier] || rankTier }}</span>
            </span>
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="t in rankTiers" :key="t" :value="t">
              <div class="flex items-center gap-2">
                <img v-if="getTierIconUrl(t)" :src="getTierIconUrl(t)" alt="" class="size-5" />
                <span>{{ tierLabelMap[t] || t }}</span>
              </div>
            </SelectItem>
          </SelectContent>
        </Select>
        <Select v-model="rankDivision" :disabled="noDivisionTiers.includes(rankTier)">
          <SelectTrigger class="h-9 w-full text-sm sm:w-[5.5rem]">
            <SelectValue placeholder="小段" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="d in rankDivisions" :key="d" :value="d">
              {{ d }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Button class="h-9 shrink-0" :disabled="updatingRank" @click="handleSave">
          {{ updatingRank ? '保存中…' : '保存段位' }}
        </Button>
      </div>
      <p v-if="noDivisionTiers.includes(rankTier)" class="text-xs text-muted-foreground">
        大师、宗师、王者无小段位，自动设为 I
      </p>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { Trophy } from 'lucide-vue-next'
import { getTierIconUrl } from '@/lib'
import { useGameHelper } from '../composables/useGameHelper'

withDefaults(
  defineProps<{
    embedded?: boolean
  }>(),
  { embedded: false }
)

const rankQueues = [
  { label: '单双排', value: 'RANKED_SOLO_5x5' },
  { label: '灵活组排', value: 'RANKED_FLEX_SR' }
]
const rankTiers = [
  'IRON',
  'BRONZE',
  'SILVER',
  'GOLD',
  'PLATINUM',
  'EMERALD',
  'DIAMOND',
  'MASTER',
  'GRANDMASTER',
  'CHALLENGER'
]
const rankDivisions = ['I', 'II', 'III', 'IV']
const noDivisionTiers = ['MASTER', 'GRANDMASTER', 'CHALLENGER']
const tierLabelMap: Record<string, string> = {
  IRON: '坚韧黑铁',
  BRONZE: '英勇青铜',
  SILVER: '不屈白银',
  GOLD: '荣耀黄金',
  PLATINUM: '华贵铂金',
  EMERALD: '流光翡翠',
  DIAMOND: '璀璨钻石',
  MASTER: '超凡大师',
  GRANDMASTER: '傲世宗师',
  CHALLENGER: '最强王者'
}

const rankQueue = ref('RANKED_SOLO_5x5')
const rankTier = ref('GOLD')
const rankDivision = ref('IV')
const { setSummonerChatProfile, updatingRank } = useGameHelper()

watch(rankTier, (newTier) => {
  if (noDivisionTiers.includes(newTier)) {
    rankDivision.value = 'I'
  }
})

const handleSave = async () => {
  await setSummonerChatProfile({
    queue: rankQueue.value,
    tier: rankTier.value,
    division: rankDivision.value
  })
}
</script>
