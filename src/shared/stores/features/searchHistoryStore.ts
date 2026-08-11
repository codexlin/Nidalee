const MAX_HISTORY = 10

export const useSearchHistoryStore = defineStore(
  'searchHistory',
  () => {
    /** 最近查询成功的召唤师名（新 → 旧） */
    const items = ref<string[]>([])

    const add = (names: string | string[]) => {
      const list = (Array.isArray(names) ? names : [names])
        .map((n) => n.trim())
        .filter(Boolean)
      if (!list.length) return

      let next = [...items.value]
      for (const name of list) {
        next = next.filter((item) => item.toLowerCase() !== name.toLowerCase())
        next.unshift(name)
      }
      items.value = next.slice(0, MAX_HISTORY)
    }

    const remove = (name: string) => {
      items.value = items.value.filter((item) => item.toLowerCase() !== name.toLowerCase())
    }

    const clear = () => {
      items.value = []
    }

    return { items, add, remove, clear }
  },
  { persist: true }
)
