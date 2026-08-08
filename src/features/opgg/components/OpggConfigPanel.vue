<template>
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Settings class="w-5 h-5" />
        配置选项
      </CardTitle>
      <CardDescription>选择区域、模式、段位和英雄来获取推荐数据</CardDescription>
    </CardHeader>
    <CardContent>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
        <!-- 区域选择 -->
        <div class="space-y-2">
          <Label for="region">区域</Label>
          <Select v-model="localConfig.region">
            <SelectTrigger>
              <SelectValue placeholder="选择区域" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="region in regions" :key="region.value" :value="region.value">
                {{ region.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 模式选择 -->
        <div class="space-y-2">
          <Label for="mode">模式</Label>
          <Select v-model="localConfig.mode">
            <SelectTrigger>
              <SelectValue placeholder="选择模式" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="mode in modes" :key="mode.value" :value="mode.value">
                {{ mode.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 段位选择 -->
        <div class="space-y-2">
          <Label for="tier">段位</Label>
          <Select v-model="localConfig.tier">
            <SelectTrigger>
              <SelectValue placeholder="选择段位" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="tier in tiers" :key="tier.value" :value="tier.value">
                {{ tier.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 位置选择 (仅排位模式显示) -->
        <div v-if="localConfig.mode === 'ranked'" class="space-y-2">
          <Label for="position">位置</Label>
          <Select v-model="localConfig.position">
            <SelectTrigger>
              <SelectValue placeholder="选择位置" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="position in positions" :key="position.value" :value="position.value">
                {{ position.label }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <!-- 英雄选择（Combobox 官方用法） -->
        <div class="space-y-2">
          <Label for="championId">英雄</Label>
          <Combobox v-model="selectedChampion" by="value">
            <ComboboxAnchor as-child>
              <ComboboxTrigger as-child>
                <Button variant="outline" class="justify-between w-full">
                  {{ selectedChampion?.label ?? '选择英雄' }}
                  <ChevronsUpDown class="ml-2 h-4 w-4 shrink-0 opacity-50" />
                </Button>
              </ComboboxTrigger>
            </ComboboxAnchor>
            <ComboboxList>
              <div class="relative w-full max-w-sm items-center">
                <ComboboxInput
                  class="pl-9 focus-visible:ring-0 border-0 border-b rounded-none h-10"
                  placeholder="搜索英雄..."
                />
                <span class="absolute start-0 inset-y-0 flex items-center justify-center px-3">
                  <Search class="size-4 text-muted-foreground" />
                </span>
              </div>
              <ComboboxEmpty> 没有找到英雄 </ComboboxEmpty>
              <ScrollArea class="h-60 rounded-md border">
                <ComboboxGroup>
                  <ComboboxItem v-for="champion in championOptions" :key="champion.value" :value="champion">
                    {{ champion.label }}
                    <ComboboxItemIndicator>
                      <Check class="ml-auto h-4 w-4" />
                    </ComboboxItemIndicator>
                  </ComboboxItem>
                </ComboboxGroup>
              </ScrollArea>
            </ComboboxList>
          </Combobox>
        </div>
      </div>
    </CardContent>
  </Card>
</template>

<script setup lang="ts">
import { Check, ChevronsUpDown, Search, Settings } from 'lucide-vue-next'
import { getAllChampions } from '@/lib'
import type { DDragonChampion } from '@/lib/dataApi'

interface Props {
  config: {
    region: string
    mode: string
    tier: string
    position: string
    championId: number
  }
  regions: Array<{ value: string; label: string }>
  modes: Array<{ value: string; label: string }>
  tiers: Array<{ value: string; label: string }>
  positions: Array<{ value: string; label: string }>
}

interface ChampionOption {
  value: number
  label: string
}

interface Emits {
  (e: 'update:config', value: Props['config']): void
}
const selectedChampion = ref<ChampionOption | undefined>()
const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// 创建本地响应式配置对象
const localConfig = computed({
  get: () => props.config,
  set: (value) => emit('update:config', value)
})

// 获取游戏版本
const dataStore = useDataStore()
const gameVersion = computed(() => dataStore.gameVersion)

// 拉取所有英雄数据
const allChampions = ref<DDragonChampion[]>([])
onMounted(async () => {
  if (gameVersion.value) {
    const data = await getAllChampions(gameVersion.value)
    allChampions.value = Object.values(data.data || {})
  }
})

const championOptions = computed(() =>
  allChampions.value.map((champ) => ({
    value: Number(champ.key),
    label: champ.name
  }))
)

// 同步 v-model
watch(
  [() => localConfig.value.championId, championOptions],
  ([id, options]) => {
    if (!id || !options.length) return
    if (!selectedChampion.value || selectedChampion.value.value !== id) {
      const found = options.find((opt) => opt.value === id)
      if (found) selectedChampion.value = found
    }
  },
  { immediate: true }
)
watch(selectedChampion, (val) => {
  if (val && val.value !== localConfig.value.championId) {
    emit('update:config', { ...localConfig.value, championId: val.value })
  }
})
</script>
