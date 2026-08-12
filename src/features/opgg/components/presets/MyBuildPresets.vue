<template>
  <BuildPresetEditor
    v-if="showEditor"
    :preset="editingPreset"
    :saving="isSaving"
    @close="handleCloseEditor"
    @save="handleSavePreset"
  />

  <div v-else class="space-y-4">
    <!-- 操作按钮 -->
    <div class="flex items-center gap-2">
      <Button @click="handleAddPreset" class="flex items-center gap-2 bg-primary hover:bg-primary/90 shadow-md">
        <Plus class="h-4 w-4" />
        新建方案
      </Button>
      <Button
        v-if="presetCount > 0"
        @click="handleExportPresets"
        variant="outline"
        class="flex items-center gap-2 hover:bg-primary/10 hover:text-primary hover:border-primary/50"
      >
        <Upload class="h-4 w-4" />
        导出方案
      </Button>
      <Button v-if="presetCount > 0" variant="outline" class="flex items-center gap-2" @click="importInput?.click()">
        <FileInput class="h-4 w-4" />
        导入方案
      </Button>
      <input
        ref="importInput"
        type="file"
        accept="application/json,.json"
        class="hidden"
        @change="handleImportPresets"
      />
    </div>

    <!-- 搜索和筛选 -->
    <div v-if="presetCount > 0" class="flex items-center gap-2">
      <div class="relative flex-1">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
        <Input v-model="searchQuery" placeholder="搜索英雄名称..." class="pl-9" />
      </div>
      <Select v-model:model-value="positionFilter">
        <SelectTrigger class="w-[140px]">
          <SelectValue placeholder="筛选位置" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="all">全部位置</SelectItem>
          <SelectItem value="TOP">上路</SelectItem>
          <SelectItem value="JUNGLE">打野</SelectItem>
          <SelectItem value="MID">中路</SelectItem>
          <SelectItem value="ADC">下路</SelectItem>
          <SelectItem value="SUPPORT">辅助</SelectItem>
        </SelectContent>
      </Select>
    </div>

    <!-- 方案列表 -->
    <div v-if="filteredPresets.length > 0" class="space-y-3">
      <div
        v-for="preset in filteredPresets"
        :key="preset.id"
        class="group p-5 rounded-xl border border-border bg-card/50 backdrop-blur-sm hover:bg-card hover:border-primary/50 hover:shadow-md transition-all duration-200"
      >
        <div class="flex items-start justify-between gap-4">
          <!-- 配置信息 -->
          <div class="flex-1 space-y-2">
            <div class="flex items-center gap-2">
              <h3 class="font-semibold text-foreground">{{ preset.name }}</h3>
              <Badge v-if="preset.isDefault" variant="default" class="text-xs"> 默认 </Badge>
              <Badge variant="outline" class="text-xs">
                {{ getSourceLabel(preset.source.kind) }}
              </Badge>
            </div>

            <div class="flex items-center gap-4 text-xs text-muted-foreground">
              <span class="flex items-center gap-1">
                <User class="h-3 w-3" />
                {{ preset.applicability.championName || '通用' }}
              </span>
              <span class="flex items-center gap-1">
                <MapPin class="h-3 w-3" />
                {{ getPositionLabel(preset.applicability.position) }}
              </span>
              <span class="flex items-center gap-1">
                <Sparkles class="h-3 w-3" />
                主系: {{ getStyleName(preset.components.runes.primaryStyleId) }}
              </span>
              <span class="flex items-center gap-1">
                <TrendingUp class="h-3 w-3" />
                使用 {{ preset.usageCount }} 次
              </span>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="flex items-center gap-1">
            <Button
              size="sm"
              class="h-8 gap-1.5 px-2.5 text-xs"
              :disabled="applyingPresetId !== null"
              @click="handleApplyPreset(preset)"
            >
              <Spinner v-if="applyingPresetId === preset.id" data-icon="inline-start" />
              <Wand2 v-else class="h-3.5 w-3.5" />
              应用
            </Button>
            <Button
              v-if="!preset.isDefault"
              @click="handleSetDefault(preset.id)"
              variant="ghost"
              size="sm"
              class="h-8 w-8 p-0 hover:bg-primary/10 hover:text-primary opacity-0 group-hover:opacity-100 transition-opacity"
              title="设为默认"
            >
              <Star class="h-4 w-4" />
            </Button>
            <Button
              @click="handleEditPreset(preset)"
              variant="ghost"
              size="sm"
              class="h-8 w-8 p-0 hover:bg-primary/10 hover:text-primary opacity-0 group-hover:opacity-100 transition-opacity"
              title="编辑"
            >
              <Pencil class="h-4 w-4" />
            </Button>
            <Button
              @click="requestDeletePreset(preset)"
              variant="ghost"
              size="sm"
              class="h-8 w-8 p-0 hover:bg-destructive/10 hover:text-destructive opacity-0 group-hover:opacity-100 transition-opacity"
              title="删除"
            >
              <Trash2 class="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </div>

    <!-- 空状态 -->
    <div v-else-if="presetCount === 0" class="py-16 text-center space-y-4">
      <div class="inline-flex h-20 w-20 items-center justify-center rounded-full bg-primary/10 shadow-sm">
        <Sparkles class="h-10 w-10 text-primary" />
      </div>
      <div>
        <h3 class="text-lg font-semibold text-foreground mb-2">还没有保存的方案</h3>
        <p class="text-sm text-muted-foreground">新建方案，或从推荐方案中保存一份快照</p>
      </div>
    </div>

    <!-- 无搜索结果 -->
    <div v-else class="py-16 text-center space-y-3">
      <div class="inline-flex h-16 w-16 items-center justify-center rounded-full bg-muted">
        <Search class="h-8 w-8 text-muted-foreground" />
      </div>
      <div>
        <p class="text-sm text-foreground">没有找到匹配的配置</p>
        <p class="text-xs text-muted-foreground mt-1">尝试调整搜索条件或筛选器</p>
      </div>
    </div>
  </div>

  <AlertDialog v-model:open="deleteDialogOpen">
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>删除构建方案？</AlertDialogTitle>
        <AlertDialogDescription>
          「{{ pendingDeletePreset?.name }}」将从我的方案中移除，此操作无法撤销。
        </AlertDialogDescription>
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

