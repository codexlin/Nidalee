<template>
  <div class="space-y-3">
    <!-- 选择器标题 -->
    <div class="flex items-center gap-2">
      <Gamepad2 class="h-4 w-4 text-primary" />
      <span class="text-sm font-medium text-foreground">游戏类型过滤</span>
    </div>

    <!-- 快速选择按钮 -->
    <div class="flex flex-wrap gap-2">
      <Button
        v-for="preset in presets"
        :key="preset.key"
        variant="outline"
        size="sm"
        @click="togglePreset(preset.key)"
        class="text-xs transition-colors text-foreground"
        :class="{
          '!bg-primary !text-primary-foreground !border-primary': preset.selected,
          '!bg-secondary/80 !border-secondary': preset.partial && !preset.selected
        }"
      >
        {{ preset.label }}
      </Button>
    </div>

    <!-- 类型选择器 -->
    <div class="grid grid-cols-4 gap-2">
      <div
        v-for="type in gameTypes"
        :key="type.id"
        class="flex items-center space-x-2 p-3 rounded-lg border cursor-pointer transition-colors"
        :class="{
          'bg-primary/10 border-primary text-primary-foreground': selectedTypes.includes(type.id),
          'border-border bg-card hover:bg-accent hover:text-accent-foreground': !selectedTypes.includes(type.id)
        }"
        @click="toggleType(type.id)"
      >
        <Checkbox :checked="selectedTypes.includes(type.id)" @click.stop @update:checked="() => toggleType(type.id)" />
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <component
              :is="type.icon"
              class="h-4 w-4 transition-colors text-primary"
              :class="{
                'text-primary': selectedTypes.includes(type.id),
                'text-primary-400 dark:text-primary-300': !selectedTypes.includes(type.id)
              }"
            />
            <span class="text-sm font-medium truncate text-foreground">{{ type.name }}</span>
          </div>
          <p
            class="text-xs truncate transition-colors"
            :class="{
              'text-primary/80': selectedTypes.includes(type.id),
              'text-slate-500 dark:text-slate-400': !selectedTypes.includes(type.id)
            }"
          >
            {{ type.description }}
          </p>
        </div>
      </div>
    </div>

    <!-- 操作按钮 -->
    <div class="flex gap-2 pt-3 border-t border-border">
      <Button
        variant="outline"
        size="sm"
        @click="selectAll"
        :disabled="selectedTypes.length === gameTypes.length"
        class="flex-1 text-foreground"
      >
        全选
      </Button>
      <Button
        variant="outline"
        size="sm"
        @click="clearAll"
        :disabled="selectedTypes.length === 0"
        class="flex-1 text-foreground"
      >
        清空
      </Button>
      <Button variant="outline" size="sm" @click="resetToDefault" class="flex-1 text-foreground"> 重置 </Button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Gamepad2 } from 'lucide-vue-next'
import { gameTypes, gameTypePresets, type GameTypePreset } from '@/common'

// 预设选择
const presets = ref<GameTypePreset[]>(
  gameTypePresets.map((preset) => ({
    ...preset,
    selected: false,
    partial: false
  }))
)

const selectedTypes = defineModel<number[]>('selectedTypes', { default: () => [] as number[] })

// 计算预设选择状态
const updatePresetStates = () => {
  presets.value.forEach((preset) => {
    // 改进逻辑：如果预设中的所有类型都被选中，则预设为完全选中状态
    // 如果预设中的部分类型被选中，则预设为部分选中状态
    const selectedCount = preset.types.filter((type) => selectedTypes.value.includes(type)).length

    preset.selected = selectedCount === preset.types.length
    preset.partial = selectedCount > 0 && selectedCount < preset.types.length
  })
}

// 监听选择变化，更新预设状态
watch(selectedTypes, updatePresetStates, { immediate: true })

// 切换类型选择
const toggleType = (typeId: number) => {
  const index = selectedTypes.value.indexOf(typeId)
  if (index > -1) {
    // 创建新数组以确保响应性
    selectedTypes.value = selectedTypes.value.filter((id) => id !== typeId)
  } else {
    // 创建新数组以确保响应性
    selectedTypes.value = [...selectedTypes.value, typeId]
  }
}

// 切换预设
const togglePreset = (presetKey: string) => {
  const preset = presets.value.find((p) => p.key === presetKey)
  if (!preset) return

  if (preset.selected) {
    // 取消选择预设中的所有类型
    selectedTypes.value = selectedTypes.value.filter((id) => !preset.types.includes(id))
  } else {
    // 选择预设中的所有类型（合并，避免重复）
    const newTypes = [...new Set([...selectedTypes.value, ...preset.types])]
    selectedTypes.value = newTypes
  }
}

// 全选
const selectAll = () => {
  selectedTypes.value = gameTypes.map((type) => type.id)
}

// 清空
const clearAll = () => {
  selectedTypes.value = []
}

// 重置为默认（选择所有排位赛类型）
const resetToDefault = () => {
  selectedTypes.value = [420, 440] // 单双排 + 灵活组排
}

// 获取选中的类型名称
const getSelectedTypeNames = computed(() => {
  return selectedTypes.value.map((id) => gameTypes.find((type) => type.id === id)?.name).filter(Boolean)
})

// 暴露方法给父组件
defineExpose({
  getSelectedTypeNames,
  resetToDefault
})
</script>
