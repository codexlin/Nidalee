<template>
  <form class="overflow-hidden rounded-xl border" @submit.prevent="handleSave">
    <header class="flex flex-wrap items-center justify-between gap-3 border-b px-4 py-3 sm:px-5">
      <div class="flex min-w-0 items-center gap-3">
        <Button
          type="button"
          variant="ghost"
          size="icon"
          aria-label="返回配置列表"
          :disabled="saving"
          @click="handleClose"
        >
          <ArrowLeft />
        </Button>
        <div class="min-w-0">
          <h3 class="flex items-center gap-2 text-base font-semibold">
            <Sparkles class="size-4 text-primary" />
            {{ isEditing ? '编辑自定义符文' : '新建自定义符文' }}
          </h3>
          <p class="mt-0.5 text-xs text-muted-foreground">设置适用条件并完成一套 9 枚符文。</p>
        </div>
      </div>
      <div class="flex items-center gap-2">
        <Button type="button" variant="outline" :disabled="saving" @click="handleClose">返回列表</Button>
        <Button type="submit" :disabled="!isFormValid || saving">
          <Spinner v-if="saving" data-icon="inline-start" />
          {{ saving ? '保存中' : isEditing ? '保存修改' : '创建配置' }}
        </Button>
      </div>
    </header>

    <div class="grid lg:grid-cols-[300px_minmax(0,1fr)]">
      <aside class="border-b lg:border-r lg:border-b-0">
        <div class="flex flex-col gap-5 p-4 sm:p-5">
          <section class="flex flex-col gap-3">
            <div>
              <h3 class="text-sm font-semibold">适用条件</h3>
              <p class="mt-1 text-xs text-muted-foreground">越精确的配置会优先匹配。</p>
            </div>

            <div class="grid gap-2" role="radiogroup" aria-label="符文适用范围">
              <button
                v-for="option in scopeOptions"
                :key="option.value"
                type="button"
                role="radio"
                :aria-checked="formData.scope === option.value"
                :class="scopeOptionClass(option.value)"
                @click="formData.scope = option.value"
              >
                <span class="flex items-center justify-between gap-3">
                  <span class="text-sm font-medium">{{ option.label }}</span>
                  <Check v-if="formData.scope === option.value" class="size-4 text-primary" />
                </span>
                <span class="text-xs text-muted-foreground">{{ option.description }}</span>
              </button>
            </div>
          </section>

          <Separator />

          <section class="flex flex-col gap-4">
            <div class="flex flex-col gap-2">
              <Label for="config-name">配置名称</Label>
              <Input
                id="config-name"
                v-model="formData.name"
                maxlength="40"
                placeholder="例如：薇恩下路常用"
                :aria-invalid="nameInvalid"
              />
              <p class="text-xs text-muted-foreground">用于在配置列表中快速识别。</p>
            </div>

            <div v-if="needsChampion" class="relative flex flex-col gap-2">
              <Label for="champion-search">英雄</Label>
              <Input
                id="champion-search"
                v-model="championSearch"
                autocomplete="off"
                placeholder="输入英雄名称或英文名"
                :aria-invalid="championInvalid"
                @input="handleChampionInput"
                @keydown.esc="championSearchFocused = false"
                @focus="championSearchFocused = true"
              />
              <div
                v-if="showChampionResults"
                class="surface-overlay absolute top-full right-0 left-0 mt-1 overflow-hidden rounded-lg border shadow-lg"
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
                <p v-if="championSearchResults.length === 0" class="px-3 py-3 text-xs text-muted-foreground">
                  没有找到匹配的英雄
                </p>
              </div>
              <p v-if="formData.championId" class="flex items-center gap-1.5 text-xs text-muted-foreground">
                <CheckCircle2 class="size-3.5 text-primary" />
                已选择 {{ formData.championName }}
              </p>
              <p v-else class="text-xs text-muted-foreground">输入至少 1 个字符后选择英雄。</p>
            </div>

            <div v-if="needsPosition" class="flex flex-col gap-2">
              <Label for="position">位置</Label>
              <Select v-model:model-value="formData.position">
                <SelectTrigger id="position" :aria-invalid="positionInvalid">
                  <SelectValue placeholder="选择位置" />
                </SelectTrigger>
                <SelectContent>
                  <SelectGroup>
                    <SelectItem v-for="position in positions" :key="position.value" :value="position.value">
                      {{ position.label }}
                    </SelectItem>
                  </SelectGroup>
                </SelectContent>
              </Select>
            </div>

            <label class="flex cursor-pointer items-start gap-3 rounded-lg border p-3">
              <Checkbox v-model:checked="formData.isDefault" class="mt-0.5" />
              <span class="min-w-0">
                <span class="block text-sm font-medium">设为默认配置</span>
                <span class="mt-0.5 block text-xs text-muted-foreground">同样条件下优先使用这一套。</span>
              </span>
            </label>
          </section>
        </div>
      </aside>

      <div>
        <div class="flex flex-wrap items-center justify-between gap-3 border-b px-5 py-3">
          <div>
            <h3 class="text-sm font-semibold">符文方案</h3>
            <p class="mt-0.5 text-xs text-muted-foreground">手动选择，或从推荐与客户端快速载入。</p>
          </div>
          <div class="flex items-center gap-2">
            <Button type="button" variant="outline" size="sm" :disabled="isImporting" @click="handleImportFromOpgg">
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
          <RunePerkPicker
            :primary-style-id="formData.primaryStyleId"
            :sub-style-id="formData.subStyleId"
            :selected-perk-ids="formData.selectedPerkIds"
            @update:primary-style-id="formData.primaryStyleId = $event"
            @update:sub-style-id="formData.subStyleId = $event"
            @update:selected-perk-ids="formData.selectedPerkIds = $event"
          />
        </div>
      </div>
    </div>

    <footer class="flex flex-wrap items-center justify-between gap-3 border-t px-4 py-3 sm:px-5">
      <p class="text-xs text-muted-foreground">{{ formStatusText }}</p>
      <p class="text-xs text-muted-foreground">保存后将按适用条件参与自动匹配。</p>
    </footer>
  </form>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { toast } from 'vue-sonner'
