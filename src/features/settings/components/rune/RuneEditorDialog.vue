<template>
  <Dialog :open="open" @update:open="handleClose">
    <DialogContent
      class="max-w-6xl max-h-[90vh] overflow-y-auto bg-gradient-to-br from-background via-muted/30 to-background/80"
    >
      <DialogHeader>
        <DialogTitle class="flex items-center gap-2 text-foreground">
          <Sparkles class="h-5 w-5 text-primary" />
          {{ isEditing ? '编辑符文配置' : '新增符文配置' }}
        </DialogTitle>
        <DialogDescription class="text-muted-foreground">
          {{ isEditing ? '修改现有的符文配置' : '创建一个新的符文配置' }}
        </DialogDescription>
      </DialogHeader>

      <div class="space-y-6 py-4">
        <!-- 基础信息 -->
        <div class="space-y-4 p-6 rounded-xl bg-card/50 backdrop-blur-sm border border-border/50">
          <h3 class="text-sm font-semibold text-primary flex items-center gap-2">
            <div class="h-1 w-1 rounded-full bg-primary"></div>
            基础信息
          </h3>

          <!-- 配置名称 -->
          <div class="space-y-2">
            <Label for="config-name">配置名称</Label>
            <Input id="config-name" v-model="formData.name" placeholder="例如: 劫-中路-电刑" />
          </div>

          <!-- 作用域选择 -->
          <div class="space-y-2">
            <Label>作用域</Label>
            <RadioGroup v-model="formData.scope">
              <div class="flex items-center space-x-2">
                <RadioGroupItem value="champion-position" id="scope-1" />
                <Label for="scope-1" class="font-normal cursor-pointer"> 英雄+位置专属（推荐） </Label>
              </div>
              <div class="flex items-center space-x-2">
                <RadioGroupItem value="champion-all" id="scope-2" />
                <Label for="scope-2" class="font-normal cursor-pointer"> 英雄通用（适用于该英雄所有位置） </Label>
              </div>
              <div class="flex items-center space-x-2">
                <RadioGroupItem value="position-all" id="scope-3" />
                <Label for="scope-3" class="font-normal cursor-pointer"> 位置通用（适用于该位置所有英雄） </Label>
              </div>
            </RadioGroup>
          </div>

          <!-- 英雄选择 -->
          <div v-if="formData.scope !== 'position-all'" class="space-y-2">
            <Label for="champion" class="text-foreground">英雄</Label>
            <Input
              id="champion"
              v-model="championSearch"
              placeholder="输入英雄名称搜索..."
              @input="handleChampionSearch"
              class="bg-background/50"
            />
            <div
              v-if="championSearchResults.length > 0"
              class="mt-2 max-h-48 overflow-y-auto border border-border rounded-lg bg-card shadow-sm"
            >
              <div
                v-for="champ in championSearchResults"
                :key="champ.id"
                @click="selectChampion(champ)"
                class="px-3 py-2 hover:bg-primary/10 hover:text-primary cursor-pointer transition-colors border-b border-border/50 last:border-0"
              >
                <span class="font-medium">{{ champ.name }}</span>
                <span class="text-xs text-muted-foreground ml-2">({{ champ.alias }})</span>
              </div>
            </div>
            <div
              v-if="formData.championId"
              class="flex items-center gap-2 p-2 rounded-md bg-primary/10 border border-primary/20"
            >
              <CheckCircle2 class="h-4 w-4 text-primary" />
              <span class="text-sm font-medium text-primary">已选择: {{ formData.championName }}</span>
            </div>
          </div>

          <!-- 位置选择 -->
          <div v-if="formData.scope !== 'champion-all'" class="space-y-2">
            <Label for="position">位置</Label>
            <Select v-model:model-value="formData.position">
              <SelectTrigger>
                <SelectValue placeholder="选择位置" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="TOP">上路</SelectItem>
                <SelectItem value="JUNGLE">打野</SelectItem>
                <SelectItem value="MID">中路</SelectItem>
                <SelectItem value="ADC">下路</SelectItem>
                <SelectItem value="SUPPORT">辅助</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <!-- 设为默认 -->
          <div class="flex items-center space-x-2">
            <Checkbox id="is-default" v-model:checked="formData.isDefault" />
            <Label for="is-default" class="font-normal cursor-pointer"> 设为此英雄+位置的默认配置 </Label>
          </div>
        </div>

        <!-- 符文选择（可视化） -->
        <div class="space-y-4 p-6 rounded-xl bg-card/50 backdrop-blur-sm border border-border/50">
          <div class="flex items-center justify-between">
            <h3 class="text-sm font-semibold text-primary flex items-center gap-2">
              <div class="h-1 w-1 rounded-full bg-primary"></div>
              符文配置
            </h3>
            <div class="flex items-center gap-2">
              <Button
                @click="handleImportFromOpgg"
                variant="outline"
                size="sm"
                :disabled="isImporting"
                class="hover:bg-primary/10 hover:text-primary hover:border-primary/50"
              >
                <Download class="h-3 w-3 mr-1" />
                从 OP.GG 导入
              </Button>
              <Button
                @click="handleLoadFromClient"
                variant="outline"
                size="sm"
                :disabled="isImporting"
                class="hover:bg-primary/10 hover:text-primary hover:border-primary/50"
              >
                <FileInput class="h-3 w-3 mr-1" />
                从客户端导入
              </Button>
            </div>
          </div>

          <Separator class="bg-border/50" />

          <!-- 可视化符文选择器 -->
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

      <DialogFooter class="gap-2">
        <Button variant="outline" @click="handleClose" class="hover:bg-muted">取消</Button>
        <Button @click="handleSave" :disabled="!isFormValid" class="bg-primary hover:bg-primary/90">
          {{ isEditing ? '保存修改' : '创建配置' }}
        </Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import type { RuneConfig } from '@/shared/stores/features/userRuneStore'
