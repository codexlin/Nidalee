<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { ArrowLeft, CheckCircle2, Download, FileInput, Sparkles } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { fetchOpggChampionBuild, fetchOpggTierList } from '@/lib/dataApi'
import {
  rankedPositionFromScenario,
  rankedScenarioFromPosition,
  validateRuneSelection,
  type BuildPosition,
  type BuildPreset,
  type BuildPresetSource,
  type BuildScenario
} from '@/shared/models/buildPreset'
import { useBuildPresetStore } from '@/shared/stores/features/buildPresetStore'
import { selectMainOpggPosition } from '@/shared/models/opggRecommendation'
import RuneSelectionPicker from './RuneSelectionPicker.vue'

type ScenarioKind = 'ranked' | 'normal-sr' | 'aram'

interface Props {
  preset?: BuildPreset | null
  initialChampion?: { championId: number; championName: string } | null
  saving?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  preset: null,
  initialChampion: null,
  saving: false
})

const emit = defineEmits<{
  close: []
  save: [preset: BuildPreset]
}>()

const positions: Array<{ value: BuildPosition; label: string }> = [
  { value: 'TOP', label: '上路' },
  { value: 'JUNGLE', label: '打野' },
  { value: 'MID', label: '中路' },
  { value: 'ADC', label: '下路' },
  { value: 'SUPPORT', label: '辅助' }
]

const scenarioKinds: Array<{ value: ScenarioKind; label: string; description: string }> = [
  { value: 'ranked', label: '排位赛', description: '单双排与灵活排位，按位置匹配' },
  { value: 'normal-sr', label: '匹配峡谷', description: '匹配与征召，不区分位置' },
  { value: 'aram', label: '极地大乱斗', description: '普通大乱斗专用方案' }
]

const { data: allChampionsData } = useChampions()
const presetStore = useBuildPresetStore()
const championSearch = shallowRef('')
const championSearchFocused = shallowRef(false)
const importSource = shallowRef<'opgg' | 'client' | null>(null)
const importedTargetKey = shallowRef<string | null>(null)
const targetChangedNotice = shallowRef(false)

const formData = reactive({
  name: '',
  championId: null as number | null,
  championName: '',
  scenarioKind: 'ranked' as ScenarioKind,
  position: 'JUNGLE' as BuildPosition,
  primaryStyleId: 8000,
  subStyleId: 8200,
  selectedPerkIds: [] as number[],
  autoUse: true,
  source: { kind: 'custom' } as BuildPresetSource
})

const isEditing = computed(() => props.preset !== null)
const isImporting = computed(() => importSource.value !== null)
const nameInvalid = computed(() => formData.name.trim().length === 0)
const championInvalid = computed(() => formData.championId === null)
const currentScenario = computed<BuildScenario>(() => {
  if (formData.scenarioKind === 'ranked') return rankedScenarioFromPosition(formData.position) ?? 'ranked-jungle'
  return formData.scenarioKind
})
const currentTargetKey = computed(() => `${formData.championId ?? 0}:${currentScenario.value}`)

const championSearchResults = computed(() => {
  const query = championSearch.value.trim().toLowerCase()
  if (!query) return []
  return (allChampionsData.value ?? [])
    .filter((champion) => champion.name.toLowerCase().includes(query) || champion.alias.toLowerCase().includes(query))
    .slice(0, 8)
})

const showChampionResults = computed(
  () => championSearchFocused.value && championSearch.value.trim().length > 0 && formData.championId === null
)

const runeError = computed(() =>
  validateRuneSelection({
    primaryStyleId: formData.primaryStyleId,
    subStyleId: formData.subStyleId,
    selectedPerkIds: formData.selectedPerkIds
  })
)

const isFormValid = computed(() => !nameInvalid.value && !championInvalid.value && runeError.value === null)
const formStatusText = computed(() => {
  if (nameInvalid.value) return '请填写方案名称'
  if (championInvalid.value) return '请选择适用英雄'
  if (runeError.value) return runeError.value
  return '方案完整，可以保存'
})

const toScenarioKind = (scenario: BuildScenario): ScenarioKind => {
  switch (scenario) {
    case 'normal-sr':
    case 'aram':
      return scenario
    default:
      return 'ranked'
  }
}