import { ArrowLeft, Check, CheckCircle2, Download, FileInput, Sparkles } from 'lucide-vue-next'
import { cn } from '@/lib/utils'
import { useUserRuneStore, type RuneConfig } from '@/shared/stores/features/userRuneStore'
import RunePerkPicker from './RunePerkPicker.vue'

type RuneScope = RuneConfig['scope']

interface Props {
  config?: RuneConfig | null
  saving?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  config: null,
  saving: false
})

const emit = defineEmits<{
  close: []
  save: [config: RuneConfig]
}>()

const scopeOptions: Array<{ value: RuneScope; label: string; description: string }> = [
  { value: 'champion-position', label: '英雄 + 位置', description: '最精确，优先级最高' },
  { value: 'champion-all', label: '英雄通用', description: '该英雄在所有位置使用' },
  { value: 'position-all', label: '位置通用', description: '该位置的所有英雄使用' }
]

const positions = [
  { value: 'TOP', label: '上路' },
  { value: 'JUNGLE', label: '打野' },
  { value: 'MID', label: '中路' },
  { value: 'ADC', label: '下路' },
  { value: 'SUPPORT', label: '辅助' }
]

const { data: allChampionsData } = useChampions()
const userRuneStore = useUserRuneStore()
const championSearch = ref('')
const championSearchFocused = ref(false)
const importSource = ref<'opgg' | 'client' | null>(null)

const formData = reactive({
  name: '',
  championId: null as number | null,
  championName: null as string | null,
  position: null as string | null,
  scope: 'champion-position' as RuneScope,
  primaryStyleId: 8000,
  subStyleId: 8200,
  selectedPerkIds: [] as number[],
  isDefault: false,
  source: 'custom' as RuneConfig['source']
})

const isEditing = computed(() => props.config !== null)
const isImporting = computed(() => importSource.value !== null)
const needsChampion = computed(() => formData.scope !== 'position-all')
const needsPosition = computed(() => formData.scope !== 'champion-all')
const nameInvalid = computed(() => formData.name.trim().length === 0)
const championInvalid = computed(() => needsChampion.value && formData.championId === null)
const positionInvalid = computed(() => needsPosition.value && formData.position === null)

