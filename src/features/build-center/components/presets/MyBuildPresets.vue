<script setup lang="ts">
import { useTemplateRef } from 'vue'
import { FileInput, Plus, Search, Sparkles, Upload } from 'lucide-vue-next'
import { toast } from 'vue-sonner'
import { getChampionIconUrl } from '@/lib'
import { cn } from '@/lib/utils'
import { BUILD_SCENARIOS, scenarioLabel, type BuildPreset, type BuildScenario } from '@/shared/models/buildPreset'
import { useBuildApplication } from '@/shared/composables/game/useBuildApplication'
import { useBuildPresetStore } from '@/shared/stores/features/buildPresetStore'
import BuildPresetEditor from './BuildPresetEditor.vue'

interface PresetGroup {
  championId: number
  championName: string
  presets: BuildPreset[]
  updatedAt: number
}

const presetStore = useBuildPresetStore()
const { applyPreset } = useBuildApplication()
const importInput = useTemplateRef<HTMLInputElement>('importInput')

const searchQuery = shallowRef('')
const scenarioFilter = shallowRef<'all' | BuildScenario>('all')
const showEditor = shallowRef(false)
const editingPreset = shallowRef<BuildPreset | null>(null)
const isSaving = shallowRef(false)
const isDeleting = shallowRef(false)
const applyingPresetId = shallowRef<string | null>(null)
const autoUsePresetId = shallowRef<string | null>(null)
const deleteDialogOpen = shallowRef(false)
const pendingDeletePreset = shallowRef<BuildPreset | null>(null)
const selectedChampionId = shallowRef<number | null>(null)
const initialChampion = shallowRef<{ championId: number; championName: string } | null>(null)

const presets = computed(() => presetStore.presets)
const presetCount = computed(() => presetStore.presetCount)
const autoBuildEnabled = computed(() => presetStore.autoBuild.enabled)
const filteredPresets = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  return presets.value.filter((preset) => {
    const matchesSearch =
      !query || preset.name.toLowerCase().includes(query) || preset.target.championName.toLowerCase().includes(query)
    const matchesScenario = scenarioFilter.value === 'all' || preset.target.scenario === scenarioFilter.value
    return matchesSearch && matchesScenario
  })
})
const groupedPresets = computed<PresetGroup[]>(() => {
  const groups = new Map<number, PresetGroup>()
  for (const preset of filteredPresets.value) {
    const current = groups.get(preset.target.championId)
    if (current) current.presets.push(preset)
    else {
      groups.set(preset.target.championId, {
        championId: preset.target.championId,
        championName: preset.target.championName,
        presets: [preset],
        updatedAt: preset.updatedAt
      })
    }
  }
  return [...groups.values()]
    .map((group) => ({
      ...group,
      updatedAt: Math.max(...group.presets.map((preset) => preset.updatedAt)),
      presets: [...group.presets].sort(
        (left, right) => Number(right.autoUse) - Number(left.autoUse) || right.updatedAt - left.updatedAt
      )
    }))
    .sort(
      (left, right) => right.updatedAt - left.updatedAt || left.championName.localeCompare(right.championName, 'zh-CN')
    )
})
const selectedGroup = computed(
  () => groupedPresets.value.find((group) => group.championId === selectedChampionId.value) ?? null
)
const selectedAutoUseCount = computed(() => selectedGroup.value?.presets.filter((preset) => preset.autoUse).length ?? 0)

const scenarioOptions: Array<{ value: BuildScenario; label: string }> = BUILD_SCENARIOS.map((scenario) => ({
  value: scenario,
  label: scenarioLabel(scenario)
}))

watch(
  groupedPresets,
  (groups) => {
    if (!groups.some((group) => group.championId === selectedChampionId.value)) {
      selectedChampionId.value = groups[0]?.championId ?? null
    }
  },
  { immediate: true }
)

const selectChampion = (championId: number) => {
  selectedChampionId.value = championId
}

const handleAddPreset = () => {
  editingPreset.value = null
  initialChampion.value = null
  showEditor.value = true
}

const handleAddPresetForSelectedChampion = () => {
  if (!selectedGroup.value) return
  editingPreset.value = null
  initialChampion.value = {
    championId: selectedGroup.value.championId,
    championName: selectedGroup.value.championName
  }
  showEditor.value = true
}

const handleEditPreset = (preset: BuildPreset) => {
  editingPreset.value = preset
  initialChampion.value = null
  showEditor.value = true
}

const handleCloseEditor = () => {
  if (isSaving.value) return
  showEditor.value = false
  editingPreset.value = null
  initialChampion.value = null
}

