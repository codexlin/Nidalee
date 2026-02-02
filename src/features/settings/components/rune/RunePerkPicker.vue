<template>
  <div class="space-y-6">
    <!-- 加载状态 -->
    <div v-if="isLoading" class="flex items-center justify-center py-8">
      <div class="text-muted-foreground">加载符文数据中...</div>
    </div>

    <!-- 错误状态 -->
    <Alert v-else-if="error" variant="destructive">
      <AlertCircle class="h-4 w-4" />
      <AlertTitle>加载失败</AlertTitle>
      <AlertDescription>{{ error }}</AlertDescription>
    </Alert>

    <!-- 符文选择器 -->
    <div v-else class="space-y-6">
      <!-- 主系符文 -->
      <div class="space-y-4">
        <div class="flex items-center gap-2">
          <div
            class="h-8 w-8 rounded-full flex items-center justify-center"
            :style="{ backgroundColor: getPrimaryStyleColor(primaryStyleId) }"
          >
            <Sparkles class="h-4 w-4 text-white" />
          </div>
          <h3 class="text-sm font-semibold text-foreground">主系符文: {{ primaryStyle?.name || '未选择' }}</h3>
        </div>

        <!-- 主系选择 -->
        <div class="grid grid-cols-5 gap-3">
          <div
            v-for="style in perkStyles"
            :key="style.id"
            @click="selectPrimaryStyle(style.id)"
            class="relative cursor-pointer rounded-lg border-2 p-4 transition-all hover:scale-105"
            :class="[
              primaryStyleId === style.id
                ? 'border-primary bg-primary/10 shadow-md'
                : 'border-border hover:border-primary/50'
            ]"
          >
            <div class="flex flex-col items-center gap-2">
              <img
                :src="runeData.getStyleIconUrl(style.iconPath)"
                :alt="style.name"
                class="h-10 w-10"
                @error="handleImageError"
              />
              <span class="text-xs font-medium text-center">{{ style.name }}</span>
            </div>
            <div
              v-if="primaryStyleId === style.id"
              class="absolute top-2 right-2 h-5 w-5 rounded-full bg-primary flex items-center justify-center shadow-sm"
            >
              <Check class="h-3 w-3 text-primary-foreground" />
            </div>
          </div>
        </div>

        <!-- 主系符文槽位（树状图展示） -->
        <div v-if="primaryStyle" class="space-y-4">
          <div v-for="(slot, slotIndex) in primaryStyleSlots" :key="slotIndex" class="space-y-2">
            <!-- 层级标题 -->
            <div class="flex items-center gap-2">
              <div class="h-px flex-1 bg-border"></div>
              <div
                class="text-xs font-medium px-2"
                :class="slot.type === 'kKeyStone' ? 'text-primary' : 'text-muted-foreground'"
              >
                {{ slot.slotLabel || getSlotLabel(slot.type, slotIndex) }}
                <span v-if="slot.type === 'kKeyStone'" class="ml-1 text-primary/70">(3选1)</span>
              </div>
              <div class="h-px flex-1 bg-border"></div>
            </div>

            <!-- 符文选项 -->
            <div class="grid grid-cols-3 gap-3">
              <div
                v-for="perkId in slot.perks"
                :key="perkId"
                @click="selectPerk(slotIndex, perkId)"
                class="relative cursor-pointer rounded-lg border-2 p-3 transition-all hover:scale-105"
                :class="[
                  selectedPrimaryPerks[slotIndex] === perkId
                    ? 'border-primary bg-primary/10 shadow-md'
                    : 'border-border hover:border-primary/50',
                  slot.type === 'kKeyStone' && selectedPrimaryPerks[slotIndex] === perkId
                    ? 'ring-2 ring-primary/30'
                    : ''
                ]"
              >
                <div class="flex flex-col items-center gap-2">
                  <img
                    :src="getPerkIconUrl(perkId)"
                    :alt="getPerkName(perkId)"
                    :class="slot.type === 'kKeyStone' ? 'h-16 w-16' : 'h-12 w-12'"
                    class="rounded"
                    @error="handleImageError"
                  />
                  <span
                    class="text-xs text-center line-clamp-2 leading-tight"
                    :class="slot.type === 'kKeyStone' ? 'font-semibold' : ''"
                  >
                    {{ getPerkName(perkId) }}
                  </span>
                </div>
                <div
                  v-if="selectedPrimaryPerks[slotIndex] === perkId"
                  class="absolute top-2 right-2 h-5 w-5 rounded-full bg-primary flex items-center justify-center shadow-sm"
                >
                  <Check class="h-3 w-3 text-primary-foreground" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <Separator />

      <!-- 副系符文 -->
      <div class="space-y-4">
        <div class="flex items-center gap-2">
          <div
            class="h-8 w-8 rounded-full flex items-center justify-center"
            :style="{ backgroundColor: getSubStyleColor(subStyleId) }"
          >
            <Sparkles class="h-4 w-4 text-white" />
          </div>
          <h3 class="text-sm font-semibold text-foreground">副系符文: {{ subStyle?.name || '未选择' }}</h3>
          <span class="text-xs text-muted-foreground">(选择 2 个)</span>
        </div>

        <!-- 副系选择 -->
        <div class="grid grid-cols-4 gap-3">
          <div
            v-for="style in availableSubStyles"
            :key="style.id"
            @click="selectSubStyle(style.id)"
            class="relative cursor-pointer rounded-lg border-2 p-4 transition-all hover:scale-105"
            :class="[
              subStyleId === style.id
                ? 'border-primary bg-primary/10 shadow-md'
                : 'border-border hover:border-primary/50'
            ]"
          >
            <div class="flex flex-col items-center gap-2">
              <img
                :src="runeData.getStyleIconUrl(style.iconPath)"
                :alt="style.name"
                class="h-10 w-10"
                @error="handleImageError"
              />
              <span class="text-xs font-medium text-center">{{ style.name }}</span>
            </div>
            <div
              v-if="subStyleId === style.id"
              class="absolute top-2 right-2 h-5 w-5 rounded-full bg-primary flex items-center justify-center shadow-sm"
            >
              <Check class="h-3 w-3 text-primary-foreground" />
            </div>
          </div>
        </div>

        <!-- 副系符文选择（分层展示） -->
        <div v-if="subStyle" class="space-y-4">
          <div class="flex items-center justify-between">
            <div class="text-xs font-medium text-muted-foreground">可选符文（从下面任选 2 个）</div>
            <div class="text-xs text-primary font-medium">已选: {{ selectedSubPerks.length }} / 2</div>
          </div>

          <!-- 分层展示副系符文 -->
          <div class="space-y-3">
            <div v-for="(slot, slotIndex) in subStyleSlots" :key="slotIndex" class="space-y-2">
              <div class="flex items-center gap-2">
                <div class="h-px flex-1 bg-border"></div>
                <div class="text-xs font-medium text-muted-foreground px-2">第 {{ slotIndex + 2 }} 层</div>
                <div class="h-px flex-1 bg-border"></div>
              </div>
              <div class="grid grid-cols-3 gap-3">
                <div
                  v-for="perkId in slot.perks"
                  :key="perkId"
                  @click="toggleSubPerk(perkId)"
                  class="relative cursor-pointer rounded-lg border-2 p-3 transition-all hover:scale-105"
                  :class="[
                    selectedSubPerks.includes(perkId)
                      ? 'border-primary bg-primary/10 shadow-md'
                      : 'border-border hover:border-primary/50',
                    selectedSubPerks.length >= 2 && !selectedSubPerks.includes(perkId)
                      ? 'opacity-50 cursor-not-allowed'
                      : ''
                  ]"
                >
                  <div class="flex flex-col items-center gap-2">
                    <img
                      :src="getPerkIconUrl(perkId)"
                      :alt="getPerkName(perkId)"
                      class="h-12 w-12 rounded"
                      @error="handleImageError"
                    />
                    <span class="text-xs text-center line-clamp-2 leading-tight">
                      {{ getPerkName(perkId) }}
                    </span>
                  </div>
                  <div
                    v-if="selectedSubPerks.includes(perkId)"
                    class="absolute top-2 right-2 h-5 w-5 rounded-full bg-primary flex items-center justify-center shadow-sm"
                  >
                    <Check class="h-3 w-3 text-primary-foreground" />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <Separator />

      <!-- 属性碎片 -->
      <div class="space-y-4">
        <h3 class="text-sm font-semibold text-foreground flex items-center gap-2">
          <Zap class="h-4 w-4 text-amber-500" />
          属性碎片
          <span class="text-xs text-muted-foreground font-normal">(每行3选1)</span>
        </h3>

        <div v-if="statModSlots.length > 0" class="space-y-3">
          <div v-for="(slot, slotIndex) in statModSlots" :key="slotIndex" class="space-y-2">
            <!-- 属性碎片分类标签 -->
            <div class="flex items-center gap-2">
              <div class="h-px flex-1 bg-border/50"></div>
              <div class="text-xs font-medium text-muted-foreground px-2 flex items-center gap-1">
                {{ slot.slotLabel }}
                <span class="text-[10px] text-muted-foreground/70">
                  {{ getStatModDescription(slotIndex) }}
                </span>
              </div>
              <div class="h-px flex-1 bg-border/50"></div>
            </div>

            <!-- 属性碎片选项 -->
            <div class="grid grid-cols-3 gap-3">
              <div
                v-for="perkId in slot.perks"
                :key="perkId"
                @click="selectStatMod(slotIndex, perkId)"
                class="relative cursor-pointer rounded-lg border-2 p-3 transition-all hover:scale-105"
                :class="[
                  selectedStatMods[slotIndex] === perkId
                    ? 'border-amber-500 bg-amber-500/10 shadow-md'
                    : 'border-border hover:border-amber-500/50'
                ]"
              >
                <div class="flex flex-col items-center gap-2">
                  <img
                    :src="getPerkIconUrl(perkId)"
                    :alt="getPerkName(perkId)"
                    class="h-10 w-10 rounded"
                    @error="handleImageError"
                  />
                  <span class="text-xs text-center line-clamp-2 leading-tight">
                    {{ getPerkName(perkId) }}
                  </span>
                </div>
                <div
                  v-if="selectedStatMods[slotIndex] === perkId"
                  class="absolute top-2 right-2 h-5 w-5 rounded-full bg-amber-500 flex items-center justify-center shadow-sm"
                >
                  <Check class="h-3 w-3 text-white" />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 选择总结 -->
      <div class="p-4 rounded-lg bg-muted/50">
        <div class="text-xs text-muted-foreground space-y-1">
          <div>已选择: {{ totalSelectedPerks }} / 9 个符文</div>
          <div v-if="totalSelectedPerks === 9" class="text-primary font-medium">✓ 符文配置完整</div>
          <div v-else class="text-destructive">还需选择 {{ 9 - totalSelectedPerks }} 个符文</div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRuneData } from '@/shared/composables/game/useRuneData'
