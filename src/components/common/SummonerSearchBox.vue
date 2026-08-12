<template>
  <div class="w-full" :class="compact ? '' : 'mx-auto max-w-xl'">
    <div class="surface-raised w-full" :class="compact ? 'p-3' : 'p-4'">
      <form class="flex items-center gap-2" @submit.prevent="submit">
        <Input
          v-model="summonerName"
          type="text"
          placeholder="输入召唤师名称，多个用 , 分隔"
          class="h-10 flex-1 text-sm"
          :disabled="loading"
          @keyup.enter="submit"
        />
        <FloatIconButton
          type="submit"
          variant="pill"
          class="shrink-0"
          :disabled="loading || !summonerName.trim()"
          :title="loading ? '查询中…' : '查询战绩'"
        >
          <Loader2 v-if="loading" class="size-4 animate-spin" />
          <Search v-else class="size-4" />
          {{ loading ? '查询中' : '查询' }}
        </FloatIconButton>
      </form>
      <p v-if="!compact" class="mt-2 text-xs text-muted-foreground">支持批量查询，使用英文逗号分隔多名召唤师</p>
    </div>

    <!-- 空态：最近搜索 -->
    <div v-if="showHistory && historyItems.length" class="mt-3 space-y-2">
      <div class="flex items-center justify-between px-0.5">
        <span class="text-xs text-muted-foreground">最近搜索</span>
        <button
          type="button"
          class="text-xs text-muted-foreground outline-none transition-colors hover:text-foreground"
          @click="searchHistoryStore.clear()"
        >
          清空
        </button>
      </div>
      <div class="flex flex-wrap gap-1.5">
        <button
          v-for="name in historyItems"
          :key="name"
          type="button"
          class="surface-chip inline-flex max-w-full items-center gap-1 rounded-lg px-2.5 py-1 text-xs font-medium text-muted-foreground outline-none transition-colors hover:bg-muted/50 hover:text-foreground focus-visible:ring-ring/50 focus-visible:ring-[3px]"
          :disabled="loading"
          :title="`查询 ${name}`"
          @click="quickSearch(name)"
        >
          <span class="truncate">{{ name }}</span>
          <span
            class="rounded p-0.5 text-muted-foreground/70 hover:bg-muted hover:text-foreground"
            title="移除"
            @click.stop="searchHistoryStore.remove(name)"
          >
            <X class="size-3" />
          </span>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Loader2, Search, X } from 'lucide-vue-next'
import FloatIconButton from '@/components/common/FloatIconButton.vue'
import { useSearchHistoryStore } from '@/shared/stores/features/searchHistoryStore'
import { storeToRefs } from 'pinia'

const props = withDefaults(
  defineProps<{
    loading?: boolean
    /** 有结果时顶栏式矮搜索条 */
    compact?: boolean
    /** 展示最近搜索（空态） */
    showHistory?: boolean
  }>(),
  {
    loading: false,
    compact: false,
    showHistory: false
  }
)

const summonerName = defineModel<string>('summonerName', { default: '' })
const emit = defineEmits<{ onSearch: [] }>()

const searchHistoryStore = useSearchHistoryStore()
const { items: historyItems } = storeToRefs(searchHistoryStore)

const submit = () => {
  if (!summonerName.value.trim()) return
  emit('onSearch')
}

const quickSearch = (name: string) => {
  if (props.loading) return
  summonerName.value = name
  emit('onSearch')
}
</script>