const resetForm = () => {
  const preset = props.preset
  const initialChampion = props.initialChampion
  const runes = preset?.components.runes
  const scenario = preset?.target.scenario ?? 'ranked-jungle'
  Object.assign(formData, {
    name: preset?.name ?? '',
    championId: preset?.target.championId ?? initialChampion?.championId ?? null,
    championName: preset?.target.championName ?? initialChampion?.championName ?? '',
    scenarioKind: toScenarioKind(scenario),
    position: rankedPositionFromScenario(scenario) ?? 'JUNGLE',
    primaryStyleId: runes?.primaryStyleId ?? 8000,
    subStyleId: runes?.subStyleId ?? 8200,
    selectedPerkIds: [...(runes?.selectedPerkIds ?? [])],
    autoUse: preset?.autoUse ?? true,
    source: preset ? { ...preset.source } : { kind: 'custom' }
  })
  championSearch.value = preset?.target.championName ?? initialChampion?.championName ?? ''
  championSearchFocused.value = false
  importSource.value = null
  importedTargetKey.value = preset && preset.source.kind !== 'custom' ? currentTargetKey.value : null
  targetChangedNotice.value = false
}

watch([() => props.preset, () => props.initialChampion], resetForm, { immediate: true })
watch(
  currentTargetKey,
  (nextTarget) => {
    if (!importedTargetKey.value || nextTarget === importedTargetKey.value) return
    formData.selectedPerkIds = []
    formData.source = { kind: 'custom' }
    importedTargetKey.value = null
    targetChangedNotice.value = true
  },
  { flush: 'sync' }
)

const handleChampionInput = () => {
  if (championSearch.value !== formData.championName) {
    formData.championId = null
    formData.championName = ''
  }
  championSearchFocused.value = true
}

const selectChampion = (champion: ChampionInfo) => {
  formData.championId = champion.id
  formData.championName = champion.name
  championSearch.value = champion.name
  championSearchFocused.value = false
}

const handleImportFromOpgg = async () => {
  if (formData.championId === null) {
    toast.error('请先选择英雄')
    return
  }
  const mode = formData.scenarioKind === 'aram' ? 'aram' : 'ranked'
  importSource.value = 'opgg'
  try {
    let position: BuildPosition | 'none' = formData.scenarioKind === 'ranked' ? formData.position : 'none'
    if (formData.scenarioKind === 'normal-sr') {
      const tierResponse = await fetchOpggTierList({
        region: 'kr',
        mode: 'ranked',
        tier: presetStore.autoBuild.opggTier
      })
      if (!tierResponse.success || !tierResponse.data) {
        throw new Error(tierResponse.error || '无法确定该英雄的主流位置')
      }
      const mainPosition = selectMainOpggPosition(tierResponse.data, formData.championId)
      if (!mainPosition) throw new Error('没有找到该英雄的主流位置')
      position = mainPosition
    }

    const response = await fetchOpggChampionBuild({
      region: 'kr',
      mode,
      champion_id: formData.championId,
      position,
      tier: presetStore.autoBuild.opggTier
    })
    if (!response.success || !response.data) throw new Error(response.error || '推荐数据暂不可用')
    const bestPerk = response.data.perks?.[0]
    if (!bestPerk || bestPerk.perks.length !== 9) throw new Error('推荐数据不完整')
    formData.primaryStyleId = bestPerk.primaryId
    formData.subStyleId = bestPerk.secondaryId
    formData.selectedPerkIds = [...bestPerk.perks]
    formData.source = {
      kind: 'opgg',
      provider: 'opgg',
      region: 'kr',
      mode,
      tier: presetStore.autoBuild.opggTier,
      capturedAt: Date.now()
    }
    importedTargetKey.value = currentTargetKey.value
    targetChangedNotice.value = false
    toast.success('已载入 OP.GG 推荐符文')
  } catch (error) {
    toast.error(`载入失败：${error instanceof Error ? error.message : String(error)}`)
  } finally {
    importSource.value = null
  }
}

const handleLoadFromClient = async () => {
  importSource.value = 'client'
  try {
    const currentPage = await invoke<RunePage | null>('get_current_rune_page')
    if (!currentPage || currentPage.selectedPerkIds.length !== 9) throw new Error('客户端当前没有完整符文页')
    formData.primaryStyleId = currentPage.primaryStyleId
    formData.subStyleId = currentPage.subStyleId
    formData.selectedPerkIds = [...currentPage.selectedPerkIds]
    formData.source = { kind: 'client', capturedAt: Date.now() }
    importedTargetKey.value = currentTargetKey.value
    targetChangedNotice.value = false
    toast.success('已载入客户端当前符文页')
  } catch (error) {
    toast.error(`载入失败：${error instanceof Error ? error.message : String(error)}`)
  } finally {
    importSource.value = null
  }
}