const championSearchResults = computed(() => {
  const query = championSearch.value.trim().toLowerCase()
  if (!query) return []
  return (allChampionsData.value ?? [])
    .filter((champion) => champion.name.toLowerCase().includes(query) || champion.alias.toLowerCase().includes(query))
    .slice(0, 8)
})

const showChampionResults = computed(
  () => championSearchFocused.value && championSearch.value.trim().length > 0 && !formData.championId
)

const isFormValid = computed(() => {
  if (!formData.name.trim()) return false
  if (needsChampion.value && formData.championId === null) return false
  if (needsPosition.value && formData.position === null) return false
  return formData.selectedPerkIds.length === 9
})

const formStatusText = computed(() => {
  if (!formData.name.trim()) return '请填写配置名称'
  if (needsChampion.value && formData.championId === null) return '请选择适用英雄'
  if (needsPosition.value && formData.position === null) return '请选择适用位置'
  if (formData.selectedPerkIds.length !== 9) return `符文尚未完整：${formData.selectedPerkIds.length} / 9`
  return '配置完整，可以保存'
})

const resetForm = () => {
  const config = props.config
  Object.assign(formData, {
    name: config?.name ?? '',
    championId: config?.championId ?? null,
    championName: config?.championName ?? null,
    position: config?.position ?? null,
    scope: config?.scope ?? 'champion-position',
    primaryStyleId: config?.primaryStyleId ?? 8000,
    subStyleId: config?.subStyleId ?? 8200,
    selectedPerkIds: [...(config?.selectedPerkIds ?? [])],
    isDefault: config?.isDefault ?? false,
    source: config?.source ?? 'custom'
  })
  championSearch.value = config?.championName ?? ''
  championSearchFocused.value = false
  importSource.value = null
}

watch(() => props.config, resetForm, { immediate: true })

const scopeOptionClass = (scope: RuneScope) =>
  cn(
    'flex flex-col gap-1 rounded-lg border px-3 py-2.5 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring',
    formData.scope === scope ? 'border-primary bg-primary/5' : 'border-border hover:bg-accent/50'
  )

const handleChampionInput = () => {
  if (championSearch.value !== formData.championName) {
    formData.championId = null
    formData.championName = null
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
  if (formData.championId === null || formData.position === null) {
    toast.error('请先选择英雄和位置')
    return
  }

  importSource.value = 'opgg'
  try {
    const build = await invoke<OpggChampionBuild>('get_opgg_champion_build', {
      region: 'kr',
      mode: 'ranked',
      championId: formData.championId,
      position: formData.position,
      tier: userRuneStore.autoApply.opggTier
    })
    const bestPerk = build.perks?.[0]
    if (!bestPerk || bestPerk.perks.length !== 9) {
      throw new Error('推荐数据不完整')
    }
    formData.primaryStyleId = bestPerk.primaryId
    formData.subStyleId = bestPerk.secondaryId
    formData.selectedPerkIds = [...bestPerk.perks]
    formData.source = 'opgg'
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
    if (!currentPage || currentPage.selectedPerkIds.length !== 9) {
      throw new Error('客户端当前没有完整符文页')
    }
    formData.primaryStyleId = currentPage.primaryStyleId
    formData.subStyleId = currentPage.subStyleId
    formData.selectedPerkIds = [...currentPage.selectedPerkIds]
    formData.source = 'import'
    toast.success('已载入客户端当前符文页')
  } catch (error) {
    toast.error(`载入失败：${error instanceof Error ? error.message : String(error)}`)
  } finally {
    importSource.value = null
  }
}

const handleSave = () => {
  if (!isFormValid.value || props.saving) return

  emit('save', {
    id: props.config?.id ?? crypto.randomUUID(),
    name: formData.name.trim(),
    championId: needsChampion.value ? formData.championId : null,
    championName: needsChampion.value ? formData.championName : null,
    position: needsPosition.value ? formData.position : null,
    scope: formData.scope,
    primaryStyleId: formData.primaryStyleId,
    subStyleId: formData.subStyleId,
    selectedPerkIds: [...formData.selectedPerkIds],
    isDefault: formData.isDefault,
    source: formData.source,
    createdAt: props.config?.createdAt ?? Date.now(),
    updatedAt: Date.now(),
    usageCount: props.config?.usageCount ?? 0
  })
}

const handleClose = () => {
  if (!props.saving) emit('close')
}
</script>