const handleSavePreset = async (preset: BuildPreset) => {
  if (isSaving.value) return
  isSaving.value = true
  try {
    if (editingPreset.value) await presetStore.updatePreset(editingPreset.value.id, preset)
    else await presetStore.addPreset(preset)
    selectedChampionId.value = preset.target.championId
    toast.success(editingPreset.value ? '构建方案已更新' : '构建方案已创建')
    showEditor.value = false
    editingPreset.value = null
    initialChampion.value = null
  } catch (error) {
    toast.error(`保存失败：${error instanceof Error ? error.message : String(error)}`)
  } finally {
    isSaving.value = false
  }
}

const handleAutoUseChange = async (preset: BuildPreset, enabled: boolean) => {
  if (autoUsePresetId.value) return
  autoUsePresetId.value = preset.id
  try {
    await presetStore.setAutoUse(preset.id, enabled)
    toast.success(
      enabled ? `已为${scenarioLabel(preset.target.scenario)}启用「${preset.name}」` : '已停止自动使用此方案'
    )
  } catch (error) {
    toast.error(`设置失败：${error instanceof Error ? error.message : String(error)}`)
  } finally {
    autoUsePresetId.value = null
  }
}

const handleApplyPreset = async (preset: BuildPreset) => {
  if (applyingPresetId.value) return
  applyingPresetId.value = preset.id
  try {
    await applyPreset(preset)
    toast.success(`已应用「${preset.name}」`)
  } catch (error) {
    toast.error(`应用失败：${error instanceof Error ? error.message : String(error)}`)
  } finally {
    applyingPresetId.value = null
  }
}

const requestDeletePreset = (preset: BuildPreset) => {
  pendingDeletePreset.value = preset
  deleteDialogOpen.value = true
}

const confirmDeletePreset = async (event: Event) => {
  event.preventDefault()
  const preset = pendingDeletePreset.value
  if (!preset || isDeleting.value) return
  isDeleting.value = true
  try {
    await presetStore.deletePreset(preset.id)
    toast.success(`已删除「${preset.name}」`)
    deleteDialogOpen.value = false
    pendingDeletePreset.value = null
  } catch (error) {
    toast.error(`删除失败：${error instanceof Error ? error.message : String(error)}`)
  } finally {
    isDeleting.value = false
  }
}