import { Sparkles, Download, FileInput, CheckCircle2 } from 'lucide-vue-next'
import RunePerkPicker from './RunePerkPicker.vue'

interface Props {
  open: boolean
  config?: RuneConfig | null
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
  save: [config: RuneConfig]
}>()

// 使用缓存的英雄数据
const { data: allChampionsData } = useChampions()

// 状态
const isEditing = computed(() => !!props.config)
const championSearch = ref('')
const championSearchResults = ref<ChampionInfo[]>([])
const isImporting = ref(false)

// 表单数据
const formData = reactive({
  name: '',
  championId: null as number | null,
  championName: null as string | null,
  position: null as string | null,
  scope: 'champion-position' as 'champion-position' | 'champion-all' | 'position-all',
  primaryStyleId: 8000,
  subStyleId: 8200,
  selectedPerkIds: [] as number[],
  isDefault: false,
  source: 'custom' as 'opgg' | 'custom' | 'import'
})

// 初始化表单数据
watch(
  () => props.config,
  (config) => {
    if (config) {
      formData.name = config.name
      formData.championId = config.championId
      formData.championName = config.championName
      formData.position = config.position
      formData.scope = config.scope
      formData.primaryStyleId = config.primaryStyleId
      formData.subStyleId = config.subStyleId
      formData.selectedPerkIds = [...config.selectedPerkIds]
      formData.isDefault = config.isDefault
      formData.source = config.source
    } else {
      // 重置表单
      formData.name = ''
      formData.championId = null
      formData.championName = null
      formData.position = null
      formData.scope = 'champion-position'
      formData.primaryStyleId = 8000
      formData.subStyleId = 8200
      formData.selectedPerkIds = []
      formData.isDefault = false
      formData.source = 'custom'
    }
  },
  { immediate: true }
)