const handleSave = () => {
  if (!isFormValid.value || props.saving || formData.championId === null) return
  const now = Date.now()
  emit('save', {
    id: props.preset?.id ?? crypto.randomUUID(),
    name: formData.name.trim(),
    target: {
      championId: formData.championId,
      championName: formData.championName,
      scenario: currentScenario.value
    },
    components: {
      runes: {
        primaryStyleId: formData.primaryStyleId,
        subStyleId: formData.subStyleId,
        selectedPerkIds: [...formData.selectedPerkIds]
      }
    },
    source: { ...formData.source },
    autoUse: formData.autoUse,
    createdAt: props.preset?.createdAt ?? now,
    updatedAt: now,
    usageCount: props.preset?.usageCount ?? 0
  })
}

const handleClose = () => {
  if (!props.saving) emit('close')
}

const takeOwnershipOfSelection = () => {
  formData.source = { kind: 'custom' }
  targetChangedNotice.value = false
}

const handlePrimaryStyleChange = (styleId: number) => {
  takeOwnershipOfSelection()
  formData.primaryStyleId = styleId
}

const handleSubStyleChange = (styleId: number) => {
  takeOwnershipOfSelection()
  formData.subStyleId = styleId
}

const handleSelectedPerksChange = (perkIds: number[]) => {
  takeOwnershipOfSelection()
  formData.selectedPerkIds = perkIds
}
</script>

