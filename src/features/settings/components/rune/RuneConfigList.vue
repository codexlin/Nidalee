<template>
  <RuneEditorPanel
    v-if="showEditor"
    :config="editingConfig"
    :saving="isSaving"
    @close="handleCloseEditor"
    @save="handleSaveConfig"
  />

  <div v-else class="space-y-4">
    <!-- 操作按钮 -->
    <div class="flex items-center gap-2">
      <Button @click="handleAddConfig" class="flex items-center gap-2 bg-primary hover:bg-primary/90 shadow-md">
        <Plus class="h-4 w-4" />
        新增配置
      </Button>
      <Button
        v-if="configCount > 0"
        @click="handleExportConfigs"
        variant="outline"
        class="flex items-center gap-2 hover:bg-primary/10 hover:text-primary hover:border-primary/50"
      >
        <Upload class="h-4 w-4" />
        导出配置
      </Button>
    </div>

    <!-- 搜索和筛选 -->
    <div v-if="configCount > 0" class="flex items-center gap-2">
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

    <!-- 配置列表 -->
    <div v-if="filteredConfigs.length > 0" class="space-y-3">
      <div
        v-for="config in filteredConfigs"
        :key="config.id"
        class="group p-5 rounded-xl border border-border bg-card/50 backdrop-blur-sm hover:bg-card hover:border-primary/50 hover:shadow-md transition-all duration-200"
      >
        <div class="flex items-start justify-between gap-4">
          <!-- 配置信息 -->
          <div class="flex-1 space-y-2">
            <div class="flex items-center gap-2">
              <h3 class="font-semibold text-foreground">{{ config.name }}</h3>
              <Badge v-if="config.isDefault" variant="default" class="text-xs"> 默认 </Badge>
              <Badge variant="outline" class="text-xs">
                {{ getSourceLabel(config.source) }}
              </Badge>
            </div>

            <div class="flex items-center gap-4 text-xs text-muted-foreground">
              <span class="flex items-center gap-1">
                <User class="h-3 w-3" />
                {{ config.championName || '通用' }}
              </span>
              <span class="flex items-center gap-1">
                <MapPin class="h-3 w-3" />
                {{ getPositionLabel(config.position) }}
              </span>
              <span class="flex items-center gap-1">
                <Sparkles class="h-3 w-3" />
                主系: {{ getStyleName(config.primaryStyleId) }}
              </span>
              <span class="flex items-center gap-1">
                <TrendingUp class="h-3 w-3" />
                使用 {{ config.usageCount }} 次
              </span>
            </div>
          </div>

          <!-- 操作按钮 -->
          <div class="flex items-center gap-1">
            <Button
              v-if="!config.isDefault"
              @click="handleSetDefault(config.id)"
              variant="ghost"
              size="sm"
              class="h-8 w-8 p-0 hover:bg-primary/10 hover:text-primary opacity-0 group-hover:opacity-100 transition-opacity"
              title="设为默认"
            >
              <Star class="h-4 w-4" />
            </Button>
            <Button
              @click="handleEditConfig(config)"
              variant="ghost"
              size="sm"
              class="h-8 w-8 p-0 hover:bg-primary/10 hover:text-primary opacity-0 group-hover:opacity-100 transition-opacity"
              title="编辑"
            >
              <Pencil class="h-4 w-4" />
            </Button>
            <Button
              @click="handleDeleteConfig(config.id)"
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
    <div v-else-if="configCount === 0" class="py-16 text-center space-y-4">
      <div class="inline-flex h-20 w-20 items-center justify-center rounded-full bg-primary/10 shadow-sm">
        <Sparkles class="h-10 w-10 text-primary" />
      </div>
      <div>
        <h3 class="text-lg font-semibold text-foreground mb-2">还没有符文配置</h3>
        <p class="text-sm text-muted-foreground">点击"新增配置"创建您的第一个符文配置</p>
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
</template>

<script setup lang="ts">
import { useUserRuneStore, type RuneConfig } from '@/shared/stores/features/userRuneStore'
import { toast } from 'vue-sonner'
import { Plus, Upload, Search, User, MapPin, Sparkles, TrendingUp, Star, Pencil, Trash2 } from 'lucide-vue-next'
import RuneEditorPanel from './RuneEditorPanel.vue'

const userRuneStore = useUserRuneStore()

// 状态
const searchQuery = ref('')
const positionFilter = ref('all')
const showEditor = ref(false)
const editingConfig = ref<RuneConfig | null>(null)
const isSaving = ref(false)

// 计算属性
const configCount = computed(() => userRuneStore.configCount)
const configs = computed(() => userRuneStore.configs)

const filteredConfigs = computed(() => {
  let result = configs.value

  // 搜索过滤
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter((c) => c.name.toLowerCase().includes(query) || c.championName?.toLowerCase().includes(query))
  }

  // 位置过滤
  if (positionFilter.value !== 'all') {
    result = result.filter((c) => c.position === positionFilter.value)
  }

  return result
})

// 辅助函数
const getSourceLabel = (source: string) => {
  const labels: Record<string, string> = {
    opgg: 'OP.GG',
    custom: '自定义',
    import: '导入'
  }
  return labels[source] || source
}

const getPositionLabel = (position: string | null) => {
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
const handleAddConfig = () => {
  editingConfig.value = null
  showEditor.value = true
}

const handleEditConfig = (config: RuneConfig) => {
  editingConfig.value = config
  showEditor.value = true
}

const handleCloseEditor = () => {
  if (isSaving.value) return
  showEditor.value = false
  editingConfig.value = null
}

const handleSaveConfig = async (config: RuneConfig) => {
  if (isSaving.value) return
  isSaving.value = true
  try {
    if (editingConfig.value) {
      await userRuneStore.updateConfig(editingConfig.value.id, config)
    } else {
      await userRuneStore.addConfig(config)
    }
    toast.success(editingConfig.value ? '符文配置已更新' : '符文配置已创建')
    showEditor.value = false
    editingConfig.value = null
  } catch (error) {
    toast.error(`保存失败：${error instanceof Error ? error.message : String(error)}`)
  } finally {
    isSaving.value = false
  }
}

const handleSetDefault = async (id: string) => {
  try {
    await userRuneStore.setAsDefault(id)
  } catch (error) {
    console.error('设置默认配置失败:', error)
  }
}

const handleDeleteConfig = async (id: string) => {
  // TODO: 添加确认对话框
  if (confirm('确定要删除这个符文配置吗？')) {
    try {
      await userRuneStore.deleteConfig(id)
    } catch (error) {
      console.error('删除配置失败:', error)
    }
  }
}

const handleExportConfigs = () => {
  try {
    const json = userRuneStore.exportConfigs()
    const blob = new Blob([json], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = `nidalee-rune-configs-${Date.now()}.json`
    a.click()
    URL.revokeObjectURL(url)
  } catch (error) {
    console.error('导出配置失败:', error)
    toast.error('导出配置失败')
  }
}
</script>