const handleExportPresets = () => {
  try {
    const blob = new Blob([presetStore.exportPresets()], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const anchor = document.createElement('a')
    anchor.href = url
    anchor.download = `nidalee-build-presets-${Date.now()}.json`
    anchor.click()
    URL.revokeObjectURL(url)
  } catch (error) {
    toast.error(`导出失败：${error instanceof Error ? error.message : String(error)}`)
  }
}

const handleImportPresets = async (event: Event) => {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  input.value = ''
  if (!file) return
  try {
    const count = await presetStore.importPresets(await file.text())
    if (count) toast.success(`已导入 ${count} 个方案`)
    else toast.info('没有导入新方案', { description: '文件中的方案均已存在' })
  } catch (error) {
    toast.error(`导入失败：${error instanceof Error ? error.message : String(error)}`)
  }
}
</script>

<template>
  <BuildPresetEditor
    v-if="showEditor"
    :preset="editingPreset"
    :initial-champion="initialChampion"
    :saving="isSaving"
    @close="handleCloseEditor"
    @save="handleSavePreset"
  />

  <Card v-else class="gap-0 overflow-hidden py-0">
    <CardHeader class="flex-row flex-wrap items-center justify-between gap-3 border-b border-border/50 p-4">
      <div class="min-w-0">
        <CardTitle class="text-lg font-medium">我的方案</CardTitle>
        <CardDescription class="mt-0.5 text-xs">
          按英雄和场景保存高级覆盖方案；未开启自动使用的方案仍可手动应用。
        </CardDescription>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <Button @click="handleAddPreset">
          <Plus data-icon="inline-start" />
          新建方案
        </Button>
        <Button v-if="presetCount" variant="outline" @click="handleExportPresets">
          <Upload data-icon="inline-start" />
          导出
        </Button>
        <Button variant="outline" @click="importInput?.click()">
          <FileInput data-icon="inline-start" />
          导入
        </Button>
        <input
          ref="importInput"
          type="file"
          accept="application/json,.json"
          class="hidden"
          @change="handleImportPresets"
        />
      </div>
    </CardHeader>

    <CardContent class="flex flex-col gap-4 p-4">
      <div v-if="presetCount" class="grid gap-3 sm:grid-cols-[minmax(0,1fr)_11rem]">
        <div class="relative">
          <Search class="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
          <Input v-model="searchQuery" aria-label="搜索构建方案" placeholder="搜索英雄或方案名称" class="pl-9" />
        </div>
        <Select v-model:model-value="scenarioFilter">
          <SelectTrigger aria-label="按游戏场景筛选"><SelectValue placeholder="全部场景" /></SelectTrigger>
          <SelectContent>
            <SelectGroup>
              <SelectItem value="all">全部场景</SelectItem>
              <SelectItem v-for="scenario in scenarioOptions" :key="scenario.value" :value="scenario.value">
                {{ scenario.label }}
              </SelectItem>
            </SelectGroup>
          </SelectContent>
        </Select>
      </div>

      <div
        v-if="groupedPresets.length && selectedGroup"
        class="grid min-h-[28rem] overflow-hidden rounded-xl surface-inset lg:h-[calc(100dvh-22rem)] lg:max-h-[42rem] lg:grid-cols-[15rem_minmax(0,1fr)]"
      >
        <aside class="flex min-h-0 flex-col border-b border-border/50 lg:border-r lg:border-b-0">
          <div class="border-b border-border/40 px-3 py-2.5">
            <p class="text-sm font-medium">英雄</p>
            <p class="text-xs text-muted-foreground">
              {{ groupedPresets.length }} 个英雄 · {{ filteredPresets.length }} 个方案
            </p>
          </div>
          <div class="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto p-2" aria-label="选择英雄方案">
            <button
              v-for="group in groupedPresets"
              :key="group.championId"
              type="button"
              :aria-pressed="group.championId === selectedChampionId"
              :class="
                cn(
                  'flex w-full items-center gap-2.5 rounded-xl px-2.5 py-2 text-left transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50',
                  group.championId === selectedChampionId
                    ? 'bg-primary/12 text-foreground'
                    : 'text-muted-foreground hover:bg-accent/45 hover:text-foreground'
                )
              "
              @click="selectChampion(group.championId)"
            >
              <img :src="getChampionIconUrl(group.championId)" alt="" class="size-9 rounded-xl ring-1 ring-border/60" />
              <span class="min-w-0 flex-1">
                <span class="block truncate text-sm font-medium">{{ group.championName }}</span>
                <span class="block text-xs text-muted-foreground">
                  {{ group.presets.length }} 个方案<span v-if="group.presets.some((preset) => preset.autoUse)">
                    · 已启用</span
                  >
                </span>
              </span>
            </button>
          </div>
        </aside>

        <section class="flex min-h-0 min-w-0 flex-col">
          <header class="flex flex-wrap items-center justify-between gap-3 border-b border-border/40 px-4 py-3">
            <div class="flex min-w-0 items-center gap-3">
              <img
                :src="getChampionIconUrl(selectedGroup.championId)"
                alt=""
                class="size-10 rounded-xl ring-1 ring-border/60"
              />
              <div class="min-w-0">
                <h3 class="truncate text-base font-medium">{{ selectedGroup.championName }}</h3>
                <p class="text-xs text-muted-foreground">
                  {{ selectedGroup.presets.length }} 个方案<span v-if="selectedAutoUseCount">
                    · {{ selectedAutoUseCount }} 个自动使用</span
                  >
                </p>
              </div>
            </div>
            <Button size="sm" variant="outline" @click="handleAddPresetForSelectedChampion">
              <Plus data-icon="inline-start" />
              为该英雄新建
            </Button>
          </header>

          <div class="grid min-h-0 flex-1 content-start gap-2 overflow-y-auto p-3 md:grid-cols-2">
            <BuildPresetCard
              v-for="preset in selectedGroup.presets"
              :key="preset.id"
              :preset="preset"
              :applying="applyingPresetId === preset.id"
              :actions-disabled="applyingPresetId !== null"
              :auto-use-disabled="autoUsePresetId !== null"
              @apply="handleApplyPreset"
              @edit="handleEditPreset"
              @remove="requestDeletePreset"
              @set-auto-use="handleAutoUseChange"
            />
          </div>
        </section>
      </div>

      <div v-else class="flex min-h-56 flex-col items-center justify-center gap-3 text-center">
        <Sparkles class="size-8 text-muted-foreground" />
        <div class="min-w-0">
          <p class="text-sm font-medium">{{ presetCount ? '没有符合条件的方案' : '还没有个人方案' }}</p>
          <p class="mt-1 text-xs text-muted-foreground">
            <template v-if="presetCount">尝试清除搜索或切换场景。</template>
            <template v-else-if="autoBuildEnabled">自动构建已开启；没有个人方案时会使用在线推荐。</template>
            <template v-else>自动构建尚未开启；你仍可新建、导入或手动应用方案。</template>
          </p>
        </div>
        <div v-if="!presetCount" class="flex flex-wrap items-center justify-center gap-2">
          <Button variant="outline" @click="handleAddPreset">为常用英雄新建方案</Button>
          <Button variant="ghost" @click="importInput?.click()">导入已有方案</Button>
        </div>
      </div>
    </CardContent>
  </Card>

  <AlertDialog v-model:open="deleteDialogOpen">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>删除构建方案？</AlertDialogTitle>
        <AlertDialogDescription>「{{ pendingDeletePreset?.name }}」将被永久移除。</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel :disabled="isDeleting">取消</AlertDialogCancel>
        <AlertDialogAction :disabled="isDeleting" @click="confirmDeletePreset">
          <Spinner v-if="isDeleting" data-icon="inline-start" />
          删除
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
</template>