import { Sparkles, Check, Zap, AlertCircle } from 'lucide-vue-next'

interface Props {
  primaryStyleId: number
  subStyleId: number
  selectedPerkIds: number[]
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'update:primaryStyleId': [value: number]
  'update:subStyleId': [value: number]
  'update:selectedPerkIds': [value: number[]]
}>()

// 符文数据
const runeData = useRuneData()
const { perkStyles, perks, isLoading, error } = runeData

// 本地状态
const selectedPrimaryPerks = ref<number[]>([])
const selectedSubPerks = ref<number[]>([])
const selectedStatMods = ref<number[]>([])

// 计算属性
const primaryStyle = computed(() => perkStyles.value.find((s: any) => s.id === props.primaryStyleId))

const subStyle = computed(() => perkStyles.value.find((s: any) => s.id === props.subStyleId))

const availableSubStyles = computed(() => {
  if (!primaryStyle.value) return []
  return perkStyles.value.filter((s: any) => primaryStyle.value!.allowedSubStyles.includes(s.id))
})

const primaryStyleSlots = computed(() => {
  if (!primaryStyle.value) return []
  // 只返回主系符文槽位（基石 + 普通符文），排除属性碎片
  return primaryStyle.value.slots.filter((s: any) => s.type === 'kKeyStone' || s.type === 'kMixedRegularSplashable')
})

