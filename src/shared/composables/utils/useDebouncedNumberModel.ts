import { type Ref, computed, ref, watch } from 'vue'
import { useDebounceFn } from '@vueuse/core'

export interface DebouncedNumberModelOptions {
  delay?: number
  min?: number
  max?: number
  step?: number
}

/**
 * 为数字模型创建防抖版本
 * 用于滑块、输入框等需要防抖的场景
 */
export function useDebouncedNumberModel(
  model: Ref<number>,
  options: DebouncedNumberModelOptions = {}
) {
  const { delay = 500 } = options

  const pendingValue = ref<number>(model.value)
  const isPending = ref(false)

  // 防抖更新函数
  const updateModel = useDebounceFn((value: number) => {
    model.value = value
    isPending.value = false
  }, delay)

  // 立即刷新待处理的值
  const flush = () => {
    if (isPending.value) {
      model.value = pendingValue.value
      isPending.value = false
    }
  }

  // 监听待处理值的变化
  watch(pendingValue, (newValue) => {
    const clampedValue = Math.max(
      options.min ?? Number.NEGATIVE_INFINITY,
      Math.min(options.max ?? Number.POSITIVE_INFINITY, newValue)
    )
    isPending.value = true
    updateModel(clampedValue)
  })

  // 监听模型变化，同步待处理值
  watch(model, (newValue) => {
    if (!isPending.value) {
      pendingValue.value = newValue
    }
  })

  return {
    value: computed({
      get: () => pendingValue.value,
      set: (val: number) => {
        pendingValue.value = val
      }
    }),
    isPending: computed(() => isPending.value),
    flush
  }
}