<template>
  <form @submit.prevent="handleSave">
    <Card class="gap-0 overflow-hidden py-0">
      <CardHeader
        class="flex-row flex-wrap items-center justify-between gap-3 border-b border-border/50 px-4 py-3 sm:px-5"
      >
        <div class="flex min-w-0 items-center gap-3">
          <Button
            type="button"
            variant="ghost"
            size="icon"
            aria-label="返回方案列表"
            :disabled="saving"
            @click="handleClose"
          >
            <ArrowLeft />
          </Button>
          <div class="min-w-0">
            <CardTitle class="flex items-center gap-2 text-base">
              <Sparkles class="size-4 text-primary" />
              {{ isEditing ? '编辑个人方案' : '新建个人方案' }}
            </CardTitle>
            <CardDescription class="mt-0.5 text-xs">为一个英雄和游戏场景保存完整符文。</CardDescription>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <Button type="button" variant="outline" :disabled="saving" @click="handleClose">取消</Button>
          <Button type="submit" :disabled="!isFormValid || saving">
            <Spinner v-if="saving" data-icon="inline-start" />
            {{ saving ? '保存中' : '保存方案' }}
          </Button>
        </div>
      </CardHeader>

      <CardContent class="grid p-0 lg:grid-cols-[300px_minmax(0,1fr)]">
        <aside class="border-b border-border/50 lg:border-r lg:border-b-0">
          <div class="flex flex-col gap-5 p-4 sm:p-5">
            <section class="flex flex-col gap-4">
              <div>
                <h3 class="text-sm font-semibold">适用目标</h3>
                <p class="mt-1 text-xs text-muted-foreground">每个方案固定对应一个英雄和场景。</p>
              </div>

              <div class="flex flex-col gap-1.5">
                <Label for="config-name">方案名称</Label>
                <Input
                  id="config-name"
                  v-model="formData.name"
                  maxlength="40"
                  placeholder="例如：皇子打野"
                  :aria-invalid="nameInvalid"
                />
              </div>

              <div class="relative flex flex-col gap-1.5">
                <Label for="champion-search">英雄</Label>
                <Input
                  id="champion-search"
                  v-model="championSearch"
                  autocomplete="off"
                  placeholder="搜索英雄名称"
                  :aria-invalid="championInvalid"
                  @input="handleChampionInput"
                  @keydown.esc="championSearchFocused = false"
                  @focus="championSearchFocused = true"
                />
                <div
                  v-if="showChampionResults"
                  class="surface-overlay absolute top-full right-0 left-0 mt-1 overflow-hidden rounded-xl border"
                >
                  <button
                    v-for="champion in championSearchResults"
                    :key="champion.id"
                    type="button"
                    class="flex w-full items-center justify-between gap-3 px-3 py-2 text-left text-sm transition-colors hover:bg-accent focus-visible:bg-accent focus-visible:outline-none"
                    @mousedown.prevent="selectChampion(champion)"
                  >
                    <span class="font-medium">{{ champion.name }}</span>
                    <span class="text-xs text-muted-foreground">{{ champion.alias }}</span>
                  </button>
                  <p v-if="!championSearchResults.length" class="px-3 py-3 text-xs text-muted-foreground">
                    没有找到匹配的英雄
                  </p>
                </div>
                <p v-if="formData.championId" class="flex items-center gap-1.5 text-xs text-muted-foreground">
                  <CheckCircle2 class="size-3.5 text-primary" />
                  已选择 {{ formData.championName }}
                </p>
              </div>

              <div class="flex flex-col gap-1.5">
                <Label for="scenario">游戏场景</Label>
                <Select v-model:model-value="formData.scenarioKind">
                  <SelectTrigger id="scenario"><SelectValue placeholder="选择场景" /></SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem
                        v-for="scenario in scenarioKinds"
                        :key="scenario.value"
                        :value="scenario.value"
                        :text-value="scenario.label"
                      >
                        <div>
                          <div class="text-sm font-medium">{{ scenario.label }}</div>
                          <div class="text-xs text-muted-foreground">{{ scenario.description }}</div>
                        </div>
                      </SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </div>

              <div v-if="formData.scenarioKind === 'ranked'" class="flex flex-col gap-1.5">
                <Label for="position">排位位置</Label>
                <Select v-model:model-value="formData.position">
                  <SelectTrigger id="position"><SelectValue placeholder="选择位置" /></SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem v-for="position in positions" :key="position.value" :value="position.value">{{
                        position.label
                      }}</SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </div>

              <label class="flex cursor-pointer items-start gap-3 rounded-xl surface-inset p-3">
                <Checkbox v-model:checked="formData.autoUse" class="mt-0.5" />
                <span class="min-w-0">
                  <span class="block text-sm font-medium">自动使用此方案</span>
                  <span class="mt-0.5 block text-xs text-muted-foreground"
                    >锁定该英雄并进入此场景时覆盖在线推荐；同一目标只保留一套。</span
                  >
                </span>
              </label>
            </section>
          </div>
        </aside>

        <div>
          <div class="flex flex-wrap items-center justify-between gap-3 border-b border-border/50 px-4 py-3 sm:px-5">
            <div>
              <h3 class="text-sm font-semibold">符文方案</h3>
              <p class="mt-0.5 text-xs text-muted-foreground">手动选择，或从推荐和客户端载入。</p>
            </div>
            <div class="flex items-center gap-2">
              <Button
                type="button"
                variant="outline"
                size="sm"
                :disabled="isImporting"
                title="载入 OP.GG 推荐"
                @click="handleImportFromOpgg"
              >
                <Spinner v-if="importSource === 'opgg'" data-icon="inline-start" />
                <Download v-else data-icon="inline-start" />
                OP.GG 推荐
              </Button>
              <Button type="button" variant="outline" size="sm" :disabled="isImporting" @click="handleLoadFromClient">
                <Spinner v-if="importSource === 'client'" data-icon="inline-start" />
                <FileInput v-else data-icon="inline-start" />
                当前客户端
              </Button>
            </div>
          </div>

          <div class="p-4 sm:p-5">
            <Alert v-if="targetChangedNotice" class="mb-4">
              <AlertTitle>目标已变更，请重新选择符文</AlertTitle>
              <AlertDescription>
                此前载入的符文属于另一个英雄或场景，已清除，避免保存后自动应用错误方案。
              </AlertDescription>
            </Alert>
            <RuneSelectionPicker
              :primary-style-id="formData.primaryStyleId"
              :sub-style-id="formData.subStyleId"
              :selected-perk-ids="formData.selectedPerkIds"
              @update:primary-style-id="handlePrimaryStyleChange"
              @update:sub-style-id="handleSubStyleChange"
              @update:selected-perk-ids="handleSelectedPerksChange"
            />
          </div>
        </div>
      </CardContent>

      <CardFooter class="flex flex-wrap items-center justify-between gap-3 border-t border-border/50 px-4 py-3 sm:px-5">
        <p class="text-xs text-muted-foreground">{{ formStatusText }}</p>
        <p class="text-xs text-muted-foreground">保存后可在方案列表中随时切换是否自动使用。</p>
      </CardFooter>
    </Card>
  </form>
</template>