// 副系符文按层级分组
const subStyleSlots = computed(() => {
  if (!subStyle.value) return []
  // 只返回普通符文槽位（排除基石和属性碎片）
  return subStyle.value.slots.filter((s: any) => s.type === 'kMixedRegularSplashable')
})

// 保留旧的 subStylePerks 用于兼容性
const subStylePerks = computed(() => {
  const allPerks: number[] = []
  subStyleSlots.value.forEach((slot: any) => {
    allPerks.push(...slot.perks)
  })
  return allPerks
})

// 属性碎片槽位 - 只从主系获取（所有符文系的属性碎片都相同）
const statModSlots = computed(() => {
  if (!primaryStyle.value) return []
  // 属性碎片在所有符文系中都是相同的，只需要从主系获取一次
  return primaryStyle.value.slots.filter((s: any) => s.type === 'kStatMod')
})

const totalSelectedPerks = computed(() => {
  return selectedPrimaryPerks.value.length + selectedSubPerks.value.length + selectedStatMods.value.length
})

// 方法
const selectPrimaryStyle = (styleId: number) => {
  emit('update:primaryStyleId', styleId)
  // 重置主系符文选择
  selectedPrimaryPerks.value = []
  // 如果当前副系不可用，重置副系
  const newPrimaryStyle = perkStyles.value.find((s: any) => s.id === styleId)
  if (newPrimaryStyle && !newPrimaryStyle.allowedSubStyles.includes(props.subStyleId)) {
    emit('update:subStyleId', newPrimaryStyle.allowedSubStyles[0] || 8000)
    selectedSubPerks.value = []
  }
  updateSelectedPerkIds()
}