<script setup lang="ts">
import { useBuildPresetStore } from '@/shared/stores/features/buildPresetStore'
import type { BuildPosition, BuildPreset, BuildPresetSourceKind } from '@/shared/models/buildPreset'
import { toast } from 'vue-sonner'
import {
  FileInput,
  MapPin,
  Pencil,
  Plus,
  Search,
  Sparkles,
  Star,
  Trash2,
  TrendingUp,
  Upload,
  User,
  Wand2
} from 'lucide-vue-next'
import BuildPresetEditor from './BuildPresetEditor.vue'
import { useBuildApplication } from '@/shared/composables/game/useBuildApplication'

const presetStore = useBuildPresetStore()
const { applyPreset } = useBuildApplication()

// 状态
const searchQuery = ref('')
const positionFilter = ref('all')
const showEditor = ref(false)
const editingPreset = ref<BuildPreset | null>(null)
const isSaving = ref(false)
const isDeleting = ref(false)
const applyingPresetId = ref<string | null>(null)
const importInput = ref<HTMLInputElement | null>(null)
const deleteDialogOpen = ref(false)
const pendingDeletePreset = ref<BuildPreset | null>(null)

// 计算属性
const presetCount = computed(() => presetStore.presetCount)
const presets = computed(() => presetStore.presets)

const filteredPresets = computed(() => {
  let result = presets.value

  // 搜索过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(
      (preset) =>
        preset.name.toLowerCase().includes(query) || preset.applicability.championName?.toLowerCase().includes(query)
    )
  }

  // 位置过滤
  if (positionFilter.value !== 'all') {
    result = result.filter((preset) => preset.applicability.position === positionFilter.value)
  }

  return result
})

// 辅助函数
const getSourceLabel = (source: BuildPresetSourceKind) => {
  const labels: Record<BuildPresetSourceKind, string> = {
    opgg: 'OP.GG',
    custom: '自定义',
    import: '导入',
    client: '客户端'
  }
  return labels[source] || source
}

const getPositionLabel = (position: BuildPosition | null) => {
  if (!position) return '通用'
  const labels: Record<string, string> = {
    TOP: '上路',
    JUNGLE: '打野',
    MID: '中路',
    ADC: '下路',
    SUPPORT: '辅助'
  }
  return labels[position] || position
}

const getStyleName = (styleId: number) => {
  const names: Record<number, string> = {
    8000: '精密',
    8100: '主宰',
    8200: '巫术',
    8300: '启迪',
    8400: '坚决'
  }
  return names[styleId] || `未知(${styleId})`
}

// 事件处理
const handleAddPreset = () => {
  editingPreset.value = null
  showEditor.value = true
}

const handleEditPreset = (preset: BuildPreset) => {
  editingPreset.value = preset
  showEditor.value = true
}

const handleCloseEditor = () => {
  if (isSaving.value) return
  showEditor.value = false
  editingPreset.value = null
}

const handleSavePreset = async (preset: BuildPreset) => {
  if (isSaving.value) return
  isSaving.value = true
  try {
    if (editingPreset.value) {
      await presetStore.updatePreset(editingPreset.value.id, preset)
    } else {
      await presetStore.addPreset(preset)
    }
    toast.success(editingPreset.value ? '构建方案已更新' : '构建方案已创建')
    showEditor.value = false
    editingPreset.value = null
  } catch (error) {
    toast.error(`保存失败：${error instanceof Error ? error.message : String(error)}`)
  } finally {
    isSaving.value = false
  }
}

const handleSetDefault = async (id: string) => {
  try {
    await presetStore.setDefault(id)
    toast.success('默认方案已更新')
  } catch (error) {
    toast.error(`设置失败：${error instanceof Error ? error.message : String(error)}`)
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
    const json = presetStore.exportPresets()
    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `nidalee-build-presets-${Date.now()}.json`
    a.click()
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
    toast.success(`已导入 ${count} 个方案`)
  } catch (error) {
    toast.error(`导入失败：${error instanceof Error ? error.message : String(error)}`)
  }
}
</script>