// 表单验证
const isFormValid = computed(() => {
  if (!formData.name) return false

  // 根据作用域验证
  if (formData.scope === 'champion-position') {
    if (!formData.championId || !formData.position) return false
  } else if (formData.scope === 'champion-all') {
    if (!formData.championId) return false
  } else if (formData.scope === 'position-all') {
    if (!formData.position) return false
  }

  // 验证符文 ID（必须选择9个）
  if (formData.selectedPerkIds.length !== 9) return false

  return true
})

// 英雄搜索
const handleChampionSearch = () => {
  if (!championSearch.value || championSearch.value.length < 2) {
    championSearchResults.value = []
    return
  }

  // 使用缓存的英雄数据（无需等待）
  const allChampions = allChampionsData.value || []

  // 根据输入过滤英雄
  const query = championSearch.value.toLowerCase()
  championSearchResults.value = allChampions
    .filter((champ) => champ.name.toLowerCase().includes(query) || champ.alias.toLowerCase().includes(query))
    .slice(0, 10) // 最多显示10个结果
}

const selectChampion = (champion: ChampionInfo) => {
  formData.championId = champion.id
  formData.championName = champion.name
  championSearch.value = champion.name
  championSearchResults.value = []
}

// 从 OP.GG 导入
const handleImportFromOpgg = async () => {
  if (!formData.championId || !formData.position) {
    alert('请先选择英雄和位置')
    return
  }

  isImporting.value = true
  try {
    // 调用后端获取 OP.GG 推荐
    const build = await invoke<OpggChampionBuild>('get_opgg_champion_build', {
      region: 'kr',
      mode: 'ranked',
      championId: formData.championId,
      position: formData.position,
      tier: 'DIAMOND+'
    })

    if (build.perks && build.perks.length > 0) {
      const bestPerk = build.perks[0]
      formData.primaryStyleId = bestPerk.primaryId
      formData.subStyleId = bestPerk.secondaryId
      formData.selectedPerkIds = bestPerk.perks
      formData.source = 'opgg'

      alert('从 OP.GG 导入成功！')
    }
  } catch (error) {
    console.error('从 OP.GG 导入失败:', error)
    alert('导入失败: ' + (error instanceof Error ? error.message : '未知错误'))
  } finally {
    isImporting.value = false
  }
}

// 从游戏客户端导入
const handleLoadFromClient = async () => {
  isImporting.value = true
  try {
    const currentPage = await invoke<RunePage>('get_current_rune_page')
    if (currentPage) {
      formData.primaryStyleId = currentPage.primaryStyleId
      formData.subStyleId = currentPage.subStyleId
      formData.selectedPerkIds = currentPage.selectedPerkIds
      formData.source = 'import'

      alert('从游戏客户端导入成功！')
    } else {
      alert('未找到当前符文页')
    }
  } catch (error) {
    console.error('从游戏客户端导入失败:', error)
    alert('导入失败: ' + (error instanceof Error ? error.message : '未知错误'))
  } finally {
    isImporting.value = false
  }
}

// 保存配置
const handleSave = () => {
  // 根据作用域设置 championId 和 position
  if (formData.scope === 'position-all') {
    formData.championId = null
    formData.championName = null
  } else if (formData.scope === 'champion-all') {
    formData.position = null
  }

  const config: RuneConfig = {
    id: props.config?.id || crypto.randomUUID(),
    name: formData.name,
    championId: formData.championId,
    championName: formData.championName,
    position: formData.position,
    scope: formData.scope,
    primaryStyleId: formData.primaryStyleId,
    subStyleId: formData.subStyleId,
    selectedPerkIds: formData.selectedPerkIds,
    isDefault: formData.isDefault,
    source: formData.source,
    createdAt: props.config?.createdAt || Date.now(),
    updatedAt: Date.now(),
    usageCount: props.config?.usageCount || 0
  }

  emit('save', config)
}

const handleClose = () => {
  emit('close')
}
</script>