const selectSubStyle = (styleId: number) => {
  emit('update:subStyleId', styleId)
  // 重置副系符文选择
  selectedSubPerks.value = []
  updateSelectedPerkIds()
}

const selectPerk = (slotIndex: number, perkId: number) => {
  selectedPrimaryPerks.value[slotIndex] = perkId
  updateSelectedPerkIds()
}

const toggleSubPerk = (perkId: number) => {
  const index = selectedSubPerks.value.indexOf(perkId)
  if (index > -1) {
    // 已选中，取消选择
    selectedSubPerks.value.splice(index, 1)
  } else {
    // 未选中，添加选择（最多2个）
    if (selectedSubPerks.value.length < 2) {
      selectedSubPerks.value.push(perkId)
    }
  }
  updateSelectedPerkIds()
}

const selectStatMod = (slotIndex: number, perkId: number) => {
  selectedStatMods.value[slotIndex] = perkId
  updateSelectedPerkIds()
}

const updateSelectedPerkIds = () => {
  const allPerks = [
    ...selectedPrimaryPerks.value.filter((id) => id > 0),
    ...selectedSubPerks.value,
    ...selectedStatMods.value.filter((id) => id > 0)
  ]
  emit('update:selectedPerkIds', allPerks)
}

const getPerkIconUrl = (perkId: number) => {
  const perk = perks.value.find((p: any) => p.id === perkId)
  if (perk) {
    return runeData.getPerkIconUrl(perk.iconPath)
  }
  return ''
}

const getPerkName = (perkId: number) => {
  const perk = perks.value.find((p: any) => p.id === perkId)
  return perk?.name || `符文 ${perkId}`
}

const getPrimaryStyleColor = (styleId: number) => {
  const colors: Record<number, string> = {
    8000: '#C8AA6E', // 精密 - 金色
    8100: '#E74C3C', // 主宰 - 红色
    8200: '#3498DB', // 巫术 - 蓝色
    8300: '#9B59B6', // 启迪 - 紫色
    8400: '#2ECC71' // 坚决 - 绿色
  }
  return colors[styleId] || '#888888'
}

const getSubStyleColor = (styleId: number) => {
  return getPrimaryStyleColor(styleId)
}

const getSlotLabel = (type: string, index: number) => {
  if (type === 'kKeyStone') return '基石符文'
  if (type === 'kStatMod') {
    return ['进攻', '灵活', '防御'][index - 4] || '属性'
  }
  return `第 ${index + 1} 层`
}

const getStatModDescription = (slotIndex: number) => {
  const descriptions = ['攻击力/法强/攻速', '自适应/攻速/CD', '生命值/护甲/魔抗']
  return descriptions[slotIndex] || ''
}

const handleImageError = (e: Event) => {
  const img = e.target as HTMLImageElement
  img.style.display = 'none'
}

// 初始化
onMounted(async () => {
  // 加载符文数据
  if (perkStyles.value.length === 0) {
    await runeData.loadRuneData()
  }

  // 从 props 初始化选中的符文
  initializeFromProps()
})

watch(
  () => props.selectedPerkIds,
  () => {
    initializeFromProps()
  }
)

const initializeFromProps = () => {
  if (props.selectedPerkIds.length === 0) {
    // 使用默认值
    selectedPrimaryPerks.value = []
    selectedSubPerks.value = []
    selectedStatMods.value = []
    return
  }

  // 解析已选中的符文
  const perkIds = [...props.selectedPerkIds]

  // 主系符文（前4个）
  selectedPrimaryPerks.value = perkIds.slice(0, 4)

  // 副系符文（中间2个）
  selectedSubPerks.value = perkIds.slice(4, 6)

  // 属性碎片（后3个）
  selectedStatMods.value = perkIds.slice(6, 9)
}
</script>
